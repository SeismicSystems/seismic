---
description: Deploy and interact with a shielded contract
icon: rocket
---

# Contract Deployment

This example demonstrates the full contract lifecycle: compile a contract (with notes on sfoundry/ssolc), deploy it, verify the deployment, interact with it using shielded calls, and subscribe to events via WebSocket.

## Prerequisites

```bash
# Install Seismic Foundry tools
# See Seismic Foundry documentation for installation

# Set environment variables
export PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
export RPC_URL="https://testnet-1.seismictest.net/rpc"
export WS_URL="wss://testnet-1.seismictest.net/ws"
```

`Cargo.toml` — see [Installation](../installation.md) for the full template including the required `[patch.crates-io]` block:

```toml
[package]
name = "contract-deployment"
version = "0.1.0"
edition = "2021"
rust-version = "1.82"

[dependencies]
seismic-prelude        = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-network  = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-provider = { git = "https://github.com/SeismicSystems/seismic-alloy" }
alloy-provider         = "1.1"
alloy-signer-local     = "1.1"
alloy-primitives       = "1.1"
alloy-sol-types        = "1.1"
alloy-network          = "1.1"
alloy-rpc-types-eth    = "1.1"
futures-util           = "0.3"
tokio                  = { version = "1", features = ["full"] }
reqwest                = "0.12"

# [patch.crates-io] block required — see Installation.
```

## Step 1: Compile the Contract

Seismic contracts use `ssolc` (Seismic Solidity Compiler) and `sfoundry` (Seismic Foundry) for compilation. A typical contract:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

contract SeismicCounter {
    suint256 number;

    event setNumberEmit();
    event incrementEmit();

    function setNumber(suint256 newNumber) public {
        number = newNumber;
        emit setNumberEmit();
    }

    function increment() public {
        number++;
        emit incrementEmit();
    }

    function isOdd() public view returns (bool) {
        return (number % 2 != 0);
    }
}
```

Compile with sfoundry:

```bash
# In your Foundry project directory
sforge build
```

The compiled bytecode is in `out/SeismicCounter.sol/SeismicCounter.json`. Extract the `bytecode.object` field for the `#[sol(rpc, bytecode = "...")]` attribute.

{% hint style="info" %}
`ssolc` extends the standard Solidity compiler with support for shielded types (`suint256`, `sbool`, `saddress`, etc.). These types use encrypted storage (`CSTORE`/`CLOAD`) under the hood but are ABI-compatible with their standard counterparts.
{% endhint %}

## Step 2: Deploy and Interact

Use `#[sol(rpc, bytecode = "...")]` to generate a `deploy()` method and type-safe call builders:

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;
use alloy_network::ReceiptResponse;
use alloy_provider::Provider;

sol! {
    #[sol(rpc, bytecode = "6080604052...")]
    contract SeismicCounter {
        event setNumberEmit();
        event incrementEmit();

        function setNumber(suint256 newNumber) public;
        function increment() public;
        function isOdd() public view returns (bool);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up provider
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::<SeismicReth>::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;

    // Deploy contract (transparent — Create txs cannot be seismic)
    println!("Deploying SeismicCounter...");
    let contract = SeismicCounter::deploy(&provider).await?;
    println!("Deployed at: {:?}", contract.address());

    // Verify deployment
    let code = provider.get_code_at(*contract.address()).await?;
    assert!(!code.is_empty());
    println!("Verified: {} bytes of runtime code", code.len());

    Ok(())
}
```

## Step 3: Shielded Interactions

Once deployed, interact with the contract using shielded calls:

```rust
    // Shielded write — setNumber has suint256 param, auto-encrypts
    println!("\nSending shielded write: setNumber(42)...");
    let write_receipt = contract
        .setNumber(alloy_primitives::aliases::SUInt(U256::from(42)))
        .send()
        .await?
        .get_receipt()
        .await?;
    assert!(write_receipt.status());
    println!("Confirmed in block {:?}", write_receipt.block_number());

    // Shielded write — increment()
    println!("Sending shielded write: increment()...");
    let inc_receipt = contract
        .increment()
        .seismic()
        .send()
        .await?
        .get_receipt()
        .await?;
    assert!(inc_receipt.status());

    // Signed read — isOdd()
    let is_odd = contract.isOdd().seismic().call().await?;
    println!("isOdd() = {is_odd} (43 is odd)");
    assert!(is_odd);
```

## Step 4: Subscribe to Events (WebSocket)

Event subscription requires a WebSocket connection. Use an unsigned provider:

```rust
use seismic_prelude::client::*;
use alloy_rpc_types_eth::Filter;
use futures_util::StreamExt;

async fn subscribe_to_events(
    contract_address: alloy_primitives::Address,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_url: reqwest::Url = std::env::var("WS_URL")?.parse()?;

    // Create WebSocket provider (unsigned — events are public)
    let ws_provider = SeismicProviderBuilder::new()
        .connect_ws(ws_url)
        .await?;

    // Create a filter for all events from this contract
    let filter = Filter::new().address(contract_address);

    // Subscribe to logs
    let event_sub = ws_provider.subscribe_logs(&filter).await?;
    let mut event_stream = event_sub.into_stream();

    println!("\nListening for events from {contract_address:?}...");
    println!("(Send a transaction to the contract to see events)");

    // Process events as they arrive
    while let Some(log) = event_stream.next().await {
        println!("Event received:");
        println!("  Block: {:?}", log.block_number);
        println!("  Tx hash: {:?}", log.transaction_hash);
        println!("  Topics: {:?}", log.topics());
        println!("  Data: {:?}", log.data());
    }

    Ok(())
}
```

{% hint style="info" %}
Event data emitted by `emit` is public and visible on-chain. Event subscription does not require a signed provider. If you need to keep event data private, avoid emitting sensitive values.
{% endhint %}

## Full Working Example

Combining all steps into one program:

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;
use alloy_network::ReceiptResponse;
use alloy_provider::Provider;

sol! {
    #[sol(rpc, bytecode = "6080604052...")]
    contract SeismicCounter {
        event setNumberEmit();
        event incrementEmit();

        function setNumber(suint256 newNumber) public;
        function increment() public;
        function isOdd() public view returns (bool);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Set up provider
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::<SeismicReth>::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;

    // 2. Deploy contract
    println!("Deploying contract...");
    let contract = SeismicCounter::deploy(&provider).await?;
    println!("Deployed at: {:?}", contract.address());

    // 3. Verify deployment
    let code = provider.get_code_at(*contract.address()).await?;
    assert!(!code.is_empty());
    println!("Verified: {} bytes of runtime code", code.len());

    // 4. Shielded write: setNumber auto-encrypts (suint256 param)
    contract.setNumber(alloy_primitives::aliases::SUInt(U256::from(42)))
        .send()
        .await?
        .get_receipt()
        .await?;
    println!("setNumber(42) confirmed");

    // 5. Shielded write: increment needs .seismic() (no shielded params)
    contract.increment()
        .seismic()
        .send()
        .await?
        .get_receipt()
        .await?;
    println!("increment() confirmed");

    // 6. Signed read: isOdd()
    let is_odd = contract.isOdd().seismic().call().await?;
    println!("isOdd() = {is_odd} (expected: true, since 43 is odd)");

    println!("\nContract deployment and interaction complete!");
    Ok(())
}
```

## Expected Output

```
Deploying contract...
Deployed at: 0x5FbDB2315678afecb367f032d93F642f64180aa3
Verified: 1234 bytes of runtime code
setNumber(42) confirmed
increment() confirmed
isOdd() = true (expected: true, since 43 is odd)

Contract deployment and interaction complete!
```

## Next Steps

- [Shielded Write Complete](shielded-write-complete.md) - Full shielded write lifecycle
- [Signed Read Pattern](signed-read-pattern.md) - Authenticated read examples
- [Basic Setup](basic-setup.md) - Provider setup details

## See Also

- [Transparent Calls](../contract-interaction/transparent-calls.md) - Deployment and transparent operations
- [Shielded Calls](../contract-interaction/shielded-calls.md) - Encrypted operations
- [Shielded Write Guide](../guides/shielded-write.md) - Step-by-step guide
- [Signed Reads Guide](../guides/signed-reads.md) - Signed read guide
- [Installation](../installation.md) - Cargo setup and dependencies
