---
description: Full shielded write lifecycle from deployment to receipt verification
icon: shield-halved
---

# Shielded Write Complete

This example demonstrates the full lifecycle of a shielded write: deploy a contract, send a shielded write (encrypted `setNumber`), verify the result with a signed read (encrypted `isOdd`), and inspect the receipt type.

## Prerequisites

```bash
# Set environment variables
export PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
export RPC_URL="https://testnet-1.seismictest.net/rpc"
```

`Cargo.toml` — see [Installation](../installation.md) for the full template including the required `[patch.crates-io]` block:

```toml
[package]
name = "shielded-write-complete"
version = "0.1.0"
edition = "2021"
rust-version = "1.82"

[dependencies]
seismic-prelude         = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-network   = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-provider  = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-consensus = { git = "https://github.com/SeismicSystems/seismic-alloy" }
alloy-provider          = "1.1"
alloy-signer-local      = "1.1"
alloy-primitives        = "1.1"
alloy-sol-types         = "1.1"
alloy-network           = "1.1"
tokio                   = { version = "1", features = ["full"] }
reqwest                 = "0.12"

# [patch.crates-io] block required — see Installation.
```

## Complete Example

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;
use alloy_network::ReceiptResponse;
use alloy_provider::Provider;
use seismic_alloy_consensus::SeismicReceiptEnvelope;

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
    // -------------------------------------------------------
    // 1. Set up signed provider
    // -------------------------------------------------------
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::<SeismicReth>::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;
    println!("Provider created. Block: {}", provider.get_block_number().await?);

    // -------------------------------------------------------
    // 2. Deploy contract (transparent -- Create txs cannot be seismic)
    // -------------------------------------------------------
    println!("Deploying contract...");
    let contract = SeismicCounter::deploy(&provider).await?;
    let contract_address = *contract.address();
    println!("Contract deployed at: {contract_address:?}");

    // -------------------------------------------------------
    // 3. Shielded write -- setNumber has suint256 param, auto-encrypts
    // -------------------------------------------------------
    println!("Sending shielded write (setNumber(42))...");
    let write_receipt = contract
        .setNumber(alloy_primitives::aliases::SUInt(U256::from(42)))
        .send()
        .await?
        .get_receipt()
        .await?;

    assert!(write_receipt.status());
    println!("Shielded write confirmed: {:?}", write_receipt.transaction_hash);
    println!("Block: {:?}", write_receipt.block_number());

    // -------------------------------------------------------
    // 4. Check receipt type -- should be SeismicReceiptEnvelope::Seismic
    // -------------------------------------------------------
    match write_receipt.inner {
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
    println!("Executing signed read (isOdd())...");
    let is_odd = contract.isOdd().seismic().call().await?;
    println!("isOdd() = {is_odd}");

    // 42 is even, so isOdd should be false
    assert!(!is_odd, "42 is even, isOdd() should return false");
    println!("Verification passed: 42 is even");

    // -------------------------------------------------------
    // 6. Shielded write -- increment to 43 (odd)
    // -------------------------------------------------------
    println!("Sending shielded write (increment())...");
    let inc_receipt = contract
        .increment()
        .seismic()
        .send()
        .await?
        .get_receipt()
        .await?;
    assert!(inc_receipt.status());

    // -------------------------------------------------------
    // 7. Verify with another signed read
    // -------------------------------------------------------
    let is_odd = contract.isOdd().seismic().call().await?;
    println!("After increment: isOdd() = {is_odd}");

    // 43 is odd, so isOdd should be true
    assert!(is_odd, "43 is odd, isOdd() should return true");
    println!("Verification passed: 43 is odd");

    println!("\nFull lifecycle complete!");
    Ok(())
}
```

## Step-by-Step Breakdown

### 1. Provider setup

The `SeismicProviderBuilder` creates a signed provider with a wallet and RPC URL. During construction, it generates a provider-scoped secp256k1 keypair and fetches the TEE public key from the node. Both are cached for all subsequent operations.

### 2. Contract deployment (transparent)

Contract deployment uses the generated `deploy()` method. Create transactions cannot be seismic — the Seismic protocol does not support encrypting deployment bytecode.

### 3. Shielded write

The `setNumber` call has a shielded parameter (`suint256`), so the `sol!` macro wraps it in a `ShieldedCallBuilder` that auto-encrypts. Just call `.send()` directly — no `.seismic()` needed. The filler pipeline encrypts the calldata before broadcast.

### 4. Receipt inspection

Seismic transactions produce a `SeismicReceiptEnvelope::Seismic` receipt. You can pattern-match on the receipt's inner envelope to confirm the transaction was processed as a shielded (type `0x4A`) transaction.

### 5. Signed read

The `isOdd()` view function is called via `.seismic().call()`, which encrypts the calldata, sends a signed `eth_call`, decrypts the response, and returns the decoded result.

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

## Next Steps

- [Signed Read Pattern](signed-read-pattern.md) - Detailed signed read examples
- [Contract Deployment](contract-deployment.md) - Deployment patterns with verification
- [Basic Setup](basic-setup.md) - Provider setup details

## See Also

- [Shielded Write Guide](../guides/shielded-write.md) - Step-by-step guide
- [Signed Reads Guide](../guides/signed-reads.md) - Signed read guide
- [Shielded Calls](../contract-interaction/shielded-calls.md) - API reference
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Provider documentation
