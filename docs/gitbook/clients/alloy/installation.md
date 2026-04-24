---
description: Cargo setup, dependencies, and prerequisites for seismic-alloy
icon: download
---

# Installation

## Prerequisites

| Requirement   | Version | Notes                                                      |
| ------------- | ------- | ---------------------------------------------------------- |
| Rust          | 1.82+   | Install via [rustup](https://rustup.rs/)                   |
| OpenSSL       | System  | Required by `seismic-enclave` for cryptographic operations |
| Tokio runtime | 1.x     | Async runtime (most Alloy operations are async)            |

## Add to Cargo.toml

`seismic-alloy` is a workspace of several crates, not a single crate. Depend on the specific members you need via git (the crates are not yet published to crates.io). For most applications, the short list is:

```toml
[dependencies]
seismic-prelude       = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-network = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-provider = { git = "https://github.com/SeismicSystems/seismic-alloy" }

alloy-provider    = "1.1"  # Provider trait (get_block_number, get_balance, ...)
alloy-signer-local = "1.1"  # PrivateKeySigner
alloy-primitives  = "1.1"  # Address, U256, aliases::SUInt, ...

tokio    = { version = "1", features = ["full"] }
reqwest  = "0.12"          # for reqwest::Url in the http connect calls
```

### Required `[patch.crates-io]` block

`seismic-alloy` depends on forked crates that are not on crates.io (`seismic-enclave`, `seismic-revm`, Seismic's forks of `alloy-primitives` / `alloy-sol-types` / `alloy-trie`). A git dependency does **not** inherit the parent workspace's patches, so your own `Cargo.toml` needs this block at the end:

```toml
[patch.crates-io]
seismic-enclave = { git = "https://github.com/SeismicSystems/enclave.git", rev = "f90b02f38a6190e8b2ff2d051d9043f3480cd3ac" }

# seismic-alloy-core
alloy-dyn-abi            = { git = "https://github.com/SeismicSystems/seismic-alloy-core.git", rev = "994f7b3fbc4582c2e06a00d03c7f3fe476b1b7c7" }
alloy-primitives         = { git = "https://github.com/SeismicSystems/seismic-alloy-core.git", rev = "994f7b3fbc4582c2e06a00d03c7f3fe476b1b7c7" }
alloy-json-abi           = { git = "https://github.com/SeismicSystems/seismic-alloy-core.git", rev = "994f7b3fbc4582c2e06a00d03c7f3fe476b1b7c7" }
alloy-sol-macro-expander = { git = "https://github.com/SeismicSystems/seismic-alloy-core.git", rev = "994f7b3fbc4582c2e06a00d03c7f3fe476b1b7c7" }
alloy-sol-macro-input    = { git = "https://github.com/SeismicSystems/seismic-alloy-core.git", rev = "994f7b3fbc4582c2e06a00d03c7f3fe476b1b7c7" }
alloy-sol-types          = { git = "https://github.com/SeismicSystems/seismic-alloy-core.git", rev = "994f7b3fbc4582c2e06a00d03c7f3fe476b1b7c7" }
alloy-sol-type-parser    = { git = "https://github.com/SeismicSystems/seismic-alloy-core.git", rev = "994f7b3fbc4582c2e06a00d03c7f3fe476b1b7c7" }

alloy-trie = { git = "https://github.com/SeismicSystems/seismic-trie.git", rev = "2787e2265d492fa8eaee00424585b8f7e522178f" }

alloy-consensus          = { git = "https://github.com/alloy-rs/alloy", tag = "v1.1.0" }
alloy-eips               = { git = "https://github.com/alloy-rs/alloy", tag = "v1.1.0" }
alloy-serde              = { git = "https://github.com/alloy-rs/alloy", tag = "v1.1.0" }
alloy-network-primitives = { git = "https://github.com/alloy-rs/alloy", tag = "v1.1.0" }
alloy-genesis            = { git = "https://github.com/alloy-rs/alloy", tag = "v1.1.0" }

revm         = { git = "https://github.com/SeismicSystems/seismic-revm.git", rev = "dc05eece19f14516ab9f996611fa7c63e1d1a8ae" }
seismic-revm = { git = "https://github.com/SeismicSystems/seismic-revm.git", rev = "dc05eece19f14516ab9f996611fa7c63e1d1a8ae" }
```

{% hint style="info" %}
Revs here track the tip of the `seismic` branch of `seismic-alloy`. For the authoritative set, copy the `[patch.crates-io]` block from [seismic-alloy's root Cargo.toml](https://github.com/SeismicSystems/seismic-alloy/blob/seismic/Cargo.toml).
{% endhint %}

### Pin to a specific revision

For reproducible builds, pin each git dependency to a specific commit:

```toml
seismic-prelude = { git = "https://github.com/SeismicSystems/seismic-alloy", rev = "abc1234" }
```

### Workspace Crates

The `seismic-alloy` workspace contains these crates:

| Crate                     | Description                                                           |
| ------------------------- | --------------------------------------------------------------------- |
| `seismic-alloy-provider`  | `SeismicProviderBuilder`, filler pipeline, precompile helpers         |
| `seismic-alloy-network`   | `SeismicNetwork` trait, `SeismicReth`, `SeismicFoundry` network types |
| `seismic-alloy-consensus` | Seismic transaction types and consensus logic                         |
| `seismic-alloy-rpc-types` | Seismic-specific RPC request and response types                       |
| `seismic-alloy-genesis`   | Genesis configuration types                                           |
| `seismic-prelude`         | Convenience re-exports (`seismic_prelude::client::*`) used by most app code |

Pull in only the ones you need. Most applications use `seismic-prelude`, `seismic-alloy-network`, and `seismic-alloy-provider`. Add `seismic-alloy-consensus` if you pattern-match on `SeismicReceiptEnvelope` or build `TxSeismic`/`TxSeismicElements` directly. Add `seismic-alloy-rpc-types` for `SeismicTransactionRequest`.

## Recommended Imports

The recommended approach is to use the client prelude, which re-exports the most commonly used types and traits:

```rust
use seismic_prelude::client::*;
```

This single import provides: `SeismicCallExt`, `ShieldedCallExt`, `SeismicProviderBuilder`, `SeismicProviderExt`, `SignedProviderExt`, `SeismicSignedProvider`, `SeismicUnsignedProvider`, `SeismicWallet`, `sol`, `Address`, `Bytes`, `FixedBytes`, `U256`, `ReceiptResponse`, and `PrivateKeySigner`.

For types not in the prelude (e.g., `SeismicReth`, `SeismicFoundry`, `Anvil`, `Filter`), add explicit imports alongside the prelude:

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;
```

{% hint style="info" %}
The prelude also has `seismic_prelude::foundry::*` and `seismic_prelude::reth::*` modules designed for Seismic's internal forks of Foundry and Reth. These re-export Seismic types under upstream naming conventions to minimize merge conflicts. **Do not use them in application code** — they pull in revm internals and aliases that are confusing outside of the fork context. Use `seismic_prelude::client::*` instead.
{% endhint %}

## Key Dependencies

`seismic-alloy` builds on these foundational crates:

| Dependency         | Version | Purpose                                                |
| ------------------ | ------- | ------------------------------------------------------ |
| `alloy`            | 1.1.0   | Core Ethereum toolkit (providers, signers, transports) |
| `alloy-primitives` | —      | With `"seismic"` feature flag enabled                  |
| `seismic-enclave`  | 0.1.0   | TEE key exchange and AES-GCM encryption                |
| `tokio`            | 1.x     | Async runtime                                          |
| `reqwest`          | —      | HTTP transport for RPC calls                           |

## Minimal Working Example

Create a new project:

```bash
cargo new my-seismic-app
cd my-seismic-app
```

Set up `Cargo.toml` (adding the `[patch.crates-io]` block shown above):

```toml
[package]
name = "my-seismic-app"
version = "0.1.0"
edition = "2021"
rust-version = "1.82"

[dependencies]
seismic-prelude        = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-network  = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-provider = { git = "https://github.com/SeismicSystems/seismic-alloy" }

alloy-provider     = "1.1"
alloy-signer-local = "1.1"
tokio              = { version = "1", features = ["full"] }
reqwest            = "0.12"

# [patch.crates-io] — copy the block from "Required [patch.crates-io] block" above.
```

Write `src/main.rs`:

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;
use alloy_provider::Provider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::<SeismicReth>::from(signer);
    let url: reqwest::Url = "https://testnet-1.seismictest.net/rpc".parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;

    let block_number = provider.get_block_number().await?;
    println!("Connected! Block number: {block_number}");

    Ok(())
}
```

Build and run:

```bash
cargo build
cargo run
```

## Troubleshooting

### OpenSSL Not Found

If you see linker errors related to OpenSSL:

```bash
# macOS
brew install openssl

# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config

# Fedora
sudo dnf install openssl-devel
```

### Rust Version Too Old

`seismic-alloy` requires Rust 1.82 or newer:

```bash
rustup update stable
rustc --version  # Should be >= 1.82.0
```

### Git Authentication

If the git dependency fails to fetch, ensure you have SSH or HTTPS access to the repository:

```bash
# Test SSH access
ssh -T git@github.com

# Or switch to HTTPS in your Cargo.toml deps
seismic-prelude = { git = "https://github.com/SeismicSystems/seismic-alloy.git" }
```

## Notes

- The workspace version is `0.0.1` — the API may change before a stable release
- The Rust edition is 2021
- `alloy-primitives` is used with the `"seismic"` feature flag, which adds Seismic-specific primitive types
- All provider operations are async and require a Tokio runtime

## See Also

- [Provider Overview](provider/) — Signed and unsigned provider types
- [SeismicSignedProvider](provider/seismic-signed-provider.md) — Full-featured provider
- [SeismicUnsignedProvider](provider/seismic-unsigned-provider.md) — Read-only provider
- [Encryption](provider/encryption.md) — How calldata encryption works
