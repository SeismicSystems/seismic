---
description: Seismic public testnet connection and configuration
icon: cloud
---

# Seismic Testnet

The Seismic public testnet is the primary network for development and testing. It runs a full reth-based Seismic node with TEE support, shielded transactions, and all protocol features.

## Configuration

| Property          | Value                              |
| ----------------- | ---------------------------------- |
| Chain ID          | `5124`                             |
| RPC URL           | `https://node.seismicdev.net/rpc`  |
| Network Type      | `SeismicReth`                      |
| Transaction Types | Legacy, EIP-1559, Seismic (`0x4A`) |

## Connecting

### Signed Provider (Full Capabilities)

Use a signed provider for sending transactions, shielded writes, and signed reads:

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;

    let provider = sreth_signed_provider(wallet, url).await?;

    // Verify connection
    let block_number = provider.get_block_number().await?;
    let chain_id = provider.get_chain_id().await?;
    println!("Connected to chain {chain_id} at block {block_number}");

    Ok(())
}
```

### Unsigned Provider (Read-Only)

Use an unsigned provider for read-only operations that do not require a private key:

```rust
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let block_number = provider.get_block_number().await?;
    println!("Testnet block: {block_number}");

    Ok(())
}
```

### With Explicit Type Parameter

If you prefer to specify the network type explicitly:

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "https://node.seismicdev.net/rpc".parse()?;

let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;
```

## Examples

### Send a Shielded Transaction

```rust
use seismic_alloy::prelude::*;
use alloy_primitives::{address, Bytes, U256};
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;

    let provider = sreth_signed_provider(wallet, url).await?;

    // Build a Seismic transaction
    // The filler pipeline automatically handles:
    // - Encryption elements
    // - Calldata encryption
    // - Gas estimation
    // - Nonce and chain ID
    // - Signing

    let block = provider.get_block_number().await?;
    println!("Current block: {block}");

    Ok(())
}
```

### Check Connection

```rust
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    match provider.get_chain_id().await {
        Ok(chain_id) => println!("Connected to chain {chain_id}"),
        Err(e) => println!("Connection failed: {e}"),
    }

    Ok(())
}
```

### Using Environment Variables

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = env::var("PRIVATE_KEY")?;
    let signer: PrivateKeySigner = private_key.parse()?;
    let wallet = SeismicWallet::from(signer);

    let rpc_url = env::var("SEISMIC_RPC_URL")
        .unwrap_or_else(|_| "https://node.seismicdev.net/rpc".to_string());
    let url = rpc_url.parse()?;

    let provider = sreth_signed_provider(wallet, url).await?;

    let block = provider.get_block_number().await?;
    println!("Block: {block}");

    Ok(())
}
```

## Notes

- Chain ID `5124` is used for EIP-155 and EIP-712 transaction signing
- The testnet supports all Seismic protocol features including shielded transactions and signed reads
- The TEE public key is fetched automatically by `SeismicElementsFiller` on the first transaction
- WebSocket endpoints may be available; check the Seismic documentation for current URLs
- The testnet is suitable for development and testing but not for production use

## See Also

- [Chains Overview](./) - All supported chains
- [Sanvil](sanvil.md) - Local development network
- [SeismicReth](../network/seismic-reth.md) - Network type used with testnet
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Full-featured provider
- [Installation](../installation.md) - Cargo setup and prerequisites
