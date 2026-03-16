---
description: Read-only Seismic provider for public operations without a wallet
icon: globe
---

# SeismicUnsignedProvider

Read-only provider for public operations. No wallet or signing capabilities.

## Overview

`SeismicUnsignedProvider<N: SeismicNetwork>` is a lightweight provider for reading public data from Seismic nodes. It does not carry a wallet, cannot sign transactions, and cannot decrypt responses from `seismic_call`. Use it for:

- Querying block data, balances, and transaction receipts
- Fetching the TEE public key
- Public (unencrypted) contract reads
- WebSocket subscriptions

For full capabilities (shielded writes, signed reads, response decryption), use [SeismicSignedProvider](seismic-signed-provider.md) instead.

## Construction

All unsigned providers are created via `SeismicProviderBuilder` -- simply omit `.wallet()`:

### HTTP

```rust
use seismic_prelude::client::*;

let url = "https://testnet-1.seismictest.net/rpc".parse()?;
// connect_http is synchronous for unsigned providers (no .await needed)
let provider = SeismicProviderBuilder::new().connect_http(url);

let block_number = provider.get_block_number().await?;
println!("Block number: {block_number}");
```

### WebSocket

```rust
use seismic_prelude::client::*;

let url = "wss://testnet-1.seismictest.net/ws".parse()?;
let provider = SeismicProviderBuilder::new().connect_ws(url).await?;

let block_number = provider.get_block_number().await?;
println!("Block number: {block_number}");
```

{% hint style="info" %}
`connect_ws()` is async because it establishes the WebSocket connection during construction. Use WebSocket transport when you need persistent connections or event subscriptions.
{% endhint %}

### Local Development with sanvil

Use `.foundry()` to select the `SeismicFoundry` network type:

```rust
use seismic_prelude::client::*;

let url = "http://localhost:8545".parse()?;
// Synchronous -- no .await
let provider = SeismicProviderBuilder::new()
    .foundry()
    .connect_http(url);
```

## Methods

### Via `SeismicProviderExt`

#### `get_tee_pubkey()`

Fetch the TEE public key from the node.

```rust
// SeismicProviderExt is included in the prelude

let tee_pubkey = provider.get_tee_pubkey().await?;
println!("TEE public key: {tee_pubkey}");
```

{% hint style="info" %}
Unlike `SeismicSignedProvider`, the unsigned provider does **not** cache the TEE pubkey. Each call to `get_tee_pubkey()` makes a fresh RPC request to `seismic_getTeePublicKey`.
{% endhint %}

{% hint style="warning" %}
`SeismicProviderExt::transparent_send` is visible on this type for trait-bound reasons, but calling it fails at runtime -- there is no wallet to sign with. Use `SeismicSignedProvider` whenever you need to broadcast a transaction, transparent or shielded.
{% endhint %}

### Via Standard Alloy `Provider`

All standard Alloy provider methods are available:

```rust
// Block queries
let block_number = provider.get_block_number().await?;
let block = provider.get_block_by_number(BlockNumberOrTag::Latest, false).await?;

// Account queries
let balance = provider.get_balance(address).await?;
let nonce = provider.get_transaction_count(address).await?;

// Transaction queries (read-only)
let receipt = provider.get_transaction_receipt(tx_hash).await?;
let tx = provider.get_transaction_by_hash(tx_hash).await?;

// Chain info
let chain_id = provider.get_chain_id().await?;
```

## Filler Chain

The unsigned provider uses a minimal chain of standard Alloy fillers, shown in operational order:

| Step | Filler                          | Purpose                                                           |
| ---- | ------------------------------- | ----------------------------------------------------------------- |
| 1    | `NonceFiller` + `ChainIdFiller` | Composed together; fetch and set the nonce and chain ID           |
| 2    | `GasFiller`                     | Standard Alloy gas estimation via `eth_estimateGas`               |

{% hint style="info" %}
The unsigned provider does not include `SeismicElementsFiller` or `SeismicGasFiller` since it cannot perform shielded operations. Only the signed provider includes the full Seismic filler pipeline.
{% endhint %}

## Examples

### Query Block Data

```rust
use seismic_prelude::client::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://testnet-1.seismictest.net/rpc".parse()?;
    let provider = SeismicProviderBuilder::new().connect_http(url);

    let block_number = provider.get_block_number().await?;
    println!("Current block: {block_number}");

    let chain_id = provider.get_chain_id().await?;
    println!("Chain ID: {chain_id}");

    Ok(())
}
```

### Check Account Balance

```rust
use seismic_prelude::client::*;
use alloy_primitives::address;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://testnet-1.seismictest.net/rpc".parse()?;
    let provider = SeismicProviderBuilder::new().connect_http(url);

    let addr = address!("0x1234567890abcdef1234567890abcdef12345678");
    let balance = provider.get_balance(addr).await?;
    println!("Balance: {balance}");

    Ok(())
}
```

### Fetch TEE Public Key

```rust
use seismic_prelude::client::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://testnet-1.seismictest.net/rpc".parse()?;
    let provider = SeismicProviderBuilder::new().connect_http(url);

    let tee_pubkey = provider.get_tee_pubkey().await?;
    println!("TEE public key: {tee_pubkey}");

    // Pass this to connect_http_with_tee_pubkey() for synchronous
    // signed provider construction

    Ok(())
}
```

### WebSocket Connection

```rust
use seismic_prelude::client::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://testnet-1.seismictest.net/ws".parse()?;
    let provider = SeismicProviderBuilder::new().connect_ws(url).await?;

    let block_number = provider.get_block_number().await?;
    println!("Block (via WebSocket): {block_number}");

    Ok(())
}
```

## Limitations

| Limitation                    | Details                                                                  |
| ----------------------------- | ------------------------------------------------------------------------ |
| **No shielded call builder**  | `.seismic()` and auto-encryption are compile-time restricted to signed providers |
| **No transaction signing**    | Cannot send transactions -- use `SeismicSignedProvider`                  |
| **No shielded calls**         | `SignedProviderExt` methods (`seismic_call`, `seismic_send`, etc.) are not available |
| **No TEE pubkey caching**     | Each `get_tee_pubkey()` call makes a fresh RPC request                   |

## Notes

- The unsigned provider is ideal for monitoring, indexing, and read-only applications
- All standard Alloy `Provider` methods work unchanged
- Both HTTP and WebSocket transports are supported

## See Also

- [SeismicSignedProvider](seismic-signed-provider.md) -- Full-featured provider with wallet and encryption
- [Encryption](encryption.md) -- How calldata encryption works (signed provider only)
- [Provider Overview](./) -- Comparison of provider types
- [Installation](../installation.md) -- Add seismic-alloy to your project
