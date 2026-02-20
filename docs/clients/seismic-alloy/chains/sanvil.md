---
description: Local development with Sanvil (Seismic Anvil)
icon: flask
---

# Sanvil

Sanvil (Seismic Anvil) is a local development node for testing Seismic applications. It is Seismic's modified version of Foundry's Anvil, supporting the Seismic transaction type (`0x4A`) and TEE-related operations in a local environment.

## Configuration

| Property          | Value                              |
| ----------------- | ---------------------------------- |
| Chain ID          | `31337`                            |
| RPC URL           | `http://127.0.0.1:8545`            |
| Network Type      | `SeismicFoundry`                   |
| Transaction Types | Legacy, EIP-1559, Seismic (`0x4A`) |

## Installing Sanvil

Sanvil is part of the Seismic Foundry toolchain. Install it with:

```bash
# Clone the Seismic Foundry repository
git clone https://github.com/SeismicSystems/seismic-foundry.git
cd seismic-foundry

# Build sanvil
cargo build --release --bin sanvil

# The binary will be at target/release/sanvil
```

Alternatively, if you have `seismicup` installed:

```bash
seismicup
```

## Running Sanvil

Start a local Sanvil node:

```bash
sanvil
```

By default, Sanvil:

- Listens on `127.0.0.1:8545`
- Uses chain ID `31337`
- Pre-funds 10 test accounts with 10,000 ETH each
- Provides instant block mining

## Connecting

### Signed Provider (Full Capabilities)

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Well-known Anvil test account #0
    let signer: PrivateKeySigner =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "http://127.0.0.1:8545".parse()?;

    let provider = sfoundry_signed_provider(wallet, url).await?;

    let block = provider.get_block_number().await?;
    println!("Local block: {block}");

    Ok(())
}
```

### Unsigned Provider (Read-Only)

```rust
use seismic_alloy::prelude::*;

let url = "http://127.0.0.1:8545".parse()?;
let provider = sfoundry_unsigned_provider(url);

let block = provider.get_block_number().await?;
```

### With Explicit Type Parameter

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        .parse()?;
let wallet = SeismicWallet::from(signer);
let url = "http://127.0.0.1:8545".parse()?;

let provider = SeismicSignedProvider::<SeismicFoundry>::new(wallet, url).await?;
```

{% hint style="info" %}
Always use `SeismicFoundry` (not `SeismicReth`) when connecting to Sanvil. Sanvil uses Foundry-compatible transaction serialization that differs from production reth nodes.
{% endhint %}

## Examples

### Local Development Workflow

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use well-known test key
    let signer: PrivateKeySigner =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "http://127.0.0.1:8545".parse()?;

    let provider = sfoundry_signed_provider(wallet, url).await?;

    // All standard Alloy provider methods work
    let block = provider.get_block_number().await?;
    let chain_id = provider.get_chain_id().await?;
    println!("Sanvil running: chain {chain_id}, block {block}");

    Ok(())
}
```

### Testing with Multiple Accounts

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

// Anvil/Sanvil pre-funded test accounts
let test_keys = [
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
];

let url: reqwest::Url = "http://127.0.0.1:8545".parse()?;

for key in &test_keys {
    let signer: PrivateKeySigner = key.parse()?;
    let wallet = SeismicWallet::from(signer);
    let provider = sfoundry_signed_provider(wallet, url.clone()).await?;
    // Use each provider for different test accounts
}
```

### Integration Test Setup

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

async fn setup_test_provider()
    -> Result<impl std::ops::Deref, Box<dyn std::error::Error>>
{
    let signer: PrivateKeySigner =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "http://127.0.0.1:8545".parse()?;
    let provider = sfoundry_signed_provider(wallet, url).await?;
    Ok(provider)
}

#[tokio::test]
async fn test_connection() {
    let provider = setup_test_provider().await.unwrap();
    let chain_id = provider.get_chain_id().await.unwrap();
    assert_eq!(chain_id, 31337);
}
```

## Well-Known Test Accounts

Sanvil (like Anvil) pre-funds the following accounts with 10,000 ETH each:

```
Account #0: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
  Key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

Account #1: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
  Key: 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d

Account #2: 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
  Key: 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a
```

These keys are publicly known and must **never** be used on production networks.

## Notes

- Chain ID `31337` is the default Anvil/Foundry chain ID
- Sanvil supports the same Seismic transaction type (`0x4A`) as production nodes
- Blocks are mined instantly (no waiting for confirmation)
- Use `SeismicFoundry` as the network type, not `SeismicReth`
- No real value is at stake on the local network
- Sanvil automatically handles TEE key generation for local encryption testing

## See Also

- [Chains Overview](./) - All supported chains
- [Seismic Testnet](seismic-testnet.md) - Public testnet configuration
- [SeismicFoundry](../network/seismic-foundry.md) - Network type used with Sanvil
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Full-featured provider
- [Installation](../installation.md) - Cargo setup and prerequisites
