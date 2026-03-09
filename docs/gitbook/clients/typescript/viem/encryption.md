---
description: ECDH key exchange, AES-GCM calldata encryption, and the Seismic encryption pipeline
icon: lock
---

# Encryption

seismic-viem handles encryption transparently when you call `writeContract()` or `signedCall()` on a `ShieldedWalletClient`. The client derives a shared AES key during construction and encrypts calldata on every shielded operation automatically. This page documents the underlying encryption pipeline for advanced use cases where you need direct access to the cryptographic primitives.

## Encryption Flow

Every shielded transaction follows this pipeline:

```
1. Client generates an ephemeral secp256k1 keypair (or uses a provided one)
2. Client fetches the TEE public key from the node via seismic_getTeePublicKey
3. ECDH(client_sk, tee_pk) → shared secret
4. Shared secret → AES-256 key (via key derivation)
5. For each transaction:
   a. Generate a random 12-byte nonce
   b. Encode TxSeismicMetadata as Additional Authenticated Data (AAD)
   c. AES-GCM encrypt(plaintext_calldata, nonce, aad) → ciphertext
   d. Include encryptionPubkey + nonce in the transaction's SeismicTxExtras fields
```

{% hint style="info" %}
You don't need to call `getEncryption()` manually -- `createShieldedWalletClient` handles key exchange automatically. The functions on this page are for advanced use cases like offline encryption, custom key management, or debugging.
{% endhint %}

## `getEncryption(networkPk, clientSk?)`

Standalone function that performs ECDH key exchange and derives an AES-256 key from a TEE public key and an optional client secret key.

### Import

```typescript
import { getEncryption } from "seismic-viem";
```

### Parameters

| Parameter   | Type     | Required | Description                                                             |
| ----------- | -------- | -------- | ----------------------------------------------------------------------- |
| `networkPk` | `string` | Yes      | TEE's secp256k1 public key (fetched via `client.getTeePublicKey()`)     |
| `clientSk`  | `Hex`    | No       | Client's encryption private key. If omitted, generates a random keypair |

### Returns

| Property               | Type  | Description                                 |
| ---------------------- | ----- | ------------------------------------------- |
| `aesKey`               | `Hex` | Derived AES-256 key for encrypting calldata |
| `encryptionPrivateKey` | `Hex` | Client's ephemeral secp256k1 private key    |
| `encryptionPublicKey`  | `Hex` | Client's compressed secp256k1 public key    |

### Example

```typescript
import { createShieldedPublicClient, getEncryption } from "seismic-viem";
import { seismicTestnet } from "seismic-viem/chains";
import { http } from "viem";

// Fetch the TEE public key from the node
const publicClient = createShieldedPublicClient({
  chain: seismicTestnet,
  transport: http(),
});

const teePublicKey = await publicClient.getTeePublicKey();

// Derive encryption keys without constructing a full wallet client
const encryption = getEncryption(teePublicKey);
console.log("AES key:", encryption.aesKey);
console.log("Encryption public key:", encryption.encryptionPublicKey);
console.log("Encryption private key:", encryption.encryptionPrivateKey);

// Or provide your own private key for deterministic key derivation
const deterministicEncryption = getEncryption(
  teePublicKey,
  "0xYourSecp256k1PrivateKey",
);
```

## Encryption Actions

The `ShieldedWalletClient` exposes encryption operations as client methods. These use the AES key that was derived during client construction.

| Action                          | Return Type    | Description                                                     |
| ------------------------------- | -------------- | --------------------------------------------------------------- |
| `getEncryption()`               | `Hex`          | Returns the AES-256 key used for shielded operations            |
| `getEncryptionPublicKey()`      | `Hex`          | Returns the client's compressed secp256k1 encryption public key |
| `encrypt(plaintext, metadata)`  | `Promise<Hex>` | AES-GCM encrypt plaintext calldata with metadata as AAD         |
| `decrypt(ciphertext, metadata)` | `Promise<Hex>` | AES-GCM decrypt ciphertext using metadata as AAD                |

### Example

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

// Access the derived AES key
const aesKey = walletClient.getEncryption();
console.log("AES key:", aesKey);

// Access the encryption public key
const encPubKey = walletClient.getEncryptionPublicKey();
console.log("Encryption public key:", encPubKey);
```

## SeismicTxExtras

Every Seismic transaction includes additional fields that carry encryption metadata alongside the standard Ethereum transaction fields.

| Field              | Type      | Description                                               |
| ------------------ | --------- | --------------------------------------------------------- |
| `encryptionPubkey` | `Hex`     | Client's compressed secp256k1 public key for this session |
| `encryptionNonce`  | `Hex`     | Random 12-byte AES-GCM nonce                              |
| `messageVersion`   | `number`  | `0` for normal transactions, `2` for EIP-712 signed       |
| `recentBlockHash`  | `Hex`     | Recent block hash used for replay protection              |
| `expiresAtBlock`   | `bigint`  | Block number after which the transaction expires          |
| `signedRead`       | `boolean` | `true` for signed reads, `false` for standard writes      |

These fields are populated automatically by `writeContract()` and `signedCall()`. The node uses the `encryptionPubkey` to reconstruct the shared secret and decrypt the calldata inside the TEE.

## AES-GCM with AEAD

Seismic uses AES-GCM with Additional Authenticated Data (AEAD) to bind encrypted calldata to the transaction context. The `TxSeismicMetadata` is encoded and passed as AAD during encryption, which means:

- The ciphertext can only be decrypted when presented alongside the correct metadata.
- Any modification to the transaction's metadata (account, nonce, recipient, value, or SeismicTxExtras) causes decryption to fail.
- This prevents replay attacks and ensures calldata cannot be reattached to a different transaction context.

`TxSeismicMetadata` includes the following fields as AAD:

- `account` -- sender address
- `nonce` -- transaction nonce
- `to` -- recipient address
- `value` -- ETH value
- `seismicElements` -- the `encryptionPubkey`, `encryptionNonce`, `messageVersion`, `recentBlockHash`, `expiresAtBlock`, and `signedRead` fields

## Crypto Dependencies

seismic-viem uses the following `@noble` libraries for client-side cryptography. These are bundled as direct dependencies and do not need to be installed separately.

| Package          | Purpose                                 |
| ---------------- | --------------------------------------- |
| `@noble/curves`  | secp256k1 keypair generation and ECDH   |
| `@noble/ciphers` | AES-GCM encryption and decryption       |
| `@noble/hashes`  | SHA-256, HKDF, and other hash functions |

{% hint style="info" %}
All cryptographic operations happen client-side before the transaction is submitted. The plaintext calldata never leaves the client -- only the AES-GCM ciphertext is sent to the node, where it is decrypted inside the TEE for execution.
{% endhint %}

## See Also

- [Shielded Wallet Client](shielded-wallet-client.md) -- Client that performs encryption automatically
- [Shielded Writes](shielded-writes.md) -- How encrypted transactions are built and sent
- [Signed Reads](signed-reads.md) -- Authenticated reads that prove caller identity
- [Precompiles](precompiles.md) -- On-chain cryptographic operations via precompiled contracts
- [Installation](installation.md) -- Bundled crypto dependencies
