---
description: Transaction metadata used for AAD in AEAD encryption
icon: database
---

# TxSeismicMetadata

Complete transaction metadata used to construct the Additional Authenticated Data (AAD) for AES-GCM encryption. Ensures that encrypted calldata is cryptographically bound to the full transaction context.

## Overview

`TxSeismicMetadata` combines the sender address, standard EVM transaction fields, and Seismic-specific encryption parameters into a single struct. When serialized via `encode_as_aad()`, it produces the AAD bytes passed to AES-GCM during calldata encryption and decryption. This binding prevents an attacker from replaying encrypted calldata with different transaction parameters.

## Definition

```rust
pub struct TxSeismicMetadata {
    pub sender: Address,
    pub legacy_fields: TxLegacyFields,
    pub seismic_elements: TxSeismicElements,
}
```

## Fields

| Field              | Type                                          | Description                                                                    |
| ------------------ | --------------------------------------------- | ------------------------------------------------------------------------------ |
| `sender`           | `Address`                                     | The transaction sender's Ethereum address (20 bytes)                           |
| `legacy_fields`    | [`TxLegacyFields`](#txlegacyfields)           | Standard EVM fields: chain ID, nonce, recipient, value                         |
| `seismic_elements` | [`TxSeismicElements`](tx-seismic-elements.md) | Encryption metadata: public key, nonce, version, block hash, expiry, read flag |

## Methods

### AAD Encoding

| Method            | Signature                            | Description                                                               |
| ----------------- | ------------------------------------ | ------------------------------------------------------------------------- |
| `encode_as_aad()` | `fn encode_as_aad(&self) -> Vec<u8>` | RLP-encodes the metadata for use as AES-GCM Additional Authenticated Data |

The encoded AAD is an RLP list containing all fields in order:

```
AAD = RLP([
    sender,
    chain_id,
    nonce,
    to,
    value,
    encryption_pubkey,
    encryption_nonce,
    message_version,
    recent_block_hash,
    expires_at_block,
    signed_read,
])
```

### Encryption / Decryption

| Method                                              | Signature                                                                                               | Description                                                       |
| --------------------------------------------------- | ------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| `encrypt(sk, plaintext)`                            | `fn encrypt(&self, sk: &SecretKey, plaintext: &[u8]) -> Vec<u8>`                                        | Encrypt plaintext using this metadata as AAD context              |
| `decrypt(sk, ciphertext)`                           | `fn decrypt(&self, sk: &SecretKey, ciphertext: &[u8]) -> Vec<u8>`                                       | Decrypt ciphertext using this metadata as AAD context             |
| `client_encrypt(plaintext, network_pk, client_sk)`  | `fn client_encrypt(&self, plaintext: &[u8], network_pk: &PublicKey, client_sk: &SecretKey) -> Vec<u8>`  | Client-side encrypt with TEE's public key and client's secret key |
| `client_decrypt(ciphertext, network_pk, client_sk)` | `fn client_decrypt(&self, ciphertext: &[u8], network_pk: &PublicKey, client_sk: &SecretKey) -> Vec<u8>` | Client-side decrypt with TEE's public key and client's secret key |

## TxLegacyFields

Standard EVM transaction fields used as part of the AAD context.

### Definition

```rust
pub struct TxLegacyFields {
    pub chain_id: ChainId,
    pub nonce: u64,
    pub to: TxKind,
    pub value: U256,
}
```

### Fields

| Field      | Type              | Description                                                        |
| ---------- | ----------------- | ------------------------------------------------------------------ |
| `chain_id` | `ChainId` (`u64`) | Numeric chain identifier                                           |
| `nonce`    | `u64`             | Sender's transaction count                                         |
| `to`       | `TxKind`          | `TxKind::Call(Address)` for calls, `TxKind::Create` for deployment |
| `value`    | `U256`            | Amount of wei to transfer                                          |

{% hint style="info" %}
`TxLegacyFields` is a subset of the full transaction fields. It contains only the fields needed for AAD construction -- gas-related fields (`gas_price`, `gas_limit`) are intentionally excluded because they may change during filling without invalidating the encryption.
{% endhint %}

## Examples

### Building Metadata from a TxSeismic

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::Address;

let tx = TxSeismic { /* ... */ };
let sender: Address = "0xYOUR_ADDRESS".parse()?;

// Build metadata from transaction
let metadata = tx.tx_metadata(sender);

// Get AAD bytes for encryption
let aad = metadata.encode_as_aad();
println!("AAD length: {} bytes", aad.len());
```

### Manual Construction

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::{Address, U256, TxKind};

let metadata = TxSeismicMetadata {
    sender: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".parse()?,
    legacy_fields: TxLegacyFields {
        chain_id: 5124,
        nonce: 42,
        to: TxKind::Call("0x1234567890123456789012345678901234567890".parse()?),
        value: U256::from(1_000_000_000_000_000_000u128), // 1 ETH
    },
    seismic_elements: TxSeismicElements::default()
        .with_encryption_pubkey(tee_pubkey)
        .with_encryption_nonce(random_nonce)
        .with_message_version(2)
        .with_recent_block_hash(latest_hash)
        .with_expires_at_block(current_block + 100)
        .with_signed_read(false),
};
```

### Using Metadata for Client-Side Encryption

```rust
use seismic_alloy::prelude::*;

let metadata = tx.tx_metadata(sender);

// Client-side encryption using TEE's public key
let ciphertext = metadata.client_encrypt(
    &plaintext_calldata,
    &tee_public_key,
    &client_secret_key,
);

// Client-side decryption of TEE response
let plaintext = metadata.client_decrypt(
    &encrypted_response,
    &tee_public_key,
    &client_secret_key,
);
```

## Why AAD Matters

Additional Authenticated Data binds the ciphertext to the complete transaction context. Without AAD, an attacker could:

- **Replay encrypted calldata** with a different `to` address, redirecting the call
- **Change the `value`** being transferred while reusing the same encrypted function call
- **Modify expiry parameters** to extend the validity of a stale transaction
- **Alter the sender** to impersonate another account

With AAD binding:

- Ciphertext is valid **only** with the exact metadata used during encryption
- Any modification to sender, chain ID, nonce, recipient, value, or seismic elements causes decryption to fail
- The TEE rejects tampered transactions because AES-GCM authentication verification fails

## How Metadata Flows Through the Pipeline

```
1. Transaction builder creates SeismicTransactionRequest
2. Fillers populate nonce, chain ID, gas, and seismic elements
3. Provider builds TxSeismicMetadata from the filled transaction
4. metadata.encode_as_aad() produces AAD bytes
5. AES-GCM encrypts calldata with AAD
6. Encrypted calldata replaces plaintext in TxSeismic.input
7. Transaction is signed and broadcast
8. TEE reconstructs the same TxSeismicMetadata
9. TEE decrypts calldata, verifying AAD matches
```

## Notes

- **Automatically constructed** by the provider during the encryption step -- most users never interact with this type directly
- **Critical for security** -- the AAD binding is what prevents replay and parameter-tampering attacks
- **Gas fields excluded** -- `gas_price` and `gas_limit` are intentionally not part of the AAD, allowing gas estimation to change without invalidating the encryption
- **RLP-encoded** -- the `encode_as_aad()` method produces a deterministic RLP encoding that both client and TEE compute identically

## See Also

- [TxSeismic](tx-seismic.md) -- Transaction struct that provides `tx_metadata()` method
- [TxSeismicElements](tx-seismic-elements.md) -- Encryption metadata included in AAD
- [Encryption](../provider/encryption.md) -- Full encryption pipeline documentation
- [Shielded Calls](../contract-interaction/shielded-calls.md) -- How AAD is used in practice
