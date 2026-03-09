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

### With SeismicSignedProvider

The most common way to use `SeismicReth` is as the type parameter for `SeismicSignedProvider`:

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;

    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    let block_number = provider.get_block_number().await?;
    println!("Block number: {block_number}");

    Ok(())
}
```

### With Convenience Constructor

The `sreth_signed_provider` function creates a `SeismicSignedProvider<SeismicReth>` without needing to specify the type parameter:

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "https://node.seismicdev.net/rpc".parse()?;

let provider = sreth_signed_provider(wallet, url).await?;
```

### Unsigned Provider (Read-Only)

For read-only access without a private key:

```rust
use seismic_alloy::prelude::*;

let url = "https://node.seismicdev.net/rpc".parse()?;
let provider = sreth_unsigned_provider(url);

let block = provider.get_block_number().await?;
```

## Convenience Functions

These functions are re-exported through the prelude and select `SeismicReth` as the network type automatically:

| Function                             | Description                                                        |
| ------------------------------------ | ------------------------------------------------------------------ |
| `sreth_signed_provider(wallet, url)` | Create a signed provider with `SeismicReth` network                |
| `sreth_unsigned_provider(url)`       | Create an unsigned (read-only) provider with `SeismicReth` network |

### `sreth_signed_provider`

```rust
pub async fn sreth_signed_provider(
    wallet: SeismicWallet<SeismicReth>,
    url: reqwest::Url,
) -> Result<SeismicSignedProvider<SeismicReth>, Box<dyn std::error::Error>>
```

| Parameter | Type                         | Required | Description               |
| --------- | ---------------------------- | -------- | ------------------------- |
| `wallet`  | `SeismicWallet<SeismicReth>` | Yes      | Wallet containing signers |
| `url`     | `reqwest::Url`               | Yes      | RPC endpoint URL          |

**Returns:** A fully configured `SeismicSignedProvider` with the filler pipeline set up.

### `sreth_unsigned_provider`

```rust
pub fn sreth_unsigned_provider(
    url: reqwest::Url,
) -> SeismicUnsignedProvider<SeismicReth>
```

| Parameter | Type           | Required | Description      |
| --------- | -------------- | -------- | ---------------- |
| `url`     | `reqwest::Url` | Yes      | RPC endpoint URL |

**Returns:** A read-only `SeismicUnsignedProvider` with no wallet or encryption.

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
