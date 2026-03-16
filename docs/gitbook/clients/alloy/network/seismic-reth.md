---
description: Production network type for Seismic devnet, testnet, and mainnet
icon: server
---

# SeismicReth

`SeismicReth` is the network type for connecting to production Seismic nodes built on reth. Use it for devnet, testnet, and mainnet deployments.

## Overview

`SeismicReth` implements both Alloy's `Network` trait and the [`SeismicNetwork`](seismic-network-trait.md) trait. It bundles together the Seismic-specific transaction types, receipt types, and signing logic needed to interact with a reth-based Seismic node.

This is the network type you will use most often. The only exception is local development with Sanvil, which requires [`SeismicFoundry`](seismic-foundry.md).

## Definition

```rust
#[derive(Clone, Copy, Debug)]
pub struct SeismicReth {
    _private: (),
}
```

`SeismicReth` is a zero-sized type (ZST) used purely as a type parameter. It carries no runtime data.

## Associated Types

The `Network` implementation for `SeismicReth` defines the following associated types:

| Associated Type       | Concrete Type                                | Description                                          |
| --------------------- | -------------------------------------------- | ---------------------------------------------------- |
| `TxType`              | `SeismicTxType`                              | Transaction type enum (includes `0x4A` Seismic type) |
| `TxEnvelope`          | `SeismicTxEnvelope`                          | Signed transaction container                         |
| `UnsignedTx`          | `SeismicTypedTransaction`                    | Unsigned transaction before signing                  |
| `ReceiptEnvelope`     | `SeismicReceiptEnvelope`                     | Transaction receipt container                        |
| `Header`              | `alloy_consensus::Header`                    | Standard block header                                |
| `TransactionRequest`  | `SeismicTransactionRequest`                  | Builder for transaction parameters                   |
| `TransactionResponse` | `Transaction<SeismicTxEnvelope>`             | RPC response for transaction queries                 |
| `ReceiptResponse`     | `SeismicTransactionReceipt`                  | RPC response for receipt queries                     |
| `HeaderResponse`      | `Header`                                     | RPC response for header queries                      |
| `BlockResponse`       | `Block<TransactionResponse, HeaderResponse>` | RPC response for block queries                       |

## Usage

### With SeismicProviderBuilder (Signed)

The most common way to use `SeismicReth` is via `SeismicProviderBuilder`. Since `SeismicReth` is the default network type, you do not need to specify it explicitly:

```rust
use seismic_alloy_network::{reth::SeismicReth, wallet::SeismicWallet};
use seismic_alloy_provider::SeismicProviderBuilder;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::<SeismicReth>::from(signer);
    let url = "https://gcp-1.seismictest.net/rpc".parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;

    let block_number = provider.get_block_number().await?;
    println!("Block number: {block_number}");

    Ok(())
}
```

### Unsigned Provider (Read-Only)

For read-only access without a private key:

```rust
use seismic_alloy_provider::SeismicProviderBuilder;

let url = "https://gcp-1.seismictest.net/rpc".parse()?;
let provider = SeismicProviderBuilder::new()
    .connect_http(url)
    .await?;

let block = provider.get_block_number().await?;
```

## SeismicProviderBuilder

`SeismicProviderBuilder` provides a fluent API for constructing providers. It defaults to `SeismicReth` as the network type.

| Method              | Description                                        |
| ------------------- | -------------------------------------------------- |
| `.wallet(wallet)`   | Set the wallet for signing transactions            |
| `.connect_http(url)`| Connect to an HTTP RPC endpoint and build provider |

### Signed Provider

```rust
use seismic_alloy_network::{reth::SeismicReth, wallet::SeismicWallet};
use seismic_alloy_provider::SeismicProviderBuilder;

let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_http(url)
    .await?;
```

| Builder Step        | Required | Description                       |
| ------------------- | -------- | --------------------------------- |
| `.wallet(wallet)`   | Yes      | Wallet containing signers         |
| `.connect_http(url)`| Yes      | RPC endpoint URL                  |

**Returns:** A fully configured signed provider with the filler pipeline set up.

### Unsigned Provider

```rust
use seismic_alloy_provider::SeismicProviderBuilder;

let provider = SeismicProviderBuilder::new()
    .connect_http(url)
    .await?;
```

| Builder Step        | Required | Description      |
| ------------------- | -------- | ---------------- |
| `.connect_http(url)`| Yes      | RPC endpoint URL |

**Returns:** A read-only unsigned provider with no wallet or encryption.

## When to Use SeismicReth

| Scenario                      | Use SeismicReth?                                 |
| ----------------------------- | ------------------------------------------------ |
| Connecting to Seismic testnet | Yes                                              |
| Connecting to Seismic devnet  | Yes                                              |
| Connecting to Seismic mainnet | Yes                                              |
| Local testing with Sanvil     | No -- use [`SeismicFoundry`](seismic-foundry.md) |

## Notes

- `SeismicReth` is a zero-sized type and adds no runtime overhead
- It is `Clone`, `Copy`, and `Debug`
- The `_private: ()` field prevents external construction (use it only as a type parameter)
- All Seismic-specific transaction types (`SeismicTxType`, `SeismicTxEnvelope`, etc.) are defined in the `seismic-alloy-consensus` crate

## See Also

- [SeismicNetwork Trait](seismic-network-trait.md) - The trait that `SeismicReth` implements
- [SeismicFoundry](seismic-foundry.md) - Alternative network type for local development
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Full-featured provider
- [SeismicUnsignedProvider](../provider/seismic-unsigned-provider.md) - Read-only provider
- [Seismic Testnet](../chains/seismic-testnet.md) - Testnet chain configuration
