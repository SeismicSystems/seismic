---
description: Rust SDK for Seismic, built on Alloy
icon: rust
---

# seismic-alloy

Rust SDK for [Seismic](https://seismic.systems), built on [Alloy](https://github.com/alloy-rs/alloy) (v1.1.0). Requires **Rust 1.82+**.

```toml
[dependencies]
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy" }
```

## Quick Example

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;

    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    // All standard Alloy provider methods work
    let block_number = provider.get_block_number().await?;
    println!("Block number: {block_number}");

    Ok(())
}
```

```rust
// Unsigned provider -- read-only (no private key needed)
use seismic_alloy::prelude::*;

let url = "https://node.seismicdev.net/rpc".parse()?;
let provider = sreth_unsigned_provider(url);

let block = provider.get_block_number().await?;
```

## Documentation Navigation

### Getting Started

| Section                             | Description                                      |
| ----------------------------------- | ------------------------------------------------ |
| **[Installation](installation.md)** | Cargo setup, git dependencies, and feature flags |
| **[Provider](provider/)**           | Signed and unsigned provider types               |

### Provider Reference

| Section                                                              | Description                                                             |
| -------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| **[SeismicSignedProvider](provider/seismic-signed-provider.md)**     | Full-featured provider with wallet, encryption, and response decryption |
| **[SeismicUnsignedProvider](provider/seismic-unsigned-provider.md)** | Read-only provider for public operations                                |
| **[Encryption](provider/encryption.md)**                             | TEE key exchange, ECDH, AES-GCM calldata encryption                     |

## Quick Links

### By Task

- **Create a signed provider** -> [SeismicSignedProvider](provider/seismic-signed-provider.md)
- **Create a read-only provider** -> [SeismicUnsignedProvider](provider/seismic-unsigned-provider.md)
- **Understand calldata encryption** -> [Encryption](provider/encryption.md)
- **Install the crate** -> [Installation](installation.md)

### By Component

- **Provider types** -> [SeismicSignedProvider](provider/seismic-signed-provider.md), [SeismicUnsignedProvider](provider/seismic-unsigned-provider.md)
- **Encryption** -> [Encryption](provider/encryption.md)
- **Convenience constructors** -> `sreth_signed_provider()`, `sfoundry_signed_provider()`

## Features

- **Shielded Transactions** -- Encrypt calldata with TEE public key via AES-GCM
- **Signed Reads** -- Prove identity in `eth_call` with `seismic_call()`
- **Two Provider Types** -- `SeismicSignedProvider` (full capabilities) and `SeismicUnsignedProvider` (read-only)
- **Automatic Encryption Pipeline** -- Filler chain handles encryption transparently
- **Type 0x4A Transactions** -- Native support for Seismic transaction type
- **Full Alloy Compatibility** -- All standard Alloy `Provider` methods work unchanged

## Architecture

The SDK extends Alloy's provider model with a filler pipeline that automatically handles Seismic-specific transaction fields and encryption:

```
seismic-alloy (workspace)
├── consensus    -- Seismic transaction types and consensus logic
├── network      -- SeismicNetwork trait, SeismicReth, SeismicFoundry
├── provider     -- SeismicSignedProvider, SeismicUnsignedProvider, fillers
├── rpc-types    -- Seismic-specific RPC request/response types
├── genesis      -- Genesis configuration types
└── prelude      -- Convenience re-exports from all crates

SeismicSignedProvider filler chain:
  WalletFiller
  -> NonceFiller + ChainIdFiller
  -> SeismicElementsFiller
  -> SeismicGasFiller
  -> (encrypt calldata)
  -> send transaction
  -> (decrypt response)
```

## Network Types

The SDK defines two network configurations:

| Network          | Type        | Description                                |
| ---------------- | ----------- | ------------------------------------------ |
| `SeismicReth`    | Production  | Seismic devnet/testnet/mainnet             |
| `SeismicFoundry` | Development | Local Seismic Foundry (sfoundry) instances |

Both implement the `SeismicNetwork` trait and can be used as the generic parameter `N` in provider types.

### Guides & Examples

| Section                                                            | Description                                             |
| ------------------------------------------------------------------ | ------------------------------------------------------- |
| **[Guides](guides/)**                                              | Step-by-step tutorials for shielded writes and reads    |
| **[Shielded Write Guide](guides/shielded-write.md)**               | Encrypted transaction lifecycle and security parameters |
| **[Signed Reads Guide](guides/signed-reads.md)**                   | Encrypted eth_call with identity proof                  |
| **[Examples](examples/)**                                          | Complete, runnable code examples                        |
| **[Basic Setup](examples/basic-setup.md)**                         | Provider creation and connection verification           |
| **[Shielded Write Complete](examples/shielded-write-complete.md)** | Full lifecycle: deploy, write, read, inspect receipt    |
| **[Signed Read Pattern](examples/signed-read-pattern.md)**         | Comparing encrypted and transparent reads               |
| **[Contract Deployment](examples/contract-deployment.md)**         | Deploy, verify, interact, and subscribe to events       |

## Next Steps

1. **[Install seismic-alloy](installation.md)** -- Add the crate to your project
2. **[Create a signed provider](provider/seismic-signed-provider.md)** -- Connect with full capabilities
3. **[Understand encryption](provider/encryption.md)** -- Learn how calldata encryption works
4. **[Explore unsigned providers](provider/seismic-unsigned-provider.md)** -- Read-only access
5. **[Follow a guide](guides/)** -- Step-by-step shielded write or signed read tutorial
6. **[Run an example](examples/)** -- Complete, runnable code examples
