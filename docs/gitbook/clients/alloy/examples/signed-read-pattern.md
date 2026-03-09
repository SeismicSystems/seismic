---
description: Authenticated read pattern with encrypted request and response
icon: magnifying-glass
---

# Signed Read Pattern

This example demonstrates the authenticated read pattern: create a signed provider, deploy a contract, write shielded data, read it back with a signed read, and compare with a transparent read to show the difference.

## Prerequisites

```bash
export PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
export RPC_URL="https://node.seismicdev.net/rpc"
```

`Cargo.toml`:

```toml
[package]
name = "signed-read-pattern"
version = "0.1.0"
edition = "2021"
rust-version = "1.82"

[dependencies]
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy" }
alloy-signer-local = "1.1"
alloy-primitives = "1.1"
hex-literal = "0.4"
tokio = { version = "1", features = ["full"] }
```

## Complete Example

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
        function setNumber(suint256 newNumber) public;
        function increment() public;
        function isOdd() public view returns (bool);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // -------------------------------------------------------
    // 1. Create signed provider
    // -------------------------------------------------------
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;

    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;
    println!("Provider ready. Block: {}", provider.get_block_number().await?);

    // -------------------------------------------------------
    // 2. Deploy contract (transparent)
    // -------------------------------------------------------
    let deploy_bytecode = Bytes::from_static(&hex!("6080604052..."));

    let deploy_tx = seismic_foundry_tx_builder()
        .with_input(deploy_bytecode)
        .with_kind(TxKind::Create)
        .into();

    let deploy_receipt = provider.send_transaction(deploy_tx).await?
        .get_receipt().await?;
    assert!(deploy_receipt.status());
    let contract_address = deploy_receipt.contract_address.unwrap();
    println!("Contract deployed at: {contract_address:?}");

    // -------------------------------------------------------
    // 3. Write data (shielded) -- setNumber(42)
    // -------------------------------------------------------
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
    println!("Shielded write confirmed (setNumber(42))");

    // -------------------------------------------------------
    // 4. Read it back (signed read) -- isOdd()
    // -------------------------------------------------------
    let call_input = ISeismicCounter::isOddCall {}.abi_encode();

    let signed_read_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(call_input.clone()))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    println!("\n--- Signed Read (seismic_call) ---");
    let signed_result = provider.seismic_call(
        SendableTx::Builder(signed_read_tx.into()),
    ).await?;

    let signed_decoded = ISeismicCounter::isOddReturn::abi_decode(&signed_result, true)?;
    println!("isOdd() via signed read: {}", signed_decoded._0);
    println!("  - msg.sender = your wallet address");
    println!("  - Calldata was encrypted");
    println!("  - Response was encrypted, then decrypted by provider");

    // -------------------------------------------------------
    // 5. Compare with transparent read -- provider.call()
    // -------------------------------------------------------
    let transparent_read_tx = seismic_foundry_tx_builder()
        .with_input(Bytes::from(call_input))
        .with_kind(TxKind::Call(contract_address))
        .into();
    // No .seismic() -- transparent read

    println!("\n--- Transparent Read (call) ---");
    let transparent_result = provider.call(transparent_read_tx).await?;

    let transparent_decoded = ISeismicCounter::isOddReturn::abi_decode(
        &transparent_result,
        true,
    )?;
    println!("isOdd() via transparent read: {}", transparent_decoded._0);
    println!("  - msg.sender = 0x0 (zero address)");
    println!("  - Calldata was plaintext");
    println!("  - Response was plaintext");

    // -------------------------------------------------------
    // 6. Show the difference
    // -------------------------------------------------------
    println!("\n--- Comparison ---");
    println!("Signed read result:      {}", signed_decoded._0);
    println!("Transparent read result:  {}", transparent_decoded._0);

    // For isOdd() which does not depend on msg.sender,
    // both results should be the same.
    // For functions that check msg.sender (e.g., balanceOf()),
    // the transparent read would return the zero address's data.
    if signed_decoded._0 == transparent_decoded._0 {
        println!("Results match -- isOdd() does not depend on msg.sender");
    } else {
        println!("Results differ -- the function depends on msg.sender");
    }

    Ok(())
}
```

## When Results Differ

The example above uses `isOdd()`, which does not depend on `msg.sender`. Both reads return the same value. To see a real difference, consider a contract where the view function uses `msg.sender`:

```rust
sol! {
    interface IPrivateBalance {
        // Uses msg.sender internally to look up caller's balance
        function balanceOf() public view returns (uint256);
    }
}

// Signed read: msg.sender = your address, returns YOUR balance
let signed_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
    .with_input(Bytes::from(IPrivateBalance::balanceOfCall {}.abi_encode()))
    .with_kind(TxKind::Call(contract_address))
    .into()
    .seismic();

let signed_result = provider.seismic_call(
    SendableTx::Builder(signed_tx.into()),
).await?;
let your_balance = IPrivateBalance::balanceOfReturn::abi_decode(&signed_result, true)?;
println!("Your balance: {}", your_balance._0);
// e.g., "Your balance: 1000"

// Transparent read: msg.sender = 0x0, returns zero address balance
let transparent_tx = seismic_foundry_tx_builder()
    .with_input(Bytes::from(IPrivateBalance::balanceOfCall {}.abi_encode()))
    .with_kind(TxKind::Call(contract_address))
    .into();

let transparent_result = provider.call(transparent_tx).await?;
let zero_balance = IPrivateBalance::balanceOfReturn::abi_decode(&transparent_result, true)?;
println!("Zero address balance: {}", zero_balance._0);
// e.g., "Zero address balance: 0"
```

## Key Differences at a Glance

| Aspect             | `seismic_call()` (Signed Read)                          | `call()` (Transparent Read)        |
| ------------------ | ------------------------------------------------------- | ---------------------------------- |
| Method             | `provider.seismic_call(SendableTx::Builder(tx.into()))` | `provider.call(tx)`                |
| `msg.sender`       | Your wallet address                                     | Zero address (`0x0`)               |
| Calldata           | Encrypted with AES-GCM                                  | Plaintext                          |
| Response           | Encrypted by TEE, decrypted by provider                 | Plaintext                          |
| Transaction marker | `.seismic()` required                                   | No `.seismic()`                    |
| Provider           | `SeismicSignedProvider` only                            | Any provider                       |
| Privacy            | Full (observers see nothing)                            | None (calldata and result visible) |

## Multiple Signed Reads

You can execute multiple signed reads in sequence:

```rust
// Read multiple values from the same contract
let functions: Vec<(&str, Vec<u8>)> = vec![
    ("isOdd", ISeismicCounter::isOddCall {}.abi_encode()),
    // Add more view functions as needed
];

for (name, calldata) in functions {
    let tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(calldata))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    let result = provider.seismic_call(SendableTx::Builder(tx.into())).await?;
    println!("{name}: result = {:?}", result);
}
```

## Expected Output

```
Provider ready. Block: 12345
Contract deployed at: 0x5FbDB2315678afecb367f032d93F642f64180aa3
Shielded write confirmed (setNumber(42))

--- Signed Read (seismic_call) ---
isOdd() via signed read: false
  - msg.sender = your wallet address
  - Calldata was encrypted
  - Response was encrypted, then decrypted by provider

--- Transparent Read (call) ---
isOdd() via transparent read: false
  - msg.sender = 0x0 (zero address)
  - Calldata was plaintext
  - Response was plaintext

--- Comparison ---
Signed read result:      false
Transparent read result:  false
Results match -- isOdd() does not depend on msg.sender
```

## Next Steps

- [Shielded Write Complete](shielded-write-complete.md) - Full shielded write lifecycle
- [Contract Deployment](contract-deployment.md) - Deploy and interact patterns
- [Basic Setup](basic-setup.md) - Provider setup details

## See Also

- [Signed Reads Guide](../guides/signed-reads.md) - Step-by-step guide
- [Shielded Calls](../contract-interaction/shielded-calls.md) - API reference
- [Transparent Calls](../contract-interaction/transparent-calls.md) - Non-encrypted operations
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Provider documentation
