---
description: Register and look up SRC20 viewing keys in the Directory contract
icon: key
---

# Viewing Keys

SRC20 amounts are encrypted with an AES viewing key. Before an account can read its own transfers, that key has to be registered in the Directory contract at `0x1000000000000000000000000000000000000004` — exported as `DIRECTORY_ADDRESS`. This is the registry [`watchSRC20Events`](src20.md) reads from when it fetches the key for a connected wallet, and the reason that action throws when no key has been registered yet.

seismic-viem exposes the Directory surface as five standalone functions rather than client actions, so each takes the client as its first argument:

```typescript
import {
  checkRegistration,
  computeKeyHash,
  getKey,
  getKeyHash,
  registerKey,
} from "seismic-viem";
```

## `registerKey`

Registers an AES viewing key for the client's own account. The key travels in a shielded write, so it is never exposed in a public calldata.

```typescript
const txHash = await registerKey(walletClient, aesKey);
```

| Parameter | Type                   | Required | Description                                     |
| --------- | ---------------------- | -------- | ----------------------------------------------- |
| `client`  | `ShieldedWalletClient` | Yes      | The account the key is registered for           |
| `aesKey`  | `Hex`                  | Yes      | 32-byte AES viewing key                         |

**Returns:** `Promise<Hex>` — the transaction hash.

Calls `setKey` on the Directory. Registering again replaces the previous key, which makes every event encrypted under the old one unreadable through this path, so treat it as a rotation rather than an addition.

{% hint style="warning" %}
The call rejects with `Transaction timed out after 30000ms` if the write has not resolved within 30 seconds. The transaction may still land afterwards; re-check with `checkRegistration` before retrying.
{% endhint %}

## `checkRegistration`

Whether an address has any key registered. A plain public read of `checkHasKey`, so it works for any address, not only the client's own.

```typescript
const registered = await checkRegistration(walletClient, address);
```

| Parameter | Type                   | Required | Description              |
| --------- | ---------------------- | -------- | ------------------------ |
| `client`  | `ShieldedWalletClient` | Yes      | Client used for the read |
| `address` | `Address`              | Yes      | Address to check         |

**Returns:** `Promise<boolean>`

Useful as a guard before calling `watchSRC20Events`, which throws when the connected address has no key.

## `getKeyHash`

The `keccak256` hash of an address's registered key, read from the Directory. Also a plain public read: the hash is public by design, since it is what SRC20 logs carry as `encryptKeyHash` for on-chain filtering.

```typescript
const keyHash = await getKeyHash(walletClient, address);
```

| Parameter | Type                   | Required | Description              |
| --------- | ---------------------- | -------- | ------------------------ |
| `client`  | `ShieldedWalletClient` | Yes      | Client used for the read |
| `address` | `Address`              | Yes      | Address to look up       |

**Returns:** `Promise<Hex>` — a `bytes32` hash. Returns the zero hash for an address that has never registered.

## `getKey`

The client's **own** viewing key, in plaintext. This is a signed read, which is what limits the result to the caller's key: there is no `address` parameter because no account can read another's.

```typescript
const aesKey = await getKey(walletClient);
```

| Parameter | Type                   | Required | Description                          |
| --------- | ---------------------- | -------- | ------------------------------------ |
| `client`  | `ShieldedWalletClient` | Yes      | The account whose key is returned    |

**Returns:** `Promise<Hex>` — the key, zero-padded to 32 bytes.

Pass the result to [`watchSRC20EventsWithKey`](src20.md) when you want a public client to decrypt events for an account you control.

## `computeKeyHash`

The hash a key *would* register under. Pure and local: `keccak256(aesKey)`, no client and no network call.

```typescript
const expected = computeKeyHash(aesKey);
```

| Parameter | Type  | Required | Description             |
| --------- | ----- | -------- | ----------------------- |
| `aesKey`  | `Hex` | Yes      | 32-byte AES viewing key |

**Returns:** `Hex`

Compare it against `getKeyHash` to confirm a local key matches what is on-chain, without a signed read:

```typescript
const onChain = await getKeyHash(walletClient, walletClient.account.address);
const matches = computeKeyHash(localKey) === onChain;
```

## See Also

- [SRC20 Event Watching](src20.md) — the watchers that consume these keys
- [Signed Reads](signed-reads.md) — the mechanism behind `getKey`
- [Shielded Writes](shielded-writes.md) — the mechanism behind `registerKey`
