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

### With SeismicProviderBuilder (Signed)

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::foundry::SeismicFoundry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::<SeismicFoundry>::from(signer);
    let url = "http://127.0.0.1:8545".parse()?;

    let provider = SeismicProviderBuilder::new()
        .foundry()
        .wallet(wallet)
        .connect_http(url)
        .await?;

    let block_number = provider.get_block_number().await?;
    println!("Local block: {block_number}");

    Ok(())
}
```

### Unsigned Provider (Read-Only)

```rust
use seismic_prelude::client::*;

let url = "http://127.0.0.1:8545".parse()?;
let provider = SeismicProviderBuilder::new()
    .foundry()
    .connect_http(url)
    .await?;

let block = provider.get_block_number().await?;
```

## SeismicProviderBuilder with Foundry

To use `SeismicFoundry`, call `.foundry()` on the builder to switch from the default `SeismicReth` network:

| Method              | Description                                            |
| ------------------- | ------------------------------------------------------ |
| `.foundry()`        | Switch to `SeismicFoundry` network type                |
| `.wallet(wallet)`   | Set the wallet for signing transactions                |
| `.connect_http(url)`| Connect to an HTTP RPC endpoint and build the provider |

### Signed Provider

```rust
let provider = SeismicProviderBuilder::new()
    .foundry()
    .wallet(wallet)
    .connect_http(url)
    .await?;
```

| Builder Step        | Required | Description                                          |
| ------------------- | -------- | ---------------------------------------------------- |
| `.foundry()`        | Yes      | Selects SeismicFoundry network                       |
| `.wallet(wallet)`   | Yes      | Wallet containing signers                            |
| `.connect_http(url)`| Yes      | RPC endpoint URL (typically `http://127.0.0.1:8545`) |

**Returns:** A fully configured signed provider for Sanvil.

### Unsigned Provider

```rust
let provider = SeismicProviderBuilder::new()
    .foundry()
    .connect_http(url)
    .await?;
```

| Builder Step        | Required | Description      |
| ------------------- | -------- | ---------------- |
| `.foundry()`        | Yes      | Selects SeismicFoundry network |
| `.connect_http(url)`| Yes      | RPC endpoint URL |

**Returns:** A read-only unsigned provider for Sanvil.

## Transaction Building

With `#[sol(rpc)]`, you can build Seismic transactions directly from contract instances. Functions with shielded parameters auto-encrypt; for others, use `.seismic()`:

```rust
use seismic_prelude::client::*;
use alloy_primitives::address;

sol!(
    #[sol(rpc)]
    contract MyContract {
        function setValue(uint256 value) external;
    }
);

let contract = MyContract::new(contract_address, &provider);
// setValue has no shielded params, so use .seismic() to opt into encryption
let tx = contract.setValue(U256::from(42)).seismic();
```

## When to Use SeismicFoundry

| Scenario                      | Use SeismicFoundry?                        |
| ----------------------------- | ------------------------------------------ |
| Local testing with Sanvil     | Yes                                        |
| Integration tests in CI       | Yes (if using Sanvil)                      |
| Connecting to Seismic testnet | No — use [`SeismicReth`](seismic-reth.md) |
| Connecting to Seismic mainnet | No — use [`SeismicReth`](seismic-reth.md) |

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
