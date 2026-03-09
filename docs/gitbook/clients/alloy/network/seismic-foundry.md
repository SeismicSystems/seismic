---
description: Development network type for Sanvil (Seismic Anvil) local testing
icon: flask
---

# SeismicFoundry

`SeismicFoundry` is the network type for connecting to Sanvil (Seismic Anvil), the local development node. Use it when running integration tests or developing locally.

## Overview

`SeismicFoundry` implements both Alloy's `Network` trait and the [`SeismicNetwork`](seismic-network-trait.md) trait, just like [`SeismicReth`](seismic-reth.md). The key difference is that it uses Foundry-compatible transaction types that match Sanvil's expected formats.

Sanvil is Seismic's modified version of Foundry's Anvil. It supports the same Seismic transaction type (`0x4A`) but uses slightly different envelope and request types that are compatible with the Foundry toolchain.

## Definition

```rust
#[derive(Clone, Copy, Debug)]
pub struct SeismicFoundry {
    _private: (),
}
```

Like `SeismicReth`, `SeismicFoundry` is a zero-sized type used purely as a type parameter.

## Associated Types

| Associated Type       | Concrete Type                                | Description                                       |
| --------------------- | -------------------------------------------- | ------------------------------------------------- |
| `TxType`              | `SeismicFoundryTxType`                       | Transaction type enum (Foundry-compatible)        |
| `TxEnvelope`          | `SeismicFoundryTxEnvelope`                   | Signed transaction container (Foundry-compatible) |
| `UnsignedTx`          | `SeismicFoundryTypedTransaction`             | Unsigned transaction before signing               |
| `ReceiptEnvelope`     | `SeismicFoundryReceiptEnvelope`              | Transaction receipt container                     |
| `Header`              | `alloy_consensus::Header`                    | Standard block header                             |
| `TransactionRequest`  | `SeismicFoundryTransactionRequest`           | Builder for transaction parameters                |
| `TransactionResponse` | `Transaction<SeismicFoundryTxEnvelope>`      | RPC response for transaction queries              |
| `ReceiptResponse`     | `SeismicFoundryTransactionReceipt`           | RPC response for receipt queries                  |
| `HeaderResponse`      | `Header`                                     | RPC response for header queries                   |
| `BlockResponse`       | `Block<TransactionResponse, HeaderResponse>` | RPC response for block queries                    |

## Differences from SeismicReth

| Aspect        | SeismicReth                     | SeismicFoundry                     |
| ------------- | ------------------------------- | ---------------------------------- |
| Target node   | reth-based Seismic node         | Sanvil (Foundry-based)             |
| Envelope type | `SeismicTxEnvelope`             | `SeismicFoundryTxEnvelope`         |
| Request type  | `SeismicTransactionRequest`     | `SeismicFoundryTransactionRequest` |
| Receipt type  | `SeismicReceiptEnvelope`        | `SeismicFoundryReceiptEnvelope`    |
| Use case      | Production, testnet, devnet     | Local development, testing         |
| Chain ID      | Varies (e.g., 5124 for testnet) | Typically 31337                    |

{% hint style="info" %}
The Foundry-compatible types handle differences in how Sanvil serializes and deserializes Seismic transactions compared to a full reth-based node. The `SeismicNetwork` trait methods work identically for both network types.
{% endhint %}

## Usage

### With SeismicSignedProvider

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "http://127.0.0.1:8545".parse()?;

    let provider = SeismicSignedProvider::<SeismicFoundry>::new(wallet, url).await?;

    let block_number = provider.get_block_number().await?;
    println!("Local block: {block_number}");

    Ok(())
}
```

### With Convenience Constructor

The `sfoundry_signed_provider` function creates a `SeismicSignedProvider<SeismicFoundry>` without needing to specify the type parameter:

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "http://127.0.0.1:8545".parse()?;

let provider = sfoundry_signed_provider(wallet, url).await?;
```

### Unsigned Provider (Read-Only)

```rust
use seismic_alloy::prelude::*;

let url = "http://127.0.0.1:8545".parse()?;
let provider = sfoundry_unsigned_provider(url);

let block = provider.get_block_number().await?;
```

## Convenience Functions

| Function                                | Description                                                           |
| --------------------------------------- | --------------------------------------------------------------------- |
| `sfoundry_signed_provider(wallet, url)` | Create a signed provider with `SeismicFoundry` network                |
| `sfoundry_unsigned_provider(url)`       | Create an unsigned (read-only) provider with `SeismicFoundry` network |

### `sfoundry_signed_provider`

```rust
pub async fn sfoundry_signed_provider(
    wallet: SeismicWallet<SeismicFoundry>,
    url: reqwest::Url,
) -> Result<SeismicSignedProvider<SeismicFoundry>, Box<dyn std::error::Error>>
```

| Parameter | Type                            | Required | Description                                          |
| --------- | ------------------------------- | -------- | ---------------------------------------------------- |
| `wallet`  | `SeismicWallet<SeismicFoundry>` | Yes      | Wallet containing signers                            |
| `url`     | `reqwest::Url`                  | Yes      | RPC endpoint URL (typically `http://127.0.0.1:8545`) |

**Returns:** A fully configured `SeismicSignedProvider` for Sanvil.

### `sfoundry_unsigned_provider`

```rust
pub fn sfoundry_unsigned_provider(
    url: reqwest::Url,
) -> SeismicUnsignedProvider<SeismicFoundry>
```

| Parameter | Type           | Required | Description      |
| --------- | -------------- | -------- | ---------------- |
| `url`     | `reqwest::Url` | Yes      | RPC endpoint URL |

**Returns:** A read-only `SeismicUnsignedProvider` for Sanvil.

## Transaction Builder

`SeismicFoundry` provides a dedicated transaction builder function:

```rust
let tx_request = seismic_foundry_tx_builder();
```

This returns a `SeismicFoundryTransactionRequest` pre-configured with the Seismic transaction type. You can then set fields like `to`, `value`, `input`, etc.

```rust
use seismic_alloy::prelude::*;
use alloy_primitives::{address, U256};

let tx = seismic_foundry_tx_builder()
    .to(address!("0x1234567890abcdef1234567890abcdef12345678"))
    .value(U256::from(1_000_000_000));
```

## When to Use SeismicFoundry

| Scenario                      | Use SeismicFoundry?                        |
| ----------------------------- | ------------------------------------------ |
| Local testing with Sanvil     | Yes                                        |
| Integration tests in CI       | Yes (if using Sanvil)                      |
| Connecting to Seismic testnet | No -- use [`SeismicReth`](seismic-reth.md) |
| Connecting to Seismic mainnet | No -- use [`SeismicReth`](seismic-reth.md) |

## Notes

- `SeismicFoundry` is a zero-sized type and adds no runtime overhead
- It is `Clone`, `Copy`, and `Debug`
- Sanvil defaults to chain ID `31337`, matching Foundry Anvil conventions
- The Foundry-specific types handle serialization differences between Sanvil and production reth nodes
- Both `SeismicReth` and `SeismicFoundry` support the same `SeismicNetwork` trait methods

## See Also

- [SeismicNetwork Trait](seismic-network-trait.md) - The trait that `SeismicFoundry` implements
- [SeismicReth](seismic-reth.md) - Production network type
- [Sanvil](../chains/sanvil.md) - Local development chain configuration
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Full-featured provider
- [SeismicUnsignedProvider](../provider/seismic-unsigned-provider.md) - Read-only provider
