---
description: Authenticated read calls that prove caller identity
icon: signature
---

# Signed Reads

On standard Ethereum, `eth_call` can spoof any `from` address -- there is no signature check. Seismic prevents this: all unsigned `eth_call` requests have `msg.sender` set to the zero address. To read shielded data that depends on `msg.sender`, you need a **signed read**: a signed transaction submitted to `eth_call` that proves the caller's identity.

## Why Signed Reads Matter

Contracts can use `msg.sender` in view functions to gate access to shielded data. A common example: a token contract with a `balanceOf()` that takes no arguments and uses `msg.sender` internally to look up the caller's balance. Without a signed read, the contract sees the zero address and returns that address's balance -- which is almost certainly zero.

```typescript
// Smart read -- balanceOf(saddress) has shielded param → signed read automatically
const myBalance = await contract.read.balanceOf(["0x1234..."]);

// Smart read -- totalSupply() has no shielded params → transparent read automatically
const totalSupply = await contract.read.totalSupply();

// Force signed read -- always encrypted, even for non-shielded functions
const supply = await contract.sread.totalSupply();
```

seismic-viem provides several approaches:

- **`contract.read.functionName()`** -- smart routing via [getShieldedContract](contract-instance.md). Auto-detects shielded parameters and uses signed read only when needed.
- **`contract.sread.functionName()`** -- force signed read via [getShieldedContract](contract-instance.md). Always uses signed read regardless of parameter types.
- **`signedReadContract()`** -- standalone function, same API shape as viem's `readContract`. Always uses signed read.
- **`walletClient.readContract()`** -- smart routing via the wallet client. Same auto-detection as `contract.read`.
- **`walletClient.sreadContract()`** -- force signed read via the wallet client. Always uses signed read.

The signed read paths all encrypt the calldata, sign it, and decrypt the response automatically.

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

When you call `signedReadContract` (or `contract.sread.functionName`), the SDK performs the following steps:

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

| Aspect       | `.read` (Smart)                            | `.sread` (Force Signed)                    | `.tread` (Transparent)    |
| ------------ | ------------------------------------------ | ------------------------------------------ | ------------------------- |
| Calldata     | Auto-detected by ABI                       | Always encrypted                           | Always plaintext          |
| `msg.sender` | Signer if shielded, zero if not            | Always signer's address                    | Always zero address       |
| Use case     | Default -- handles most cases              | Force encryption for sensitive returns     | Public view functions     |
| Performance  | Optimal -- encrypts only when needed       | Slightly slower (sign + encrypt + decrypt) | Standard `eth_call` speed |

```typescript
// Smart read -- auto-detects: balanceOf(saddress) → signed, totalSupply() → transparent
const myBalance = await contract.read.balanceOf(["0x1234..."]);
const totalSupply = await contract.read.totalSupply();

// Force signed read -- always encrypted
const supply = await contract.sread.totalSupply();

// Force transparent read -- always plaintext
const supply2 = await contract.tread.totalSupply();
```

{% hint style="info" %}
The smart `.read` namespace handles most cases correctly by inspecting the ABI. Use `.sread` when you need the response encrypted even though the function has no shielded input parameters. Use `.tread` only for public data where you don't need authentication.
{% endhint %}

{% hint style="warning" %}
`.tread` and `walletClient.treadContract` reject the `account` option and will throw. On Seismic, transparent `eth_call` zeroes out `from` on the node, so any `account` you pass would silently be ignored. For any sender-aware read, use `.sread` / `sreadContract`.
{% endhint %}

## See Also

- [Contract Instance](contract-instance.md) -- `getShieldedContract` with `.read` and `.tread` namespaces
- [Shielded Writes](shielded-writes.md) -- Encrypted write transactions using the same pipeline
- [Encryption](encryption.md) -- ECDH key exchange and AES-GCM details
- [Shielded Wallet Client](shielded-wallet-client.md) -- Creating the client used for signed reads
