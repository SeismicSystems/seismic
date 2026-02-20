---
description: Complete provider setup with connection verification
icon: play
---

# Basic Setup

This example demonstrates how to create Seismic providers in both signed and unsigned variants, verify the connection, and query basic chain state.

## Prerequisites

```bash
# Install Rust 1.82+
rustup update stable

# Set environment variables
export PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
export RPC_URL="https://node.seismicdev.net/rpc"
```

`Cargo.toml`:

```toml
[package]
name = "basic-setup"
version = "0.1.0"
edition = "2021"
rust-version = "1.82"

[dependencies]
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy" }
alloy-signer-local = "1.1"
alloy-primitives = "1.1"
tokio = { version = "1", features = ["full"] }
```

## Signed Provider (Full Capabilities)

A signed provider can send shielded writes, perform signed reads, and execute all standard Alloy provider operations.

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load private key from environment
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;

    // Create signed provider (fetches TEE pubkey automatically)
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    // Verify connection
    let block_number = provider.get_block_number().await?;
    println!("Block number: {block_number}");

    // Get chain ID
    let chain_id = provider.get_chain_id().await?;
    println!("Chain ID: {chain_id}");

    // Get TEE public key (cached from construction)
    let tee_pubkey = provider.get_tee_pubkey().await?;
    println!("TEE public key: {:?}", tee_pubkey);

    // Check balance
    let address = provider.get_accounts().await?[0];
    let balance = provider.get_balance(address).await?;
    println!("Address: {address}");
    println!("Balance: {balance} wei");

    Ok(())
}
```

## Unsigned Provider (Read-Only)

An unsigned provider does not require a private key. It can query chain state, read public data, and subscribe to events (via WebSocket), but cannot send transactions or perform signed reads.

```rust
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;

    // Create unsigned provider (no private key needed)
    let provider = sreth_unsigned_provider(url);

    // Verify connection
    let block_number = provider.get_block_number().await?;
    println!("Block number: {block_number}");

    // Get chain ID
    let chain_id = provider.get_chain_id().await?;
    println!("Chain ID: {chain_id}");

    // Get TEE public key
    let tee_pubkey = provider.get_tee_pubkey().await?;
    println!("TEE public key: {:?}", tee_pubkey);

    // Check any address balance
    let address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".parse()?;
    let balance = provider.get_balance(address).await?;
    println!("Balance: {balance} wei");

    Ok(())
}
```

## Using Convenience Constructors

The SDK provides shorthand functions that select the correct network type automatically:

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;

    // For production / testnet (SeismicReth)
    let provider = sreth_signed_provider(wallet.clone(), url.clone()).await?;
    println!("Production block: {}", provider.get_block_number().await?);

    // For local development (SeismicFoundry)
    // let local_url: reqwest::Url = "http://127.0.0.1:8545".parse()?;
    // let provider = sfoundry_signed_provider(wallet, local_url).await?;

    Ok(())
}
```

## Local Development with Sanvil

For local testing using Sanvil (Seismic Anvil):

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;
use alloy_node_bindings::Anvil;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Spawn a local Sanvil instance
    let anvil = Anvil::at("sanvil").spawn();

    // Use one of the pre-funded accounts
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = SeismicWallet::from(signer);

    // Connect to the local instance
    let provider = SeismicSignedProvider::<SeismicFoundry>::new(
        wallet,
        anvil.endpoint_url(),
    ).await?;

    let block = provider.get_block_number().await?;
    println!("Local Sanvil block: {block}");

    let chain_id = provider.get_chain_id().await?;
    println!("Chain ID: {chain_id}");

    Ok(())
}
```

{% hint style="info" %}
`Anvil::at("sanvil")` requires `sanvil` to be installed and available on your `PATH`. See the Seismic Foundry documentation for installation instructions.
{% endhint %}

## Error Handling

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;

    // Provider creation can fail if the node is unreachable
    // or the TEE pubkey fetch times out
    let provider = match SeismicSignedProvider::<SeismicReth>::new(wallet, url).await {
        Ok(p) => {
            println!("Provider created successfully");
            p
        }
        Err(e) => {
            eprintln!("Failed to create provider: {e}");
            eprintln!("Check that RPC_URL is reachable and the node is running");
            return Err(e.into());
        }
    };

    // Verify the connection is healthy
    match provider.get_block_number().await {
        Ok(block) => println!("Connected. Block number: {block}"),
        Err(e) => {
            eprintln!("Connection check failed: {e}");
            return Err(e.into());
        }
    }

    Ok(())
}
```

## Expected Output

```
Block number: 12345
Chain ID: 1946
TEE public key: PublicKey(028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0)
Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
Balance: 10000000000000000000 wei
```

## Next Steps

- [Shielded Write Complete](shielded-write-complete.md) - Send encrypted transactions
- [Signed Read Pattern](signed-read-pattern.md) - Execute authenticated calls
- [Contract Deployment](contract-deployment.md) - Deploy and interact with contracts

## See Also

- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Full-featured provider API
- [SeismicUnsignedProvider](../provider/seismic-unsigned-provider.md) - Read-only provider API
- [Installation](../installation.md) - Cargo setup and dependencies
- [Provider Overview](../provider/) - Provider comparison and filler pipeline
