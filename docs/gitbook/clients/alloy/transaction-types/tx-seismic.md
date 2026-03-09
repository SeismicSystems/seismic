---
description: Core Seismic transaction type (0x4A)
icon: lock
---

# TxSeismic

The core Seismic transaction type, identified by type code `0x4A`. Extends a legacy-style Ethereum transaction with encryption metadata for privacy-preserving contract interactions.

## Overview

`TxSeismic` represents a complete Seismic transaction with encrypted calldata. It contains standard EVM fields (chain ID, nonce, gas, recipient, value, input) plus a `TxSeismicElements` struct carrying all encryption and expiry parameters. The `input` field holds **encrypted** calldata after the filler pipeline processes it.

## Definition

```rust
pub struct TxSeismic {
    pub chain_id: ChainId,
    pub nonce: u64,
    pub gas_price: u128,
    pub gas_limit: u64,
    pub to: TxKind,
    pub value: U256,
    pub input: Bytes,
    pub seismic_elements: TxSeismicElements,
}
```

## Type Constant

```rust
pub const TX_TYPE: u8 = 0x4A;
```

## Fields

| Field              | Type                                          | Description                                                        |
| ------------------ | --------------------------------------------- | ------------------------------------------------------------------ |
| `chain_id`         | `ChainId` (`u64`)                             | Numeric chain identifier (e.g., 5124 for Seismic testnet)          |
| `nonce`            | `u64`                                         | Sender's transaction count                                         |
| `gas_price`        | `u128`                                        | Gas price in wei                                                   |
| `gas_limit`        | `u64`                                         | Maximum gas allowed for execution                                  |
| `to`               | `TxKind`                                      | `TxKind::Call(Address)` for calls, `TxKind::Create` for deployment |
| `value`            | `U256`                                        | Amount of wei to transfer                                          |
| `input`            | `Bytes`                                       | Calldata -- encrypted after filler processing                      |
| `seismic_elements` | [`TxSeismicElements`](tx-seismic-elements.md) | Encryption metadata, block hash, expiry, and read flag             |

## Trait Implementations

`TxSeismic` implements the following Alloy traits:

| Trait                     | Description                                                 |
| ------------------------- | ----------------------------------------------------------- |
| `Transaction`             | Standard transaction interface (chain_id, nonce, gas, etc.) |
| `SignableTransaction`     | Provides signing hash and signature methods                 |
| `Typed2718`               | EIP-2718 typed transaction encoding (type `0x4A`)           |
| `InputDecryptionElements` | Access decryption metadata for the TEE                      |
| `Encodable` / `Decodable` | RLP encoding and decoding                                   |

## Methods

### EIP-712 Support

| Method                  | Signature                                    | Description                                           |
| ----------------------- | -------------------------------------------- | ----------------------------------------------------- |
| `is_eip712()`           | `fn is_eip712(&self) -> bool`                | Returns `true` if `message_version >= 2`              |
| `eip712_to_type_data()` | `fn eip712_to_type_data(&self) -> TypedData` | Converts to EIP-712 typed data for structured signing |

### Conversion

| Method            | Signature                                   | Description                                             |
| ----------------- | ------------------------------------------- | ------------------------------------------------------- |
| `to_legacy_tx()`  | `fn to_legacy_tx(&self) -> TxLegacy`        | Converts to a standard `TxLegacy` for gas estimation    |
| `legacy_fields()` | `fn legacy_fields(&self) -> TxLegacyFields` | Extracts the legacy subset (chain_id, nonce, to, value) |

### Metadata

| Method                | Signature                                                     | Description                                          |
| --------------------- | ------------------------------------------------------------- | ---------------------------------------------------- |
| `tx_metadata(sender)` | `fn tx_metadata(&self, sender: Address) -> TxSeismicMetadata` | Builds the full metadata struct for AAD construction |

### Encryption

| Method                                       | Signature                                                                                     | Description                                                   |
| -------------------------------------------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `encrypt_input_aead(sk, plaintext, sender)`  | `fn encrypt_input_aead(&self, sk: &SecretKey, plaintext: &[u8], sender: Address) -> Vec<u8>`  | Encrypts calldata using AEAD with transaction metadata as AAD |
| `decrypt_input_aead(sk, ciphertext, sender)` | `fn decrypt_input_aead(&self, sk: &SecretKey, ciphertext: &[u8], sender: Address) -> Vec<u8>` | Decrypts calldata using AEAD with transaction metadata as AAD |

### Validation

| Method                                         | Signature                                                                      | Description                                                         |
| ---------------------------------------------- | ------------------------------------------------------------------------------ | ------------------------------------------------------------------- |
| `validate_recent_block_hash(recent_blocks)`    | `fn validate_recent_block_hash(&self, recent_blocks: &[B256]) -> bool`         | Verifies that `recent_block_hash` exists in the provided block list |
| `validate_expiration(current_block)`           | `fn validate_expiration(&self, current_block: u64) -> bool`                    | Checks that the transaction has not expired                         |
| `validate_block(current_block, recent_blocks)` | `fn validate_block(&self, current_block: u64, recent_blocks: &[B256]) -> bool` | Combined freshness and expiration validation                        |

## Examples

### Constructing a TxSeismic

In practice, you rarely construct `TxSeismic` directly. The filler pipeline builds it from a `SeismicTransactionRequest`. However, for reference:

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::{Bytes, U256, TxKind, Address, B256};

let tx = TxSeismic {
    chain_id: 5124,
    nonce: 0,
    gas_price: 20_000_000_000, // 20 gwei
    gas_limit: 100_000,
    to: TxKind::Call("0x1234567890123456789012345678901234567890".parse()?),
    value: U256::ZERO,
    input: Bytes::from(encrypted_calldata),
    seismic_elements: TxSeismicElements {
        encryption_pubkey: tee_pubkey,
        encryption_nonce: U96::from(rand_nonce),
        message_version: 2, // EIP-712
        recent_block_hash: latest_block_hash,
        expires_at_block: current_block + 100,
        signed_read: false,
    },
};
```

### Using the Builder Pattern (Recommended)

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::{U256, TxKind};
use alloy::sol_types::SolCall;

sol! {
    interface IMyContract {
        function setValue(suint256 x) public;
    }
}

let calldata = IMyContract::setValueCall {
    x: U256::from(42),
}.abi_encode();

// Build and mark as seismic
let tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
    .with_input(calldata.into())
    .with_kind(TxKind::Call(contract_address))
    .into()
    .seismic();

// The filler pipeline converts this into a TxSeismic internally
let pending = provider.send_transaction(tx.into()).await?;
```

### Converting to Legacy for Gas Estimation

```rust
let seismic_tx = TxSeismic { /* ... */ };

// Convert to legacy for gas estimation
let legacy_tx = seismic_tx.to_legacy_tx();
// legacy_tx can be used with standard gas estimation RPCs
```

### EIP-712 Typed Data

```rust
let tx = TxSeismic { /* ... */ };

if tx.is_eip712() {
    let typed_data = tx.eip712_to_type_data();
    // Use typed_data for EIP-712 signing workflows
}
```

### Accessing Metadata for AAD

```rust
let tx = TxSeismic { /* ... */ };
let sender: Address = "0xSENDER...".parse()?;

let metadata = tx.tx_metadata(sender);
let aad_bytes = metadata.encode_as_aad();
// aad_bytes is used as Additional Authenticated Data in AES-GCM
```

## RLP Encoding

`TxSeismic` uses RLP encoding for both hashing and network serialization. The encoding order is:

```
RLP([
    chain_id,
    nonce,
    gas_price,
    gas_limit,
    to,
    value,
    input,
    encryption_pubkey,
    encryption_nonce,
    message_version,
    recent_block_hash,
    expires_at_block,
    signed_read,
])
```

The type prefix `0x4A` is prepended for EIP-2718 envelope encoding.

## EIP-712 Signing

When `message_version >= 2`, the transaction uses EIP-712 typed data signing instead of raw RLP hash signing. This provides:

- **Structured data** -- Wallets can display human-readable transaction details
- **Domain separation** -- Signatures are bound to a specific chain and contract
- **Better UX** -- Hardware wallets and browser extensions can show meaningful information

## Notes

- **Type 0x4A** is the Seismic-specific transaction type, distinct from all standard Ethereum types
- **`input` contains encrypted data** after the filler pipeline processes the transaction
- **Cannot be used for Create transactions** -- `to` must be `TxKind::Call(address)` for seismic transactions
- **`to_legacy_tx()`** is used internally for gas estimation, since the RPC expects legacy format
- **Validation methods** are primarily used by the node/TEE, not by client code

## See Also

- [TxSeismicElements](tx-seismic-elements.md) -- Encryption metadata struct
- [TxSeismicMetadata](tx-seismic-metadata.md) -- AAD construction from transaction fields
- [SeismicTxEnvelope](seismic-tx-envelope.md) -- Signed wrapper containing `Signed<TxSeismic>`
- [Shielded Calls](../contract-interaction/shielded-calls.md) -- How to use TxSeismic in practice
- [Encryption](../provider/encryption.md) -- Detailed encryption pipeline
