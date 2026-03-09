---
description: Deploy and interact with a shielded contract
icon: rocket
---

# Contract Deployment

This example demonstrates the full contract lifecycle: compile a contract (with notes on sfoundry/ssolc), deploy it transparently, verify the deployment, interact with it using shielded calls, and subscribe to events via WebSocket.

## Prerequisites

```bash
# Install Seismic Foundry tools
# See Seismic Foundry documentation for installation

# Set environment variables
export PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
export RPC_URL="https://node.seismicdev.net/rpc"
export WS_URL="wss://node.seismicdev.net/ws"
```

`Cargo.toml`:

```toml
[package]
name = "contract-deployment"
version = "0.1.0"
edition = "2021"
rust-version = "1.82"

[dependencies]
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy" }
alloy-signer-local = "1.1"
alloy-primitives = "1.1"
alloy-rpc-types-eth = "1.1"
futures-util = "0.3"
hex-literal = "0.4"
tokio = { version = "1", features = ["full"] }
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

The compiled bytecode is in `out/SeismicCounter.sol/SeismicCounter.json`. Extract the `bytecode.object` field for deployment.

{% hint style="info" %}
`ssolc` extends the standard Solidity compiler with support for shielded types (`suint256`, `sbool`, `saddress`, etc.). These types use encrypted storage (`CSTORE`/`CLOAD`) under the hood but are ABI-compatible with their standard counterparts.
{% endhint %}

## Step 2: Deploy the Contract

Contract deployment always uses transparent transactions because Create transactions cannot be seismic.

```rust
use seismic_alloy::prelude::*;
use alloy::sol;
use alloy::sol_types::SolCall;
use alloy_primitives::{Bytes, TxKind, U256};
use alloy_signer_local::PrivateKeySigner;
use alloy_network::ReceiptResponse;
use alloy_provider::SendableTx;
use hex_literal::hex;

sol! {
    interface ISeismicCounter {
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
    let wallet = SeismicWallet::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    // Deploy bytecode (paste compiled bytecode here)
    let deploy_bytecode = Bytes::from_static(&hex!("6080604052..."));

    let deploy_tx = seismic_foundry_tx_builder()
        .with_input(deploy_bytecode)
        .with_kind(TxKind::Create)
        .into();
    // No .seismic() -- deployment is always transparent

    println!("Deploying SeismicCounter...");
    let deploy_pending = provider.send_transaction(deploy_tx).await?;
    let deploy_receipt = deploy_pending.get_receipt().await?;

    assert!(deploy_receipt.status(), "deployment failed");
    let contract_address = deploy_receipt.contract_address
        .expect("deployment should return contract address");
    println!("Deployed at: {contract_address:?}");
    println!("Deploy tx: {:?}", deploy_receipt.transaction_hash);

    Ok(())
}
```

## Step 3: Verify Deployment

After deployment, verify the contract exists at the expected address by checking the deployed bytecode:

```rust
    // Verify deployment -- check that code exists at the address
    let code = provider.get_code_at(contract_address).await?;
    assert!(!code.is_empty(), "no code at deployed address");
    println!("Contract verified: {} bytes of code", code.len());
```

## Step 4: Interact with Shielded Calls

Once deployed, interact with the contract using shielded writes and signed reads:

```rust
    // -------------------------------------------------------
    // Shielded write -- setNumber(42)
    // -------------------------------------------------------
    let calldata = ISeismicCounter::setNumberCall {
        newNumber: U256::from(42),
    }.abi_encode();

    let write_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(calldata))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    println!("\nSending shielded write: setNumber(42)...");
    let write_receipt = provider.send_transaction(write_tx.into()).await?
        .get_receipt().await?;
    assert!(write_receipt.status());
    println!("Confirmed in block {:?}", write_receipt.block_number());

    // -------------------------------------------------------
    // Shielded write -- increment()
    // -------------------------------------------------------
    let inc_calldata = ISeismicCounter::incrementCall {}.abi_encode();

    let inc_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(inc_calldata))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    println!("Sending shielded write: increment()...");
    let inc_receipt = provider.send_transaction(inc_tx.into()).await?
        .get_receipt().await?;
    assert!(inc_receipt.status());

    // -------------------------------------------------------
    // Signed read -- isOdd()
    // -------------------------------------------------------
    let read_input = ISeismicCounter::isOddCall {}.abi_encode();

    let read_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(read_input))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    let result = provider.seismic_call(SendableTx::Builder(read_tx.into())).await?;
    let decoded = ISeismicCounter::isOddReturn::abi_decode(&result, true)?;
    println!("isOdd() = {} (43 is odd)", decoded._0);
    assert!(decoded._0);
```

## Step 5: Subscribe to Events (WebSocket)

Event subscription requires a WebSocket connection. Use `SeismicUnsignedProvider` with `new_ws()`:

```rust
use alloy_rpc_types_eth::Filter;
use futures_util::StreamExt;

async fn subscribe_to_events(
    contract_address: alloy_primitives::Address,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_url: reqwest::Url = std::env::var("WS_URL")?.parse()?;

    // Create WebSocket provider (unsigned -- events are public)
    let ws_provider = SeismicUnsignedProvider::<SeismicReth>::new_ws(ws_url).await?;

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

        // Match against known event signatures
        if let Some(topic) = log.topics().first() {
            // Compare with event signatures from the sol! macro
            println!("  Topic 0: {topic:?}");
        }
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
use seismic_alloy::prelude::*;
use alloy::sol;
use alloy::sol_types::SolCall;
use alloy_primitives::{Bytes, TxKind, U256};
use alloy_signer_local::PrivateKeySigner;
use alloy_network::ReceiptResponse;
use alloy_provider::SendableTx;
use hex_literal::hex;

sol! {
    interface ISeismicCounter {
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
    let wallet = SeismicWallet::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    // 2. Deploy contract (transparent)
    let deploy_bytecode = Bytes::from_static(&hex!("6080604052..."));
    let deploy_tx = seismic_foundry_tx_builder()
        .with_input(deploy_bytecode)
        .with_kind(TxKind::Create)
        .into();

    println!("Deploying contract...");
    let deploy_receipt = provider.send_transaction(deploy_tx).await?
        .get_receipt().await?;
    assert!(deploy_receipt.status());
    let contract_address = deploy_receipt.contract_address.unwrap();
    println!("Deployed at: {contract_address:?}");

    // 3. Verify deployment
    let code = provider.get_code_at(contract_address).await?;
    assert!(!code.is_empty());
    println!("Verified: {} bytes of runtime code", code.len());

    // 4. Shielded write: setNumber(42)
    let calldata = ISeismicCounter::setNumberCall {
        newNumber: U256::from(42),
    }.abi_encode();
    let write_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(calldata))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();
    let write_receipt = provider.send_transaction(write_tx.into()).await?
        .get_receipt().await?;
    assert!(write_receipt.status());
    println!("setNumber(42) confirmed");

    // 5. Shielded write: increment()
    let inc_calldata = ISeismicCounter::incrementCall {}.abi_encode();
    let inc_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(inc_calldata))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();
    let inc_receipt = provider.send_transaction(inc_tx.into()).await?
        .get_receipt().await?;
    assert!(inc_receipt.status());
    println!("increment() confirmed");

    // 6. Signed read: isOdd()
    let read_input = ISeismicCounter::isOddCall {}.abi_encode();
    let read_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(read_input))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();
    let result = provider.seismic_call(SendableTx::Builder(read_tx.into())).await?;
    let decoded = ISeismicCounter::isOddReturn::abi_decode(&result, true)?;
    println!("isOdd() = {} (expected: true, since 43 is odd)", decoded._0);

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

## Common Patterns

### Deploy with Constructor Arguments

If your contract has a constructor that takes arguments, append the ABI-encoded constructor arguments to the bytecode:

```rust
use alloy::sol_types::SolConstructor;

sol! {
    interface IMyContract {
        constructor(uint256 initialValue, address owner);
    }
}

// Encode constructor arguments
let constructor_args = IMyContract::constructorCall {
    initialValue: U256::from(100),
    owner: deployer_address,
}.abi_encode();

// Append to bytecode
let mut deploy_data = contract_bytecode.to_vec();
deploy_data.extend_from_slice(&constructor_args);

let deploy_tx = seismic_foundry_tx_builder()
    .with_input(Bytes::from(deploy_data))
    .with_kind(TxKind::Create)
    .into();
```

### Multiple Contract Deployments

```rust
// Deploy multiple contracts sequentially
let contracts = vec![
    ("Counter", counter_bytecode),
    ("Token", token_bytecode),
    ("Registry", registry_bytecode),
];

let mut addresses = Vec::new();

for (name, bytecode) in contracts {
    let tx = seismic_foundry_tx_builder()
        .with_input(bytecode)
        .with_kind(TxKind::Create)
        .into();

    let receipt = provider.send_transaction(tx).await?
        .get_receipt().await?;
    assert!(receipt.status(), "{name} deployment failed");

    let addr = receipt.contract_address.unwrap();
    println!("Deployed {name} at: {addr:?}");
    addresses.push(addr);
}
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
