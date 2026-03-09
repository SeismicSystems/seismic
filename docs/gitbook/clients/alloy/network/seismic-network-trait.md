---
description: Core trait defining Seismic network behavior
icon: code
---

# SeismicNetwork Trait

The `SeismicNetwork` trait is the core abstraction that all Seismic network types must implement. It extends Alloy's `Network` and `RecommendedFillers` traits with methods for handling shielded transaction elements, encrypted calldata, and Seismic-specific signing.

## Overview

In Alloy's architecture, a `Network` trait bundles together all the associated types needed to interact with a specific chain: transaction types, receipt types, header types, and so on. `SeismicNetwork` builds on this by adding methods that the Seismic filler pipeline and wallet need to:

1. Attach encryption elements (TEE public key, nonce, etc.) to transaction requests
2. Read and write encrypted calldata on both requests and signed envelopes
3. Sign transactions using `SeismicWallet`
4. Identify whether a transaction is a Seismic type
5. Extract Seismic-specific metadata from signed envelopes

## Trait Definition

```rust
#[async_trait::async_trait]
pub trait SeismicNetwork: Network + RecommendedFillers + Send + Sync
where
    Self::UnsignedTx: Send + Sync,
{
    fn set_seismic_elements(
        req: &mut Self::TransactionRequest,
        seismic_elements: TxSeismicElements,
    );

    fn get_seismic_elements(
        req: &Self::TransactionRequest,
    ) -> Option<TxSeismicElements>;

    fn get_request_input(
        req: &Self::TransactionRequest,
    ) -> Option<&Bytes>;

    fn get_envelope_input(
        req: &Self::TxEnvelope,
    ) -> &Bytes;

    fn set_request_input(
        req: &mut Self::TransactionRequest,
        input: Bytes,
    ) -> Result<(), InputDecryptionElementsError>;

    fn set_envelope_input(
        req: &mut Self::TxEnvelope,
        input: Bytes,
    ) -> Result<(), InputDecryptionElementsError>;

    async fn sign_transaction_from(
        wallet: &SeismicWallet<Self>,
        sender: Address,
        tx: Self::UnsignedTx,
    ) -> Result<Self::TxEnvelope, alloy_signer::Error>;

    fn is_seismic_tx_type(ty: Self::TxType) -> bool;

    fn extract_seismic_metadata(
        envelope: &Self::TxEnvelope,
    ) -> Option<TxSeismicMetadata>;
}
```

## Supertraits

| Supertrait           | Source          | Purpose                                                      |
| -------------------- | --------------- | ------------------------------------------------------------ |
| `Network`            | `alloy_network` | Defines associated types for transactions, receipts, headers |
| `RecommendedFillers` | `alloy_network` | Specifies the default filler stack for the network           |
| `Send + Sync`        | `std`           | Required for use across async task boundaries                |

## Required Methods

### `set_seismic_elements`

Attaches Seismic encryption elements to a transaction request. Called by `SeismicElementsFiller` during the fill phase.

```rust
fn set_seismic_elements(
    req: &mut Self::TransactionRequest,
    seismic_elements: TxSeismicElements,
);
```

| Parameter          | Type                            | Required | Description                                          |
| ------------------ | ------------------------------- | -------- | ---------------------------------------------------- |
| `req`              | `&mut Self::TransactionRequest` | Yes      | Mutable reference to the transaction request         |
| `seismic_elements` | `TxSeismicElements`             | Yes      | Encryption nonce, TEE public key, and other elements |

### `get_seismic_elements`

Retrieves previously set Seismic elements from a transaction request. Returns `None` if no elements have been set.

```rust
fn get_seismic_elements(
    req: &Self::TransactionRequest,
) -> Option<TxSeismicElements>;
```

| Parameter | Type                        | Required | Description                          |
| --------- | --------------------------- | -------- | ------------------------------------ |
| `req`     | `&Self::TransactionRequest` | Yes      | Reference to the transaction request |

**Returns:** `Option<TxSeismicElements>` -- `Some` if elements are present, `None` otherwise.

### `get_request_input`

Gets the calldata (`input`) from a transaction request. This is the raw calldata before or after encryption.

```rust
fn get_request_input(
    req: &Self::TransactionRequest,
) -> Option<&Bytes>;
```

| Parameter | Type                        | Required | Description                          |
| --------- | --------------------------- | -------- | ------------------------------------ |
| `req`     | `&Self::TransactionRequest` | Yes      | Reference to the transaction request |

**Returns:** `Option<&Bytes>` -- The calldata bytes, if present.

### `get_envelope_input`

Gets the calldata from a signed transaction envelope. Unlike `get_request_input`, this always returns a reference because signed envelopes must have input data.

```rust
fn get_envelope_input(
    req: &Self::TxEnvelope,
) -> &Bytes;
```

| Parameter | Type                | Required | Description                                  |
| --------- | ------------------- | -------- | -------------------------------------------- |
| `req`     | `&Self::TxEnvelope` | Yes      | Reference to the signed transaction envelope |

**Returns:** `&Bytes` -- The calldata bytes.

### `set_request_input`

Sets the calldata on a transaction request. Used by the encryption pipeline to replace plaintext calldata with encrypted calldata.

```rust
fn set_request_input(
    req: &mut Self::TransactionRequest,
    input: Bytes,
) -> Result<(), InputDecryptionElementsError>;
```

| Parameter | Type                            | Required | Description                                  |
| --------- | ------------------------------- | -------- | -------------------------------------------- |
| `req`     | `&mut Self::TransactionRequest` | Yes      | Mutable reference to the transaction request |
| `input`   | `Bytes`                         | Yes      | The calldata to set (typically encrypted)    |

**Returns:** `Result<(), InputDecryptionElementsError>` -- `Ok(())` on success, or an error if the operation fails.

### `set_envelope_input`

Sets the calldata on a signed transaction envelope. Used for response decryption when the provider needs to replace encrypted response data with decrypted data.

```rust
fn set_envelope_input(
    req: &mut Self::TxEnvelope,
    input: Bytes,
) -> Result<(), InputDecryptionElementsError>;
```

| Parameter | Type                    | Required | Description                               |
| --------- | ----------------------- | -------- | ----------------------------------------- |
| `req`     | `&mut Self::TxEnvelope` | Yes      | Mutable reference to the signed envelope  |
| `input`   | `Bytes`                 | Yes      | The calldata to set (typically decrypted) |

**Returns:** `Result<(), InputDecryptionElementsError>` -- `Ok(())` on success, or an error if the operation fails.

### `sign_transaction_from`

Signs an unsigned transaction using a specific signer from the wallet, identified by `sender` address. This is an async method because signing may involve hardware signers or other async operations.

```rust
async fn sign_transaction_from(
    wallet: &SeismicWallet<Self>,
    sender: Address,
    tx: Self::UnsignedTx,
) -> Result<Self::TxEnvelope, alloy_signer::Error>;
```

| Parameter | Type                   | Required | Description                      |
| --------- | ---------------------- | -------- | -------------------------------- |
| `wallet`  | `&SeismicWallet<Self>` | Yes      | The wallet containing signers    |
| `sender`  | `Address`              | Yes      | Address of the signer to use     |
| `tx`      | `Self::UnsignedTx`     | Yes      | The unsigned transaction to sign |

**Returns:** `Result<Self::TxEnvelope, alloy_signer::Error>` -- The signed transaction envelope, or a signing error.

### `is_seismic_tx_type`

Checks whether a given transaction type is a Seismic transaction type (type `0x4A`). Used by fillers to determine whether to apply Seismic-specific processing.

```rust
fn is_seismic_tx_type(ty: Self::TxType) -> bool;
```

| Parameter | Type           | Required | Description                   |
| --------- | -------------- | -------- | ----------------------------- |
| `ty`      | `Self::TxType` | Yes      | The transaction type to check |

**Returns:** `bool` -- `true` if this is a Seismic transaction type.

### `extract_seismic_metadata`

Extracts Seismic-specific metadata from a signed transaction envelope. Returns `None` if the envelope is not a Seismic transaction.

```rust
fn extract_seismic_metadata(
    envelope: &Self::TxEnvelope,
) -> Option<TxSeismicMetadata>;
```

| Parameter  | Type                | Required | Description                     |
| ---------- | ------------------- | -------- | ------------------------------- |
| `envelope` | `&Self::TxEnvelope` | Yes      | The signed transaction envelope |

**Returns:** `Option<TxSeismicMetadata>` -- Seismic metadata if present, `None` for non-Seismic transactions.

## Implementations

The SDK provides two implementations of `SeismicNetwork`:

| Implementation                         | Module                  | Description                               |
| -------------------------------------- | ----------------------- | ----------------------------------------- |
| [`SeismicReth`](seismic-reth.md)       | `seismic_alloy_network` | For production Seismic nodes (reth-based) |
| [`SeismicFoundry`](seismic-foundry.md) | `seismic_alloy_network` | For Sanvil (local development)            |

## How Fillers Use SeismicNetwork

The filler pipeline calls these methods in sequence during transaction preparation:

1. **SeismicElementsFiller** calls `set_seismic_elements()` to attach encryption nonce, TEE public key, and expiration block
2. **SeismicElementsFiller** calls `get_request_input()` to read plaintext calldata, encrypts it, then calls `set_request_input()` to write the encrypted calldata back
3. **SeismicGasFiller** calls `is_seismic_tx_type()` to determine whether to defer gas estimation
4. **WalletFiller** calls `sign_transaction_from()` to sign the fully prepared transaction

## Implementing a Custom Network

If you need to implement `SeismicNetwork` for a custom network type, you must:

1. Implement Alloy's `Network` trait with your associated types
2. Implement `RecommendedFillers` to specify the default filler stack
3. Implement all nine `SeismicNetwork` methods

{% hint style="info" %}
Custom network implementations are an advanced use case. Most users should use `SeismicReth` or `SeismicFoundry`.
{% endhint %}

## See Also

- [SeismicReth](seismic-reth.md) - Production network implementation
- [SeismicFoundry](seismic-foundry.md) - Development network implementation
- [SeismicElementsFiller](../fillers/seismic-elements-filler.md) - Filler that uses these methods
- [SeismicWallet](../wallet/seismic-wallet.md) - Wallet type used by `sign_transaction_from`
- [Network Overview](./) - Network layer overview
