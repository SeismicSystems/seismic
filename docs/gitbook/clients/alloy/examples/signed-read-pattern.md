---
description: Authenticated read pattern with encrypted request and response
icon: magnifying-glass
---

# Signed Read Pattern

This example demonstrates the authenticated read pattern: create a signed provider, deploy a contract, write shielded data, read it back with a signed read, and compare with a transparent read to show the difference.

## Prerequisites

```bash
export PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
export RPC_URL="https://testnet-1.seismictest.net/rpc"
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
alloy-sol-types = "1.1"
alloy-network = "1.1"
tokio = { version = "1", features = ["full"] }
```

## Complete Example

```rust
use seismic_alloy_network::{reth::SeismicReth, wallet::SeismicWallet};
use seismic_alloy_provider::{SeismicCallExt, SeismicProviderBuilder};
use alloy_network::ReceiptResponse;
use alloy_primitives::U256;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::sol;

sol! {
    #[sol(rpc, bytecode = "6080604052...")]
    contract SeismicCounter {
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
    let wallet = SeismicWallet::<SeismicReth>::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;
    println!("Provider ready. Block: {}", provider.get_block_number().await?);

    // -------------------------------------------------------
    // 2. Deploy contract
    // -------------------------------------------------------
    let contract = SeismicCounter::deploy(&provider).await?;
    println!("Contract deployed at: {:?}", contract.address());

    // -------------------------------------------------------
    // 3. Write data (shielded) -- setNumber(42)
    // -------------------------------------------------------
    let write_receipt = contract
        .setNumber(U256::from(42).into())
        .seismic()
        .send()
        .await?
        .get_receipt()
        .await?;
    assert!(write_receipt.status());
    println!("Shielded write confirmed (setNumber(42))");

    // -------------------------------------------------------
    // 4. Read it back (signed read) -- isOdd()
    // -------------------------------------------------------
    println!("\n--- Signed Read (.seismic().call()) ---");
    let signed_result = contract.isOdd().seismic().call().await?;
    println!("isOdd() via signed read: {signed_result}");
    println!("  - msg.sender = your wallet address");
    println!("  - Calldata was encrypted");
    println!("  - Response was encrypted, then decrypted by provider");

    // -------------------------------------------------------
    // 5. Compare with transparent read -- .call()
    // -------------------------------------------------------
    println!("\n--- Transparent Read (.call()) ---");
    let transparent_result = contract.isOdd().call().await?;
    println!("isOdd() via transparent read: {transparent_result}");
    println!("  - msg.sender = 0x0 (zero address)");
    println!("  - Calldata was plaintext");
    println!("  - Response was plaintext");

    // -------------------------------------------------------
    // 6. Show the difference
    // -------------------------------------------------------
    println!("\n--- Comparison ---");
    println!("Signed read result:      {signed_result}");
    println!("Transparent read result:  {transparent_result}");

    // For isOdd() which does not depend on msg.sender,
    // both results should be the same.
    // For functions that check msg.sender (e.g., balanceOf()),
    // the transparent read would return the zero address's data.
    if signed_result == transparent_result {
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
    #[sol(rpc)]
    contract PrivateBalance {
        // Uses msg.sender internally to look up caller's balance
        function balanceOf() public view returns (uint256);
    }
}

let contract = PrivateBalance::new(contract_address, &provider);

// Signed read: msg.sender = your address, returns YOUR balance
let your_balance = contract.balanceOf().seismic().call().await?;
println!("Your balance: {your_balance}");
// e.g., "Your balance: 1000"

// Transparent read: msg.sender = 0x0, returns zero address balance
let zero_balance = contract.balanceOf().call().await?;
println!("Zero address balance: {zero_balance}");
// e.g., "Zero address balance: 0"
```

## Key Differences at a Glance

| Aspect             | `.seismic().call()` (Signed Read)       | `.call()` (Transparent Read)       |
| ------------------ | --------------------------------------- | ---------------------------------- |
| Method             | `contract.method().seismic().call()`    | `contract.method().call()`         |
| `msg.sender`       | Your wallet address                     | Zero address (`0x0`)               |
| Calldata           | Encrypted with AES-GCM                  | Plaintext                          |
| Response           | Encrypted by TEE, decrypted by provider | Plaintext                          |
| Provider           | `SeismicSignedProvider` only            | Any provider                       |
| Privacy            | Full (observers see nothing)            | None (calldata and result visible) |

## Expected Output

```
Provider ready. Block: 12345
Contract deployed at: 0x5FbDB2315678afecb367f032d93F642f64180aa3
Shielded write confirmed (setNumber(42))

--- Signed Read (.seismic().call()) ---
isOdd() via signed read: false
  - msg.sender = your wallet address
  - Calldata was encrypted
  - Response was encrypted, then decrypted by provider

--- Transparent Read (.call()) ---
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
