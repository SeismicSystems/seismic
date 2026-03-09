---
description: Full shielded write lifecycle from deployment to receipt verification
icon: shield-halved
---

# Shielded Write Complete

This example demonstrates the full lifecycle of a shielded write: deploy a contract (transparent), send a shielded write (encrypted `setNumber`), verify the result with a signed read (encrypted `isOdd`), and inspect the receipt type.

## Prerequisites

```bash
# Set environment variables
export PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
export RPC_URL="https://node.seismicdev.net/rpc"
```

`Cargo.toml`:

```toml
[package]
name = "shielded-write-complete"
version = "0.1.0"
edition = "2021"
rust-version = "1.82"

[dependencies]
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-consensus = { git = "https://github.com/SeismicSystems/seismic-alloy" }
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
use seismic_alloy_consensus::SeismicReceiptEnvelope;

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
    // -------------------------------------------------------
    // 1. Set up signed provider
    // -------------------------------------------------------
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;

    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;
    println!("Provider created. Block: {}", provider.get_block_number().await?);

    // -------------------------------------------------------
    // 2. Deploy contract (transparent -- Create txs cannot be seismic)
    // -------------------------------------------------------
    // Replace with your compiled contract bytecode
    let deploy_bytecode = Bytes::from_static(&hex!("6080604052..."));

    let deploy_tx = seismic_foundry_tx_builder()
        .with_input(deploy_bytecode)
        .with_kind(TxKind::Create)
        .into();
    // No .seismic() -- deployment is always transparent

    println!("Deploying contract...");
    let deploy_receipt = provider.send_transaction(deploy_tx).await?
        .get_receipt().await?;

    assert!(deploy_receipt.status());
    let contract_address = deploy_receipt.contract_address
        .expect("deployment should return contract address");
    println!("Contract deployed at: {contract_address:?}");

    // -------------------------------------------------------
    // 3. Shielded write -- encrypted setNumber(42)
    // -------------------------------------------------------
    let calldata = ISeismicCounter::setNumberCall {
        newNumber: U256::from(42),
    }.abi_encode();

    let write_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(calldata))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();  // Marks for encryption

    println!("Sending shielded write (setNumber(42))...");
    let write_pending = provider.send_transaction(write_tx.into()).await?;
    let write_receipt = write_pending.get_receipt().await?;

    assert!(write_receipt.status());
    println!("Shielded write confirmed: {:?}", write_receipt.transaction_hash);
    println!("Block: {:?}", write_receipt.block_number());

    // -------------------------------------------------------
    // 4. Check receipt type -- should be SeismicReceiptEnvelope::Seismic
    // -------------------------------------------------------
    match write_receipt.inner.inner {
        SeismicReceiptEnvelope::Seismic(_r) => {
            println!("Receipt type: Seismic (0x4A) -- confirmed shielded transaction");
        }
        _ => {
            println!("Receipt type: Standard -- not a seismic transaction");
        }
    }

    // -------------------------------------------------------
    // 5. Signed read -- encrypted isOdd()
    // -------------------------------------------------------
    let call_input = ISeismicCounter::isOddCall {}.abi_encode();

    let read_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(call_input))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    println!("Executing signed read (isOdd())...");
    let result = provider.seismic_call(SendableTx::Builder(read_tx.into())).await?;

    // Decode the decrypted response
    let decoded = ISeismicCounter::isOddReturn::abi_decode(&result, true)?;
    println!("isOdd() = {}", decoded._0);

    // 42 is even, so isOdd should be false
    assert!(!decoded._0, "42 is even, isOdd() should return false");
    println!("Verification passed: 42 is even");

    // -------------------------------------------------------
    // 6. Shielded write -- increment to 43 (odd)
    // -------------------------------------------------------
    let inc_calldata = ISeismicCounter::incrementCall {}.abi_encode();

    let inc_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(inc_calldata))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    println!("Sending shielded write (increment())...");
    let inc_receipt = provider.send_transaction(inc_tx.into()).await?
        .get_receipt().await?;
    assert!(inc_receipt.status());

    // -------------------------------------------------------
    // 7. Verify with another signed read
    // -------------------------------------------------------
    let call_input_2 = ISeismicCounter::isOddCall {}.abi_encode();
    let read_tx_2: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(call_input_2))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    let result_2 = provider.seismic_call(SendableTx::Builder(read_tx_2.into())).await?;
    let decoded_2 = ISeismicCounter::isOddReturn::abi_decode(&result_2, true)?;
    println!("After increment: isOdd() = {}", decoded_2._0);

    // 43 is odd, so isOdd should be true
    assert!(decoded_2._0, "43 is odd, isOdd() should return true");
    println!("Verification passed: 43 is odd");

    println!("\nFull lifecycle complete!");
    Ok(())
}
```

## Step-by-Step Breakdown

### 1. Provider setup

The `SeismicSignedProvider` is created with a wallet and RPC URL. During construction, it generates an ephemeral secp256k1 keypair and fetches the TEE public key from the node. Both are cached for all subsequent operations.

### 2. Contract deployment (transparent)

Contract deployment uses `TxKind::Create` and does **not** call `.seismic()`. Create transactions cannot be seismic -- the Seismic protocol does not support encrypting deployment bytecode.

### 3. Shielded write

The `setNumber` call is encoded using the `sol!`-generated struct, wrapped in a `SeismicTransactionRequest` with `.seismic()`, and sent via `send_transaction()`. The filler pipeline automatically encrypts the calldata.

### 4. Receipt inspection

Seismic transactions produce a `SeismicReceiptEnvelope::Seismic` receipt. You can pattern-match on the receipt's inner envelope to confirm the transaction was processed as a shielded (type `0x4A`) transaction.

### 5. Signed read

The `isOdd()` view function is called via `seismic_call()`, which encrypts the calldata, sends a signed `eth_call`, and decrypts the response. The result is ABI-decoded into the return type.

## Expected Output

```
Provider created. Block: 12345
Deploying contract...
Contract deployed at: 0x5FbDB2315678afecb367f032d93F642f64180aa3
Sending shielded write (setNumber(42))...
Shielded write confirmed: 0xabc123...
Block: Some(12346)
Receipt type: Seismic (0x4A) -- confirmed shielded transaction
Executing signed read (isOdd())...
isOdd() = false
Verification passed: 42 is even
Sending shielded write (increment())...
After increment: isOdd() = true
Verification passed: 43 is odd

Full lifecycle complete!
```

## Error Handling Variant

For production code, wrap each step with error handling:

```rust
// Deploy with error handling
let deploy_receipt = provider.send_transaction(deploy_tx).await
    .map_err(|e| format!("failed to send deploy tx: {e}"))?
    .get_receipt().await
    .map_err(|e| format!("failed to get deploy receipt: {e}"))?;

if !deploy_receipt.status() {
    return Err("contract deployment reverted".into());
}

// Shielded write with error handling
let write_receipt = provider.send_transaction(write_tx.into()).await
    .map_err(|e| format!("failed to send shielded tx: {e}"))?
    .get_receipt().await
    .map_err(|e| format!("failed to get write receipt: {e}"))?;

if !write_receipt.status() {
    return Err("shielded write reverted".into());
}
```

## Next Steps

- [Signed Read Pattern](signed-read-pattern.md) - Detailed signed read examples
- [Contract Deployment](contract-deployment.md) - Deployment patterns with verification
- [Basic Setup](basic-setup.md) - Provider setup details

## See Also

- [Shielded Write Guide](../guides/shielded-write.md) - Step-by-step guide
- [Signed Reads Guide](../guides/signed-reads.md) - Signed read guide
- [Shielded Calls](../contract-interaction/shielded-calls.md) - API reference
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Provider documentation
