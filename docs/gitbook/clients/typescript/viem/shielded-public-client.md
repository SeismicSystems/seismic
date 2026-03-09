---
description: Read-only client with TEE key access and precompile support
icon: globe
---

# Shielded Public Client

Read-only client for interacting with a Seismic node. Extends viem's `PublicClient` with TEE public key retrieval, precompile access, block explorer URL helpers, SRC20 event watching, and deposit contract queries. Does **not** require a private key.

## Import

```typescript
import { createShieldedPublicClient } from "seismic-viem";
```

## Constructor

```typescript
const publicClient = createShieldedPublicClient({
  chain: seismicTestnet,
  transport: http(),
});
```

### Parameters

| Parameter   | Type        | Required | Description                                  |
| ----------- | ----------- | -------- | -------------------------------------------- |
| `chain`     | `Chain`     | Yes      | Chain configuration (e.g., `seismicTestnet`) |
| `transport` | `Transport` | Yes      | viem transport (e.g., `http()`)              |

### Return Type

`ShieldedPublicClient`

A viem `PublicClient` extended with `ShieldedPublicActions`, `DepositContractPublicActions`, and `SRC20PublicActions`.

## Usage

```typescript
import { createShieldedPublicClient } from "seismic-viem";
import { seismicTestnet } from "seismic-viem/chains";
import { http } from "viem";

const publicClient = createShieldedPublicClient({
  chain: seismicTestnet,
  transport: http(),
});

// Standard viem public actions work as usual
const blockNumber = await publicClient.getBlockNumber();
const balance = await publicClient.getBalance({
  address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
});
```

## Actions

### Shielded Public Actions

| Action                        | Return Type      | Description                                                            |
| ----------------------------- | ---------------- | ---------------------------------------------------------------------- |
| `getTeePublicKey()`           | `Promise<Hex>`   | Fetch the TEE's secp256k1 public key via `seismic_getTeePublicKey` RPC |
| `getStorageAt()`              | --               | Throws error (not supported on Seismic)                                |
| `publicRequest()`             | `Promise<any>`   | Raw RPC request to the node                                            |
| `explorerUrl(opts)`           | `string \| null` | Generate a block explorer URL                                          |
| `addressExplorerUrl(address)` | `string \| null` | Explorer URL for an address                                            |
| `blockExplorerUrl(block)`     | `string \| null` | Explorer URL for a block                                               |
| `txExplorerUrl(hash)`         | `string \| null` | Explorer URL for a transaction                                         |
| `tokenExplorerUrl(address)`   | `string \| null` | Explorer URL for a token                                               |

### Precompile Actions

| Action                       | Return Type          | Description                                    |
| ---------------------------- | -------------------- | ---------------------------------------------- |
| `rng(params)`                | `Promise<bigint>`    | Generate random numbers via the RNG precompile |
| `ecdh(params)`               | `Promise<Hex>`       | ECDH key exchange via precompile               |
| `aesGcmEncryption(params)`   | `Promise<Hex>`       | AES-GCM encrypt via precompile                 |
| `aesGcmDecryption(params)`   | `Promise<string>`    | AES-GCM decrypt via precompile                 |
| `hdfk(ikm)`                  | `Promise<Hex>`       | HKDF key derivation via precompile             |
| `secp256k1Signature(params)` | `Promise<Signature>` | secp256k1 signing via precompile               |

### SRC20 Actions

| Action                      | Return Type | Description                                  |
| --------------------------- | ----------- | -------------------------------------------- |
| `watchSRC20Events()`        | --          | Watch for SRC20 token events                 |
| `watchSRC20EventsWithKey()` | --          | Watch for SRC20 events with a decryption key |

### Deposit Contract Actions

| Action              | Return Type       | Description                   |
| ------------------- | ----------------- | ----------------------------- |
| `getDepositRoot()`  | `Promise<Hex>`    | Query the deposit merkle root |
| `getDepositCount()` | `Promise<bigint>` | Query the deposit count       |

### Standard viem Public Actions

All standard viem `PublicActions` are available directly on the client:

- `getBlockNumber()`, `getBlock()`, `getTransaction()`
- `getBalance()`, `getCode()`, `call()`
- `estimateGas()`, `waitForTransactionReceipt()`
- All other viem public client methods

## `getStorageAt` Disabled

{% hint style="warning" %}
Seismic does not support `eth_getStorageAt`. Calling `publicClient.getStorageAt()` will throw an error. This is by design -- storage slots on Seismic may contain shielded data and cannot be read directly via RPC.
{% endhint %}

## Examples

### Fetching the TEE Public Key

```typescript
import { createShieldedPublicClient } from "seismic-viem";
import { seismicTestnet } from "seismic-viem/chains";
import { http } from "viem";

const publicClient = createShieldedPublicClient({
  chain: seismicTestnet,
  transport: http(),
});

const teePublicKey = await publicClient.getTeePublicKey();
console.log("TEE public key:", teePublicKey);
```

### Using Precompiles

```typescript
import { createShieldedPublicClient } from "seismic-viem";
import { seismicTestnet } from "seismic-viem/chains";
import { http } from "viem";

const publicClient = createShieldedPublicClient({
  chain: seismicTestnet,
  transport: http(),
});

// Generate a random number
const randomValue = await publicClient.rng({ numWords: 1 });
console.log("Random value:", randomValue);

// ECDH key exchange
const sharedSecret = await publicClient.ecdh({
  privateKey: "0x...",
  publicKey: "0x...",
});

// AES-GCM encryption
const ciphertext = await publicClient.aesGcmEncryption({
  key: "0x...",
  plaintext: "0x...",
  nonce: "0x...",
});

// HKDF key derivation
const derivedKey = await publicClient.hdfk("0x...");
```

### Explorer URL Helpers

```typescript
const publicClient = createShieldedPublicClient({
  chain: seismicTestnet,
  transport: http(),
});

const txUrl = publicClient.txExplorerUrl("0xabc123...");
const addrUrl = publicClient.addressExplorerUrl("0x742d35Cc...");
const blockUrl = publicClient.blockExplorerUrl(12345n);

console.log("Transaction:", txUrl);
console.log("Address:", addrUrl);
console.log("Block:", blockUrl);
```

## See Also

- [Shielded Wallet Client](shielded-wallet-client.md) -- Full-featured client with encryption and shielded writes
- [Chains](chains.md) -- Chain configurations for Seismic networks
- [Precompiles](precompiles.md) -- Precompile details and parameters
- [Encryption](encryption.md) -- Encryption utilities and `getEncryption()`
