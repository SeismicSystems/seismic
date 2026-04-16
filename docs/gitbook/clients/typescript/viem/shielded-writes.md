---
description: Send encrypted write transactions with shieldedWriteContract
icon: pen
---

# Shielded Writes

Shielded writes encrypt transaction calldata before submission. This prevents calldata from being visible on-chain -- an observer can see that a transaction was sent to a particular contract address, but not what function was called or what arguments were passed.

seismic-viem provides several approaches:

- **`contract.write.functionName()`** -- smart routing via [getShieldedContract](contract-instance.md). Auto-detects shielded parameters and encrypts only when needed.
- **`contract.swrite.functionName()`** -- force shielded via [getShieldedContract](contract-instance.md). Always encrypts, regardless of parameter types.
- **`shieldedWriteContract()`** -- standalone function, same API shape as viem's `writeContract`. Always encrypts.
- **`walletClient.writeContract()`** -- smart routing via the wallet client. Same auto-detection as `contract.write`.
- **`walletClient.swriteContract()`** -- force shielded via the wallet client. Always encrypts.

The shielded paths all produce the same on-chain result: an encrypted type `0x4A` Seismic transaction.

## Standalone: `shieldedWriteContract`

```typescript
import { shieldedWriteContract } from "seismic-viem";
```

### Parameters

| Parameter      | Type     | Required | Description        |
| -------------- | -------- | -------- | ------------------ |
| `address`      | `Hex`    | Yes      | Contract address   |
| `abi`          | `Abi`    | Yes      | Contract ABI       |
| `functionName` | `string` | Yes      | Function to call   |
| `args`         | `array`  | No       | Function arguments |
| `gas`          | `bigint` | No       | Gas limit          |
| `gasPrice`     | `bigint` | No       | Gas price          |
| `value`        | `bigint` | No       | ETH value to send  |

### Returns

`Promise<Hash>` -- the transaction hash.

### Example

```typescript
import { shieldedWriteContract } from "seismic-viem";

const hash = await shieldedWriteContract(client, {
  address: "0x1234567890abcdef1234567890abcdef12345678",
  abi: myContractAbi,
  functionName: "transfer",
  args: ["0xRecipient...", 100n],
  gas: 100_000n,
});
```

---

## Send + Inspect: `shieldedWriteContractDebug`

Broadcasts a shielded transaction (like `shieldedWriteContract`) and additionally returns the plaintext transaction view and the shielded (encrypted) transaction view alongside the resulting transaction hash. Useful for inspecting exactly what the SDK encrypted and submitted.

{% hint style="info" %}
Despite the "debug" flavor of the name, `shieldedWriteContractDebug` does send the transaction -- the returned `txHash` is a real on-chain hash, not a dry run.
{% endhint %}

```typescript
import { shieldedWriteContractDebug } from "seismic-viem";

const { plaintextTx, shieldedTx, txHash } = await shieldedWriteContractDebug(
  client,
  {
    address: "0x1234567890abcdef1234567890abcdef12345678",
    abi: myContractAbi,
    functionName: "transfer",
    args: ["0xRecipient...", 100n],
  },
);

console.log("Plaintext calldata:", plaintextTx.data);
console.log("Encrypted calldata:", shieldedTx.data);
console.log("Transaction hash:", txHash);
```

---

## Low-level: `sendShieldedTransaction`

For cases where you have raw transaction data instead of ABI-encoded calls -- for example, contract deployments, pre-encoded calldata, or bypassing the contract abstraction entirely:

```typescript
const hash = await client.sendShieldedTransaction({
  to: "0x1234567890abcdef1234567890abcdef12345678",
  data: "0x...", // raw calldata
  value: 0n,
  gas: 100_000n,
  gasPrice: 1_000_000_000n,
});
```

---

## How It Works

When you call `shieldedWriteContract` (or `contract.swrite.functionName`), the SDK performs the following steps:

1. **ABI-encode** the function call into plaintext calldata
2. **Build Seismic metadata** -- encryption nonce, recent block hash, expiry block
3. **Encrypt calldata** with AES-GCM using a shared key derived via ECDH between your ephemeral keypair and the node's TEE public key
4. **Construct a type `0x4A` transaction** with the encrypted calldata and Seismic-specific fields
5. **Sign and broadcast** the transaction

The encrypted calldata is bound to the transaction context (chain ID, nonce, block hash, expiry) via AES-GCM additional authenticated data, so it cannot be replayed or tampered with.

---

## Security Parameters

Every shielded transaction includes a block-hash freshness check and an expiry window. The defaults are sensible for most cases, but you can override them per-call via `SeismicSecurityParams`:

| Parameter         | Type     | Default    | Description                                     |
| ----------------- | -------- | ---------- | ----------------------------------------------- |
| `blocksWindow`    | `bigint` | `100n`     | Number of blocks before the transaction expires |
| `encryptionNonce` | `Hex`    | Random     | Override the encryption nonce                   |
| `recentBlockHash` | `Hex`    | Latest     | Override the recent block hash                  |
| `expiresAtBlock`  | `bigint` | Calculated | Override the expiry block directly              |

```typescript
const hash = await shieldedWriteContract(client, {
  address: "0x...",
  abi: myContractAbi,
  functionName: "transfer",
  args: ["0x...", 100n],
  securityParams: {
    blocksWindow: 50n, // expires after 50 blocks instead of 100
  },
});
```

{% hint style="info" %}
The default 100-block window, random nonce, and latest block hash are appropriate for nearly all use cases. Override these only if you have a specific reason -- for example, reducing the window for time-sensitive operations or pinning the block hash in tests.
{% endhint %}

## See Also

- [Contract Instance](contract-instance.md) -- `getShieldedContract` with `.write` and `.dwrite` namespaces
- [Signed Reads](signed-reads.md) -- Authenticated reads that also use the encryption pipeline
- [Encryption](encryption.md) -- ECDH key exchange and AES-GCM details
- [Shielded Wallet Client](shielded-wallet-client.md) -- Creating the client used for shielded writes
