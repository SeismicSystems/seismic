---
description: Type-safe shielded contract instances with read/write/tread/twrite/dwrite namespaces
icon: file-contract
---

# Contract Instance

`getShieldedContract` creates a type-safe contract instance with five namespaces for interacting with shielded contracts. It extends viem's `getContract` with Seismic-specific read and write patterns, giving you a single object that covers encrypted writes, signed reads, transparent operations, and debug inspection.

```typescript
import { getShieldedContract } from "seismic-viem";
```

## Constructor

| Parameter | Type                   | Required | Description                                           |
| --------- | ---------------------- | -------- | ----------------------------------------------------- |
| `abi`     | `Abi`                  | Yes      | Contract ABI (use `as const` for full type inference) |
| `address` | `Address`              | Yes      | Deployed contract address                             |
| `client`  | `ShieldedWalletClient` | Yes      | Wallet client with encryption capabilities            |

```typescript
import { getShieldedContract } from "seismic-viem";

const abi = [
  {
    name: "balanceOf",
    type: "function",
    stateMutability: "view",
    inputs: [{ name: "account", type: "address" }],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    name: "transfer",
    type: "function",
    stateMutability: "nonpayable",
    inputs: [
      { name: "to", type: "address" },
      { name: "amount", type: "uint256" },
    ],
    outputs: [{ name: "", type: "bool" }],
  },
  {
    name: "totalSupply",
    type: "function",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "uint256" }],
  },
  {
    name: "approve",
    type: "function",
    stateMutability: "nonpayable",
    inputs: [
      { name: "spender", type: "address" },
      { name: "amount", type: "uint256" },
    ],
    outputs: [{ name: "", type: "bool" }],
  },
] as const;

const contract = getShieldedContract({
  abi,
  address: "0x1234567890abcdef1234567890abcdef12345678",
  client,
});
```

## Namespaces

The returned `ShieldedContract` exposes five namespaces. Each namespace provides every function defined in the ABI, but differs in how calldata is handled and who appears as `msg.sender` on-chain.

| Namespace | Operation Type    | Calldata  | msg.sender       | Description                             |
| --------- | ----------------- | --------- | ---------------- | --------------------------------------- |
| `.read`   | Signed Read       | Encrypted | Signer's address | Authenticated read -- proves identity   |
| `.write`  | Shielded Write    | Encrypted | Signer's address | Encrypted transaction                   |
| `.tread`  | Transparent Read  | Plaintext | Zero address     | Standard viem read                      |
| `.twrite` | Transparent Write | Plaintext | Signer's address | Standard viem write                     |
| `.dwrite` | Debug Write       | Encrypted | Signer's address | Returns plaintext + encrypted tx + hash |

---

### `.read` -- Signed Read

Sends an encrypted, signed `eth_call` that proves your identity to the contract. Use this when the contract checks `msg.sender` in a view function to gate access to shielded data.

```typescript
const balance = await contract.read.balanceOf(["0x1234..."]);
```

{% hint style="info" %}
The `.read` namespace always sets `account` to `client.account` for security. This ensures the signed read is authenticated with the wallet's address, regardless of any `account` override you pass.
{% endhint %}

See [Signed Reads](signed-reads.md) for details on how signed reads work under the hood.

---

### `.write` -- Shielded Write

Encrypts calldata before broadcasting the transaction. The calldata is not visible on-chain.

```typescript
const hash = await contract.write.transfer(["0x1234...", 100n], {
  gas: 100_000n,
});
```

See [Shielded Writes](shielded-writes.md) for the encryption lifecycle and security parameters.

---

### `.tread` -- Transparent Read

Standard viem `readContract` call with plaintext calldata. The `from` address is set to the zero address, so `msg.sender` inside the contract will be `0x0000...0000`.

```typescript
const supply = await contract.tread.totalSupply();
```

Use `.tread` for public view functions that do not depend on `msg.sender`.

---

### `.twrite` -- Transparent Write

Standard viem `writeContract` call with plaintext (unencrypted) calldata. The transaction is signed by the wallet, so `msg.sender` is the signer's address.

```typescript
const hash = await contract.twrite.approve(["0x1234...", 100n]);
```

Use `.twrite` when you intentionally want calldata to be visible on-chain (e.g., for transparency or debugging).

---

### `.dwrite` -- Debug Write

Builds the same encrypted transaction as `.write`, but returns the plaintext transaction, the shielded transaction, and the transaction hash without broadcasting. Useful for inspecting what the SDK produces before sending.

```typescript
const { plaintextTx, shieldedTx, txHash } = await contract.dwrite.transfer([
  "0x1234...",
  100n,
]);

console.log("Plaintext calldata:", plaintextTx.data);
console.log("Encrypted calldata:", shieldedTx.data);
console.log("Transaction hash:", txHash);
```

## TypeScript ABI Typing

For full type inference on function names, argument types, and return types, declare your ABI with `as const`:

```typescript
const abi = [
  {
    name: "balanceOf",
    type: "function",
    stateMutability: "view",
    inputs: [{ name: "account", type: "address" }],
    outputs: [{ name: "", type: "uint256" }],
  },
] as const;

const contract = getShieldedContract({ abi, address: "0x...", client });

// TypeScript knows:
//   - contract.read.balanceOf exists
//   - first arg is [Address]
//   - return type is bigint
const balance = await contract.read.balanceOf(["0x1234..."]);
```

Without `as const`, the ABI is widened to `readonly AbiItem[]` and the contract loses per-function type safety. You can still call functions by name, but arguments and return values will be untyped.

## See Also

- [Shielded Writes](shielded-writes.md) -- Encryption lifecycle and `shieldedWriteContract`
- [Signed Reads](signed-reads.md) -- Authenticated reads and `signedReadContract`
- [Shielded Wallet Client](shielded-wallet-client.md) -- Creating the client passed to `getShieldedContract`
- [Encryption](encryption.md) -- ECDH key exchange and AES-GCM calldata encryption
