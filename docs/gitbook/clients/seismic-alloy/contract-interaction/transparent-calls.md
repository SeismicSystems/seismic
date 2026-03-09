---
description: Standard Ethereum calls without encryption
icon: eye
---

# Transparent Calls

Non-encrypted contract interactions using standard Ethereum transaction types. Use these for public data, contract deployment, and operations that do not require privacy.

## Overview

Transparent calls are standard Ethereum operations that do not use Seismic encryption. Calldata and return values are visible on-chain, just like any regular Ethereum transaction. You build a transaction request **without** calling `.seismic()`, and use the standard Alloy `send_transaction()` or `call()` methods.

## When to Use Transparent Calls

| Scenario                           | Use Transparent |           Use Shielded            |
| ---------------------------------- | :-------------: | :-------------------------------: |
| Contract deployment                |       Yes       | No (Create txs cannot be seismic) |
| Reading public state               |       Yes       |                No                 |
| Writing public state               |       Yes       |             Optional              |
| Reading private state              |       No        |                Yes                |
| Writing private state              |       No        |                Yes                |
| Functions with shielded parameters |       No        |                Yes                |

## Contract Deployment

Contract deployment always uses transparent transactions because Create transactions cannot be seismic. After deployment, you can interact with the contract using shielded calls.

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::{Bytes, TxKind};

// Contract bytecode (from compilation)
let bytecode: Bytes = "0x6080604052...".parse()?;

// Build a standard (non-seismic) transaction for deployment
let tx = seismic_foundry_tx_builder()
    .with_input(bytecode)
    .with_kind(TxKind::Create)  // Create transaction
    .into();
    // No .seismic() -- deployment is always transparent

// Send deployment transaction
let pending_tx = provider.send_transaction(tx).await?;
let receipt = pending_tx.get_receipt().await?;

// Extract deployed contract address
let contract_address = receipt.contract_address
    .expect("deployment should return contract address");
println!("Deployed to: {:?}", contract_address);
```

{% hint style="info" %}
After deploying a contract transparently, you can immediately interact with it using shielded calls. See [Shielded Calls](shielded-calls.md) for details.
{% endhint %}

## Transparent Write

A transparent write sends a standard `eth_sendTransaction` with unencrypted calldata. Use this for functions that do not handle private data.

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::{Address, U256, TxKind};
use alloy::sol_types::SolCall;

sol! {
    interface IMyContract {
        function setPublicValue(uint256 value) public;
    }
}

let contract_address: Address = "0x1234...".parse()?;

// Encode calldata
let calldata = IMyContract::setPublicValueCall {
    value: U256::from(100),
}.abi_encode();

// Build a standard transaction (no .seismic())
let tx = seismic_foundry_tx_builder()
    .with_input(calldata.into())
    .with_kind(TxKind::Call(contract_address))
    .into();

// Send as standard transaction
let pending_tx = provider.send_transaction(tx).await?;
let receipt = pending_tx.get_receipt().await?;
println!("Transparent write confirmed: {:?}", receipt.transaction_hash);
```

## Transparent Read

A transparent read executes a standard `eth_call` with unencrypted calldata. Both `SeismicSignedProvider` and `SeismicUnsignedProvider` can perform transparent reads.

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::{Address, TxKind};
use alloy::sol_types::SolCall;

sol! {
    interface IMyContract {
        function getPublicValue() public view returns (uint256);
    }
}

let contract_address: Address = "0x1234...".parse()?;

// Encode calldata
let calldata = IMyContract::getPublicValueCall {}.abi_encode();

// Build a standard transaction request (no .seismic())
let tx = seismic_foundry_tx_builder()
    .with_input(calldata.into())
    .with_kind(TxKind::Call(contract_address))
    .into();

// Execute as standard eth_call
let result = provider.call(tx).await?;
println!("Result: {:?}", result);
```

### Using an Unsigned Provider

Transparent reads do not require a private key, so you can use `SeismicUnsignedProvider`:

```rust
use seismic_alloy::prelude::*;

// No private key needed
let url = "https://node.seismicdev.net/rpc".parse()?;
let provider = sreth_unsigned_provider(url);

// Transparent read works with unsigned provider
let calldata = IMyContract::getPublicValueCall {}.abi_encode();
let tx = seismic_foundry_tx_builder()
    .with_input(calldata.into())
    .with_kind(TxKind::Call(contract_address))
    .into();

let result = provider.call(tx).await?;
```

## Complete Example: Deploy and Interact

This example demonstrates the typical workflow: deploy a contract transparently, then use both shielded and transparent calls.

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::{Address, Bytes, U256, TxKind};
use alloy::sol_types::SolCall;
use alloy_signer_local::PrivateKeySigner;

sol! {
    interface ISeismicCounter {
        function setNumber(suint256 newNumber) public;
        function increment() public;
        function isOdd() public view returns (bool);
        function getPublicCount() public view returns (uint256);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up signed provider
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    // 1. Deploy contract (transparent -- Create txs cannot be seismic)
    let bytecode: Bytes = "0x6080604052...".parse()?;
    let deploy_tx = seismic_foundry_tx_builder()
        .with_input(bytecode)
        .with_kind(TxKind::Create)
        .into();

    let receipt = provider.send_transaction(deploy_tx).await?
        .get_receipt().await?;
    let contract_address = receipt.contract_address.unwrap();
    println!("Deployed to: {:?}", contract_address);

    // 2. Shielded write (encrypted calldata)
    let calldata = ISeismicCounter::setNumberCall {
        newNumber: U256::from(42),
    }.abi_encode();

    let shielded_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(calldata.into())
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();  // Encrypted

    provider.send_transaction(shielded_tx.into()).await?
        .get_receipt().await?;

    // 3. Transparent read (public data, no encryption needed)
    let calldata = ISeismicCounter::getPublicCountCall {}.abi_encode();
    let read_tx = seismic_foundry_tx_builder()
        .with_input(calldata.into())
        .with_kind(TxKind::Call(contract_address))
        .into();

    let result = provider.call(read_tx).await?;
    println!("Public count: {:?}", result);

    Ok(())
}
```

## Transparent vs. Shielded: The `.seismic()` Difference

The only API difference between transparent and shielded operations is the `.seismic()` call on the transaction builder:

```rust
// Transparent: no .seismic()
let transparent_tx = seismic_foundry_tx_builder()
    .with_input(calldata.into())
    .with_kind(TxKind::Call(address))
    .into();

// Shielded: with .seismic()
let shielded_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
    .with_input(calldata.into())
    .with_kind(TxKind::Call(address))
    .into()
    .seismic();
```

When `.seismic()` is omitted:

- No `TxSeismicElements` are attached
- No calldata encryption occurs
- The transaction is sent as a standard Ethereum type
- Any provider type can execute the operation

## See Also

- [Shielded Calls](shielded-calls.md) -- Encrypted contract interactions
- [Contract Interaction Overview](./) -- Comparison of all interaction patterns
- [SeismicUnsignedProvider](../provider/seismic-unsigned-provider.md) -- Read-only provider for transparent operations
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) -- Full-featured provider
