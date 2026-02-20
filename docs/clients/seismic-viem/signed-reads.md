---
description: Authenticated read calls that prove caller identity
icon: signature
---

# Signed Reads

On standard Ethereum, `eth_call` can spoof any `from` address -- there is no signature check. Seismic prevents this: all unsigned `eth_call` requests have `msg.sender` set to the zero address. To read shielded data that depends on `msg.sender`, you need a **signed read**: a signed transaction submitted to `eth_call` that proves the caller's identity.

## Why Signed Reads Matter

Contracts can use `msg.sender` in view functions to gate access to shielded data. A common example: a token contract with a `balanceOf()` that takes no arguments and uses `msg.sender` internally to look up the caller's balance. Without a signed read, the contract sees the zero address and returns that address's balance -- which is almost certainly zero.

```typescript
// Signed read -- msg.sender = your address
const myBalance = await contract.read.balanceOf(["0x1234..."]);

// Transparent read -- msg.sender = zero address
const totalSupply = await contract.tread.totalSupply();
```

seismic-viem provides two approaches:

- **`signedReadContract()`** -- standalone function, same API shape as viem's `readContract`
- **`contract.read.functionName()`** -- via a `ShieldedContract` from [getShieldedContract](contract-instance.md)

Both encrypt the calldata, sign it, and decrypt the response automatically.

---

## Standalone: `signedReadContract`

```typescript
import { signedReadContract } from "seismic-viem";
```

### Parameters

| Parameter      | Type     | Required | Description                    |
| -------------- | -------- | -------- | ------------------------------ |
| `address`      | `Hex`    | Yes      | Contract address               |
| `abi`          | `Abi`    | Yes      | Contract ABI                   |
| `functionName` | `string` | Yes      | Name of the view/pure function |
| `args`         | `array`  | No       | Function arguments             |
| `nonce`        | `number` | No       | Override the nonce             |

### Returns

`Promise<ReadContractReturnType>` -- the decoded return value, typed according to the ABI.

### Example

```typescript
import { signedReadContract } from "seismic-viem";

const balance = await signedReadContract(client, {
  address: "0x1234567890abcdef1234567890abcdef12345678",
  abi: myContractAbi,
  functionName: "balanceOf",
  args: ["0xMyAddress..."],
});
```

---

## Low-level: `signedCall`

For cases where you have raw calldata instead of ABI-encoded parameters -- for example, pre-encoded data or non-ABI interactions:

```typescript
import { signedCall } from "seismic-viem";

const result = await signedCall(client, {
  to: "0x1234567890abcdef1234567890abcdef12345678",
  data: "0x...", // raw calldata
  account: client.account,
  gas: 30_000_000n,
});
```

`signedCall` also accepts `SeismicSecurityParams` as a third argument for overriding the encryption nonce, block hash, or expiry window. See [Shielded Writes](shielded-writes.md#security-parameters) for the full parameter table.

---

## How It Works

When you call `signedReadContract` (or `contract.read.functionName`), the SDK performs the following steps:

1. **ABI-encode** the function call into plaintext calldata
2. **Build Seismic metadata** with `signedRead: true`
3. **Encrypt calldata** with AES-GCM using the shared key derived via ECDH
4. **Sign the transaction:**
   - For **local accounts** (private key): sign as a raw Seismic transaction, send to `eth_call`
   - For **JSON-RPC accounts** (MetaMask): sign EIP-712 typed data via `eth_signTypedData_v4`, send the typed data + signature to `eth_call`
5. **Decrypt the response** returned by the node
6. **Decode the ABI output** into the expected return type

Both the calldata you send and the result you receive are encrypted. An observer watching the network can see that a call was made to a particular contract address, but not what function was called or what was returned.

---

## Signed Read vs Transparent Read

| Aspect       | `.read` (Signed)                           | `.tread` (Transparent)    |
| ------------ | ------------------------------------------ | ------------------------- |
| Calldata     | Encrypted                                  | Plaintext                 |
| `msg.sender` | Signer's address                           | Zero address              |
| Use case     | Shielded data gated by `msg.sender`        | Public view functions     |
| Performance  | Slightly slower (sign + encrypt + decrypt) | Standard `eth_call` speed |

```typescript
// Signed read -- proves identity, encrypted calldata
const myBalance = await contract.read.balanceOf(["0x1234..."]);

// Transparent read -- no identity, plaintext calldata
const totalSupply = await contract.tread.totalSupply();
```

{% hint style="info" %}
Signed reads are critical for any contract that checks `msg.sender` in view functions to gate shielded data access. If you are unsure whether a view function depends on `msg.sender`, use `.read` -- it is always safe, just slightly slower than `.tread`.
{% endhint %}

## See Also

- [Contract Instance](contract-instance.md) -- `getShieldedContract` with `.read` and `.tread` namespaces
- [Shielded Writes](shielded-writes.md) -- Encrypted write transactions using the same pipeline
- [Encryption](encryption.md) -- ECDH key exchange and AES-GCM details
- [Shielded Wallet Client](shielded-wallet-client.md) -- Creating the client used for signed reads
