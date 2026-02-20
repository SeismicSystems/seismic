---
description: Complete runnable examples
icon: code
---

# Examples

Complete, runnable code examples demonstrating common Seismic Alloy (Rust) SDK patterns.

## Available Examples

### Getting Started

| Example                       | Description                                                                                                           |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| [Basic Setup](basic-setup.md) | Create providers, verify connection, check balance, and fetch the TEE public key -- both signed and unsigned variants |

### Core Workflows

| Example                                               | Description                                                                                                       |
| ----------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| [Shielded Write Complete](shielded-write-complete.md) | Full lifecycle: deploy a contract, send a shielded write, verify with a signed read, and inspect the receipt type |
| [Signed Read Pattern](signed-read-pattern.md)         | Authenticated read pattern with comparison between `seismic_call()` (encrypted) and `call()` (transparent)        |
| [Contract Deployment](contract-deployment.md)         | Deploy and interact with a shielded contract, including compilation notes, verification, and event subscription   |

## Example Template

Each example follows this structure:

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Set up provider
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    // 2. Build transaction
    let tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(calldata.into())
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    // 3. Send and verify
    let receipt = provider.send_transaction(tx.into()).await?
        .get_receipt().await?;
    assert!(receipt.status());

    Ok(())
}
```

## Running Examples

All examples assume you have:

```bash
# Install Rust 1.82+
rustup update stable

# Set environment variables
export PRIVATE_KEY="0x..."
export RPC_URL="https://node.seismicdev.net/rpc"
```

And your `Cargo.toml` includes:

```toml
[dependencies]
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy" }
alloy-signer-local = "1.1"
alloy-primitives = "1.1"
tokio = { version = "1", features = ["full"] }
```

## See Also

- [Guides](../guides/) - Step-by-step tutorials
- [Contract Interaction](../contract-interaction/) - Shielded and transparent call patterns
- [Provider](../provider/) - Provider setup and configuration
- [Installation](../installation.md) - Full dependency setup
