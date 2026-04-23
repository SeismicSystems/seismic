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
| [Signed Read Pattern](signed-read-pattern.md)         | Authenticated read pattern with comparison between `.seismic().call()` (encrypted) and `.call()` (transparent)    |
| [Contract Deployment](contract-deployment.md)         | Deploy and interact with a shielded contract, including compilation notes, verification, and event subscription   |

## Example Template

Each example follows this structure:

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

sol! {
    #[sol(rpc)]
    contract MyContract {
        function myMethod(uint256 value) public;
        function myView() public view returns (bool);
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

    let contract_address: Address = std::env::var("CONTRACT_ADDRESS")?.parse()?;
    let contract = MyContract::new(contract_address, &provider);

    // 2. Shielded write
    contract.myMethod(U256::from(42))
        .seismic()
        .send()
        .await?
        .get_receipt()
        .await?;

    // 3. Signed read
    let result = contract.myView().seismic().call().await?;

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
export RPC_URL="https://testnet-1.seismictest.net/rpc"
```

And your `Cargo.toml` follows the layout documented in [Installation](../installation.md) -- `seismic-prelude`, `seismic-alloy-network`, `seismic-alloy-provider` from git plus `alloy-provider`, `alloy-signer-local`, `alloy-primitives`, `alloy-sol-types`, `tokio`, `reqwest`, and the required `[patch.crates-io]` block.

## See Also

- [Guides](../guides/) - Step-by-step tutorials
- [Contract Interaction](../contract-interaction/) - Shielded and transparent call patterns
- [Provider](../provider/) - Provider setup and configuration
- [Installation](../installation.md) - Full dependency setup
