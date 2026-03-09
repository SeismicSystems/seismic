---
description: Encryption metadata and security parameters for Seismic transactions
icon: puzzle-piece
---

# TxSeismicElements

Encryption metadata attached to every `TxSeismic` transaction. Contains the public key, nonce, signing mode, freshness proof, expiry, and read/write flag required for Seismic's AEAD encryption scheme.

## Overview

`TxSeismicElements` carries all encryption and security parameters needed for a Seismic transaction. These fields enable ECDH key derivation, AES-GCM encryption, anti-replay protection, and transaction expiration. The filler pipeline (`SeismicElementsFiller`) populates these fields automatically when you use `.seismic()` on a transaction builder.

## Definition

```rust
pub struct TxSeismicElements {
    pub encryption_pubkey: PublicKey,     // secp256k1 compressed pubkey (33 bytes)
    pub encryption_nonce: U96,           // 12-byte nonce for AES-GCM
    pub message_version: u8,             // 0=RLP, >=2=EIP-712
    pub recent_block_hash: B256,         // anti-replay
    pub expires_at_block: u64,           // expiration block
    pub signed_read: bool,               // true for signed reads
}
```

## Fields

| Field               | Type        | Description                                                                                    |
| ------------------- | ----------- | ---------------------------------------------------------------------------------------------- |
| `encryption_pubkey` | `PublicKey` | Compressed secp256k1 public key (33 bytes) used for ECDH shared secret derivation with the TEE |
| `encryption_nonce`  | `U96`       | 12-byte random nonce for AES-GCM encryption. Must be unique per transaction                    |
| `message_version`   | `u8`        | Signing mode: `0` for raw RLP hash signing, `2` or higher for EIP-712 typed data signing       |
| `recent_block_hash` | `B256`      | Hash of a recent block, proving the transaction was created recently (freshness proof)         |
| `expires_at_block`  | `u64`       | Block number after which the transaction is invalid. Prevents delayed execution                |
| `signed_read`       | `bool`      | `true` for signed `eth_call` reads (no state change), `false` for write transactions           |

## Default Values

```rust
impl Default for TxSeismicElements {
    fn default() -> Self {
        Self {
            encryption_pubkey: /* identity point */,
            encryption_nonce: U96::ZERO,
            message_version: 0,
            recent_block_hash: B256::ZERO,
            expires_at_block: 0,
            signed_read: false,
        }
    }
}
```

{% hint style="info" %}
Default values are placeholders. The `SeismicElementsFiller` replaces them with real values (TEE public key, random nonce, latest block hash, current block + window) before the transaction is sent.
{% endhint %}

## Builder Methods

`TxSeismicElements` provides a builder pattern for constructing instances:

| Method                     | Signature                                                    | Description                             |
| -------------------------- | ------------------------------------------------------------ | --------------------------------------- |
| `with_encryption_pubkey()` | `fn with_encryption_pubkey(self, pubkey: PublicKey) -> Self` | Set the encryption public key           |
| `with_encryption_nonce()`  | `fn with_encryption_nonce(self, nonce: U96) -> Self`         | Set the AES-GCM nonce                   |
| `with_message_version()`   | `fn with_message_version(self, version: u8) -> Self`         | Set the signing mode (0=RLP, 2=EIP-712) |
| `with_recent_block_hash()` | `fn with_recent_block_hash(self, hash: B256) -> Self`        | Set the freshness block hash            |
| `with_expires_at_block()`  | `fn with_expires_at_block(self, block: u64) -> Self`         | Set the expiration block number         |
| `with_signed_read()`       | `fn with_signed_read(self, signed_read: bool) -> Self`       | Set the read/write flag                 |

### Example

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::{B256, U96};

let elements = TxSeismicElements::default()
    .with_encryption_pubkey(tee_pubkey)
    .with_encryption_nonce(U96::from(random_nonce))
    .with_message_version(2)
    .with_recent_block_hash(latest_block_hash)
    .with_expires_at_block(current_block + 100)
    .with_signed_read(false);
```

## Static Helpers

| Function                        | Signature                                                    | Description                                        |
| ------------------------------- | ------------------------------------------------------------ | -------------------------------------------------- |
| `get_rand_encryption_keypair()` | `fn get_rand_encryption_keypair() -> (SecretKey, PublicKey)` | Generate a random secp256k1 keypair for encryption |
| `get_rand_encryption_nonce()`   | `fn get_rand_encryption_nonce() -> U96`                      | Generate a random 12-byte AES-GCM nonce            |

### Example

```rust
use seismic_alloy::prelude::*;

// Generate random encryption keypair
let (secret_key, public_key) = TxSeismicElements::get_rand_encryption_keypair();

// Generate random nonce
let nonce = TxSeismicElements::get_rand_encryption_nonce();
```

## Crypto Methods

`TxSeismicElements` provides methods for encrypting and decrypting calldata using AEAD (AES-GCM with additional authenticated data).

### Low-Level Encryption

| Method                              | Signature                                                                                       | Description                                                   |
| ----------------------------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `encrypt(sk, plaintext, metadata)`  | `fn encrypt(&self, sk: &SecretKey, plaintext: &[u8], metadata: &TxSeismicMetadata) -> Vec<u8>`  | Encrypt plaintext using ECDH-derived key and metadata as AAD  |
| `decrypt(sk, ciphertext, metadata)` | `fn decrypt(&self, sk: &SecretKey, ciphertext: &[u8], metadata: &TxSeismicMetadata) -> Vec<u8>` | Decrypt ciphertext using ECDH-derived key and metadata as AAD |

### Client-Side Encryption

| Method                                                        | Signature                                                                                                                             | Description                                        |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------- |
| `client_encrypt(plaintext, network_pk, client_sk, metadata)`  | `fn client_encrypt(&self, plaintext: &[u8], network_pk: &PublicKey, client_sk: &SecretKey, metadata: &TxSeismicMetadata) -> Vec<u8>`  | Client-side encrypt using network (TEE) public key |
| `client_decrypt(ciphertext, network_pk, client_sk, metadata)` | `fn client_decrypt(&self, ciphertext: &[u8], network_pk: &PublicKey, client_sk: &SecretKey, metadata: &TxSeismicMetadata) -> Vec<u8>` | Client-side decrypt using network (TEE) public key |

### Encryption Flow

```
Client                          TEE (Node)
  |                               |
  |  1. ECDH(client_sk, tee_pk)  |
  |  -> shared_secret             |
  |                               |
  |  2. AES-GCM encrypt:         |
  |     key = shared_secret       |
  |     nonce = encryption_nonce  |
  |     aad = TxSeismicMetadata   |
  |     -> ciphertext             |
  |                               |
  |  3. Send TxSeismic ---------->|
  |                               |
  |     4. ECDH(tee_sk, client_pk)|
  |     -> same shared_secret     |
  |                               |
  |     5. AES-GCM decrypt        |
  |     -> plaintext calldata     |
```

## Field Details

### encryption_pubkey

The compressed secp256k1 public key (33 bytes) used in the ECDH key exchange. For client-initiated transactions, this is the **client's ephemeral public key** -- the TEE uses its own private key combined with this public key to derive the shared encryption secret.

### encryption_nonce

A 12-byte (96-bit) random value used as the nonce for AES-GCM encryption. Must be unique for every transaction to prevent nonce reuse attacks. Generated automatically by the filler pipeline via `get_rand_encryption_nonce()`.

### message_version

Controls the signing method:

- **`0`** -- Raw signing: the signing hash is computed from the RLP encoding of the unsigned transaction
- **`2` or higher** -- EIP-712 signing: the signing hash is computed from structured typed data

The SDK defaults to EIP-712 (`message_version = 2`) for better wallet compatibility.

### recent_block_hash

Hash of a recent block from the canonical chain. The node verifies this hash exists in its chain, ensuring the transaction was created recently and is not a stale replay. Auto-populated by `SeismicElementsFiller`.

### expires_at_block

The block number after which the transaction is rejected by the node. Typically set to `current_block + 100` (about 200 seconds on a 2-second block time). Prevents transactions from being held and executed much later.

### signed_read

- **`false`** -- This is a write transaction that will be broadcast and modify state
- **`true`** -- This is a signed `eth_call` for reading private state without state modification

The filler pipeline sets this based on whether you call `send_transaction()` or `seismic_call()`.

## Examples

### Automatic Construction (Typical)

```rust
use seismic_alloy::prelude::*;

// The filler pipeline handles TxSeismicElements construction
let tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
    .with_input(calldata.into())
    .with_kind(TxKind::Call(contract_address))
    .into()
    .seismic();  // Elements will be filled automatically

provider.send_transaction(tx.into()).await?;
```

### Manual Construction (Advanced)

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::{B256, U96};

let (client_sk, client_pk) = TxSeismicElements::get_rand_encryption_keypair();
let nonce = TxSeismicElements::get_rand_encryption_nonce();

let elements = TxSeismicElements {
    encryption_pubkey: client_pk,
    encryption_nonce: nonce,
    message_version: 2,
    recent_block_hash: B256::from_slice(&latest_hash),
    expires_at_block: current_block + 100,
    signed_read: false,
};
```

## Notes

- **Auto-populated by fillers.** In normal usage, you never construct `TxSeismicElements` manually -- the `SeismicElementsFiller` fills all fields from the network state.
- **Nonce uniqueness is critical.** Reusing an AES-GCM nonce with the same key breaks confidentiality. The random nonce generator provides sufficient entropy.
- **Part of AAD.** All fields in `TxSeismicElements` are included in the Additional Authenticated Data for AES-GCM, binding the ciphertext to these parameters.
- **RLP-encoded.** Elements are included in the RLP encoding of `TxSeismic` for hashing and signing.

## See Also

- [TxSeismic](tx-seismic.md) -- Parent struct containing `TxSeismicElements`
- [TxSeismicMetadata](tx-seismic-metadata.md) -- Uses elements for AAD construction
- [Encryption](../provider/encryption.md) -- Full encryption pipeline documentation
- [Shielded Calls](../contract-interaction/shielded-calls.md) -- How encryption is used in practice
