---
description: Full-featured client with encryption, shielded writes, and signed reads
icon: wallet
---

# Shielded Wallet Client

Full-featured client for Seismic that handles encryption, shielded writes, and signed reads. Extends viem's wallet client with an ECDH-derived AES encryption pipeline. On construction, it fetches the TEE public key from the node and derives a shared AES key for encrypting calldata.

## Import

```typescript
import { createShieldedWalletClient } from "seismic-viem";
```

## Constructor

```typescript
const walletClient = await createShieldedWalletClient({
  chain: seismicTestnet,
  transport: http(),
  account: privateKeyToAccount("0x..."),
});
```

{% hint style="info" %}
`createShieldedWalletClient` is **async** -- it fetches the TEE public key from the node during construction and derives the AES encryption key. Always `await` the result.
{% endhint %}

### Parameters

| Parameter      | Type                   | Required | Description                                                                      |
| -------------- | ---------------------- | -------- | -------------------------------------------------------------------------------- |
| `chain`        | `Chain`                | Yes      | Chain configuration (e.g., `seismicTestnet`)                                     |
| `transport`    | `Transport`            | Yes      | viem transport (e.g., `http()`)                                                  |
| `account`      | `Account`              | Yes      | viem account (from `privateKeyToAccount` or `custom`)                            |
| `encryptionSk` | `Hex`                  | No       | Custom encryption private key. If not provided, generates a random secp256k1 key |
| `publicClient` | `ShieldedPublicClient` | No       | Reuse an existing `ShieldedPublicClient` instead of creating a new one           |

### Return Type

`Promise<ShieldedWalletClient>`

A viem `Client` extended with `PublicActions`, `WalletActions`, `EncryptionActions`, `ShieldedPublicActions`, `ShieldedWalletActions`, `DepositContractPublicActions`, `DepositContractWalletActions`, and `SRC20WalletActions`.

## Usage

```typescript
import { createShieldedWalletClient } from "seismic-viem";
import { seismicTestnet } from "seismic-viem/chains";
import { http } from "viem";
import { privateKeyToAccount } from "viem/accounts";

const walletClient = await createShieldedWalletClient({
  chain: seismicTestnet,
  transport: http(),
  account: privateKeyToAccount("0x..."),
});

// Shielded write -- calldata is encrypted
const hash = await walletClient.writeContract({
  address: "0xContractAddress",
  abi: contractAbi,
  functionName: "transfer",
  args: ["0xRecipient", 1000n],
});

// Signed read -- authenticated eth_call
const balance = await walletClient.readContract({
  address: "0xContractAddress",
  abi: contractAbi,
  functionName: "balanceOf",
  args: ["0xMyAddress"],
});
```

## Initialization Lifecycle

When you call `createShieldedWalletClient()`, the following steps happen:

1. **Creates a `ShieldedPublicClient`** (or reuses the one provided via `publicClient`)
2. **Fetches the TEE public key** from the node via `seismic_getTeePublicKey` RPC
3. **Generates an ephemeral secp256k1 keypair** (or uses the provided `encryptionSk`)
4. **Derives an AES-256 key** via ECDH between the client's private key and the TEE's public key
5. **Composes all action layers** (`PublicActions`, `WalletActions`, `EncryptionActions`, `ShieldedPublicActions`, `ShieldedWalletActions`, `DepositContractPublicActions`, `DepositContractWalletActions`, `SRC20WalletActions`) onto a single viem client

## Actions

### Shielded Wallet Actions

| Action                            | Description                                                                             |
| --------------------------------- | --------------------------------------------------------------------------------------- |
| `writeContract(params)`           | Shielded write -- encrypts calldata with the AES key before sending                     |
| `twriteContract(params)`          | Transparent write -- standard viem `writeContract` (unencrypted calldata)               |
| `dwriteContract(params)`          | Debug write -- returns the plaintext tx, encrypted tx, and tx hash without broadcasting |
| `readContract(params)`            | Signed read -- authenticated `eth_call` that proves the caller's identity               |
| `treadContract(params)`           | Transparent read -- standard viem `readContract` (unsigned call)                        |
| `signedCall(params)`              | Low-level signed `eth_call`                                                             |
| `sendShieldedTransaction(params)` | Low-level shielded transaction send                                                     |

### Encryption Actions

| Action                          | Description                                                                     |
| ------------------------------- | ------------------------------------------------------------------------------- |
| `getEncryption()`               | Returns the AES encryption key used for shielded operations                     |
| `getEncryptionPublicKey()`      | Returns the client's encryption public key (the ephemeral secp256k1 public key) |
| `encrypt(plaintext, metadata)`  | Manual AES-GCM encryption using the derived key                                 |
| `decrypt(ciphertext, metadata)` | Manual AES-GCM decryption using the derived key                                 |

### Inherited Actions

The wallet client also includes all actions from `ShieldedPublicClient`:

- **Shielded public actions** -- `getTeePublicKey()`, `explorerUrl()`, etc.
- **Precompile actions** -- `rng()`, `ecdh()`, `aesGcmEncryption()`, `aesGcmDecryption()`, `hdfk()`, `secp256k1Signature()`
- **SRC20 actions** -- `watchSRC20Events()`, `watchSRC20EventsWithKey()`
- **Deposit contract actions** -- `getDepositRoot()`, `getDepositCount()`
- **Standard viem public actions** -- `getBlockNumber()`, `getBalance()`, `getBlock()`, etc.
- **Standard viem wallet actions** -- `sendTransaction()`, `signMessage()`, `signTypedData()`, etc.

{% hint style="info" %}
The wallet client automatically includes all public client actions -- you can call `getBlockNumber()`, `getBalance()`, etc. directly on it.
{% endhint %}

## Examples

### Shielded Write

```typescript
import { createShieldedWalletClient } from "seismic-viem";
import { seismicTestnet } from "seismic-viem/chains";
import { http } from "viem";
import { privateKeyToAccount } from "viem/accounts";

const walletClient = await createShieldedWalletClient({
  chain: seismicTestnet,
  transport: http(),
  account: privateKeyToAccount("0x..."),
});

// Shielded write: calldata is encrypted before submission
const hash = await walletClient.writeContract({
  address: "0xContractAddress",
  abi: contractAbi,
  functionName: "transfer",
  args: ["0xRecipient", 1000n],
});

const receipt = await walletClient.waitForTransactionReceipt({ hash });
console.log("Transaction confirmed in block:", receipt.blockNumber);
```

### Signed Read

```typescript
// Signed read: proves caller identity to the node
const balance = await walletClient.readContract({
  address: "0xContractAddress",
  abi: contractAbi,
  functionName: "balanceOf",
  args: ["0xMyAddress"],
});

console.log("Shielded balance:", balance);

// Transparent read: standard unsigned eth_call (no caller proof)
const totalSupply = await walletClient.treadContract({
  address: "0xContractAddress",
  abi: contractAbi,
  functionName: "totalSupply",
});

console.log("Total supply:", totalSupply);
```

### Reusing a Public Client

```typescript
import {
  createShieldedPublicClient,
  createShieldedWalletClient,
} from "seismic-viem";
import { seismicTestnet } from "seismic-viem/chains";
import { http } from "viem";
import { privateKeyToAccount } from "viem/accounts";

// Create a shared public client
const publicClient = createShieldedPublicClient({
  chain: seismicTestnet,
  transport: http(),
});

// Reuse it across multiple wallet clients
const walletClient1 = await createShieldedWalletClient({
  chain: seismicTestnet,
  transport: http(),
  account: privateKeyToAccount("0xPrivateKey1"),
  publicClient,
});

const walletClient2 = await createShieldedWalletClient({
  chain: seismicTestnet,
  transport: http(),
  account: privateKeyToAccount("0xPrivateKey2"),
  publicClient,
});
```

### Debug Write

```typescript
// Debug write: inspect the transaction without broadcasting
const debugResult = await walletClient.dwriteContract({
  address: "0xContractAddress",
  abi: contractAbi,
  functionName: "transfer",
  args: ["0xRecipient", 1000n],
});

console.log("Plaintext tx:", debugResult.plaintextTx);
console.log("Shielded tx:", debugResult.shieldedTx);
console.log("Tx hash:", debugResult.txHash);
```

### Custom Encryption Key

```typescript
const walletClient = await createShieldedWalletClient({
  chain: seismicTestnet,
  transport: http(),
  account: privateKeyToAccount("0x..."),
  encryptionSk: "0xCustomEncryptionPrivateKey",
});

// Access encryption details
const aesKey = walletClient.getEncryption();
const encPubKey = walletClient.getEncryptionPublicKey();
```

## `getEncryption()` Standalone Function

The encryption derivation logic is also available as a standalone function, separate from any client:

```typescript
import { getEncryption } from "seismic-viem";

const encryption = getEncryption(teePublicKey, clientPrivateKey);
// encryption.aesKey         -- the derived AES-256 key
// encryption.encryptionPrivateKey  -- the client's secp256k1 private key
// encryption.encryptionPublicKey   -- the client's secp256k1 public key
```

| Parameter   | Type  | Required | Description                                                              |
| ----------- | ----- | -------- | ------------------------------------------------------------------------ |
| `networkPk` | `Hex` | Yes      | The TEE's secp256k1 public key                                           |
| `clientSk`  | `Hex` | No       | Client's encryption private key. If not provided, generates a random one |

Returns `{ aesKey: Hex, encryptionPrivateKey: Hex, encryptionPublicKey: Hex }`.

This is useful when you need the encryption key material without constructing a full wallet client -- for example, to manually encrypt or decrypt data outside of the client's lifecycle.

## See Also

- [Shielded Public Client](shielded-public-client.md) -- Read-only client without private key
- [Encryption](encryption.md) -- Encryption utilities and `getEncryption()`
- [Chains](chains.md) -- Chain configurations for Seismic networks
- [Precompiles](precompiles.md) -- Precompile details and parameters
- [Contract Instance](contract-instance.md) -- Working with contract wrappers
