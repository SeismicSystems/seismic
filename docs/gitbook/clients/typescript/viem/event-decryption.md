---
description: Decrypt SRC20 transfer and approval events using viewing keys and the Directory contract.
icon: lock
---

# Event Decryption

Seismic emits SRC20 transfer and approval events with an **encrypted payload**. Reading the raw log only reveals the sender / spender address and an opaque ciphertext — the amount stays private until decrypted with the right viewing key. This page walks through the full decryption flow: registering a viewing key, watching events with automatic decryption, and inspecting the resulting log types.

## Import

```typescript
import {
  watchSRC20Events,
  watchSRC20EventsWithKey,
  registerKey,
  getKey,
  getKeyHash,
  checkRegistration,
  computeKeyHash,
  src20PublicActions,
  src20WalletActions,
  type DecryptedTransferLog,
  type DecryptedApprovalLog,
} from 'seismic-viem'
```

## Overview

Encrypted SRC20 events use **AES-GCM** symmetric encryption. Each account that wants to emit decryptable events registers an AES key with the on-chain **Directory** contract; recipients then fetch that key (either by address or as their own key) and decrypt incoming logs locally.

| Capability | Function | Client |
|---|---|---|
| Generate the on-chain key hash | `computeKeyHash` | — (pure) |
| Register your viewing key | `registerKey` | `ShieldedWalletClient` |
| Check whether an address is registered | `checkRegistration` | `ShieldedWalletClient` |
| Read another account's key hash | `getKeyHash` | `ShieldedWalletClient` |
| Read your own viewing key | `getKey` | `ShieldedWalletClient` (signed read) |
| Watch + auto-decrypt events | `watchSRC20Events` | `ShieldedWalletClient` |
| Watch + decrypt with explicit key | `watchSRC20EventsWithKey` | `ShieldedPublicClient` |

{% hint style="info" %}
For the practical "subscribe and listen" example without the cryptography details, see [SRC20 Event Watching](src20.md). This page focuses on the decryption primitives themselves.
{% endhint %}

## The Decryption Flow

```
┌───────────────────────────────────────────────────────────────────┐
│  1. registerKey(aesKey)          ──► Directory contract           │
│        (one-time setup)              stores keccak256(aesKey)     │
├───────────────────────────────────────────────────────────────────┤
│  2. watchSRC20Events                                              │
│        ├─ subscribe to raw logs                                   │
│        ├─ fetch viewing key (getKey or provided)                  │
│        ├─ AES-GCM decrypt the payload                             │
│        └─ yield Decrypted{Transfer,Approval}Log                   │
└───────────────────────────────────────────────────────────────────┘
```

## Viewing Key Management

### `registerKey`

Registers an AES key for the caller's address on the on-chain Directory. Subsequent calls to `getKey` (signed) or `getKeyHash` (public) will resolve to this key.

```typescript
const txHash = await registerKey(walletClient, aesKey)
```

| Parameter | Type | Description |
|---|---|---|
| `client` | `ShieldedWalletClient` | The wallet client whose address will be associated with the key. |
| `aesKey` | `Hex` | 32-byte AES-GCM key (hex-encoded, with `0x` prefix). |

**Returns:** `Promise<Hex>` — the transaction hash of the shielded write.

{% hint style="warning" %}
`registerKey` issues a **shielded write** to the Directory contract. Re-registering overwrites the previous key for the caller's address, and any subscribers using the old key will no longer be able to decrypt subsequent events.
{% endhint %}

### `computeKeyHash`

Pure helper that computes the on-chain key hash (`keccak256(aesKey)`) without touching the network. Useful for comparing against `getKeyHash` output or sanity-checking a key before registration.

```typescript
const expectedHash = computeKeyHash(aesKey)
```

| Parameter | Type | Description |
|---|---|---|
| `aesKey` | `Hex` | The AES key to hash. |

**Returns:** `Hex` — `keccak256(aesKey)`.

### `checkRegistration`

Returns whether an address has registered a viewing key on the Directory.

```typescript
const isRegistered = await checkRegistration(walletClient, address)
```

| Parameter | Type | Description |
|---|---|---|
| `client` | `ShieldedWalletClient` | Any wallet client; only used for the network read. |
| `address` | `Address` | The address to check. |

**Returns:** `Promise<boolean>`.

### `getKeyHash`

Reads the on-chain key hash for an address. This is a **public** read — anyone can call it, and it does not reveal the AES key itself.

```typescript
const keyHash = await getKeyHash(walletClient, address)
```

| Parameter | Type | Description |
|---|---|---|
| `client` | `ShieldedWalletClient` | Any wallet client. |
| `address` | `Address` | The address whose key hash to read. |

**Returns:** `Promise<Hex>` — the stored `keccak256(aesKey)`, or `0x00..00` if no key is registered.

### `getKey`

Reads the **caller's own** AES key via a [signed read](signed-reads.md) against the Directory contract. The signature proves ownership of the address, so the call returns the plaintext key.

```typescript
const aesKey = await getKey(walletClient)
```

| Parameter | Type | Description |
|---|---|---|
| `client` | `ShieldedWalletClient` | The wallet client; the signer's address determines whose key is fetched. |

**Returns:** `Promise<Hex>` — the AES key registered for `client.account.address`.

{% hint style="warning" %}
`getKey` only resolves your own key. Reading another account's key requires that account to share it off-chain (typically via `watchSRC20EventsWithKey`).
{% endhint %}

## Action Extensions

Instead of importing each helper individually, you can extend a client with all SRC20 actions at once.

### `src20PublicActions`

Extends a `PublicClient` (or `ShieldedPublicClient`) with read-only SRC20 actions: `checkRegistration`, `getKeyHash`, `watchSRC20EventsWithKey`.

```typescript
const client = createShieldedPublicClient({ chain, transport })
  .extend(src20PublicActions)

await client.checkRegistration({ address })
```

### `src20WalletActions`

Extends a `ShieldedWalletClient` with the full SRC20 surface, including the shielded-write actions `registerKey` and `getKey`, plus `watchSRC20Events`.

```typescript
const client = createShieldedWalletClient({ chain, transport, account })
  .extend(src20WalletActions)

await client.registerKey({ aesKey })
await client.watchSRC20Events({ ... })
```

## Watching Decrypted Events

The two watcher functions return a callback-driven subscription that yields decrypted logs. They differ only in **how the viewing key is sourced**:

- `watchSRC20Events` — automatically calls `getKey` on the wallet client to fetch the caller's registered key.
- `watchSRC20EventsWithKey` — accepts an explicit `viewingKey` argument; useful when the caller does not own the key (e.g. a public indexer watching events for a user that shared their key off-chain).

For full parameter tables and usage examples, see [SRC20 Event Watching](src20.md).

## Decrypted Log Types

### `DecryptedTransferLog`

Yielded by both watchers when an SRC20 `Transfer` event is decrypted.

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | The SRC20 token contract that emitted the event. |
| `blockNumber` | `bigint` | Block in which the transfer occurred. |
| `transactionHash` | `Hex` | Hash of the transaction that emitted the event. |
| `args.from` | `Address` | The sender address (plaintext on-chain). |
| `args.to` | `Address` | The recipient address (plaintext on-chain). |
| `args.value` | `bigint` | The transfer amount (decrypted client-side). |

The Solidity event signature these logs decode from is:

```solidity
event Transfer(address indexed from, address indexed to, bytes encryptedValue);
```

### `DecryptedApprovalLog`

Yielded when an SRC20 `Approval` event is decrypted.

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | The SRC20 token contract that emitted the event. |
| `blockNumber` | `bigint` | Block in which the approval occurred. |
| `transactionHash` | `Hex` | Hash of the transaction. |
| `args.owner` | `Address` | The token holder granting the allowance. |
| `args.spender` | `Address` | The spender being approved. |
| `args.value` | `bigint` | The approved amount (decrypted client-side). |

The matching event signature:

```solidity
event Approval(address indexed owner, address indexed spender, bytes encryptedValue);
```

## End-to-End Example

Register a viewing key, then watch your own transfers as they are decrypted in real time:

```typescript
import { createShieldedWalletClient, seismicTestnet } from 'seismic-viem'
import {
  registerKey,
  checkRegistration,
  watchSRC20Events,
  computeKeyHash,
  type DecryptedTransferLog,
} from 'seismic-viem'
import { generatePrivateKey } from 'viem/accounts'

const walletClient = createShieldedWalletClient({
  chain: seismicTestnet,
  transport: http(),
  account,
})

// 1. Generate a fresh AES key and register it (one-time setup).
const aesKey = generatePrivateKey() // 32 random bytes, hex-encoded
const alreadyRegistered = await checkRegistration(walletClient, account.address)

if (!alreadyRegistered) {
  await registerKey(walletClient, aesKey)
}

// 2. (Optional) Verify the on-chain hash matches.
const expected = computeKeyHash(aesKey)

// 3. Subscribe — every emitted Transfer event for this account is decrypted.
const unsubscribe = await watchSRC20Events(walletClient, {
  address: tokenAddress,
  onLogs: (logs: DecryptedTransferLog[]) => {
    for (const log of logs) {
      console.log(`${log.args.from} -> ${log.args.to}: ${log.args.value}`)
    }
  },
})

// Later, stop watching:
unsubscribe()
```

## See Also

- [SRC20 Event Watching](src20.md) — practical usage of `watchSRC20Events` and `watchSRC20EventsWithKey`.
- [Encryption](encryption.md) — the lower-level AES-GCM primitives used internally.
- [Signed Reads](signed-reads.md) — the mechanism `getKey` uses to authenticate the caller.
- [Python: Register Viewing Key](../../python/src20/intelligence-providers/register-viewing-key.md) — equivalent functionality in the Python client.
