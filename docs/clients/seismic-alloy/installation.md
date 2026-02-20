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

`seismic-alloy` is not yet published to crates.io. Install it as a git dependency:

```toml
[dependencies]
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy" }
```

### Pin to a Specific Revision

For reproducible builds, pin to a specific commit:

```toml
[dependencies]
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy", rev = "abc1234" }
```

### Pin to a Branch

```toml
[dependencies]
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy", branch = "main" }
```

## Workspace Crates

The `seismic-alloy` workspace contains six crates. The `prelude` crate re-exports everything you need for most use cases:

| Crate                     | Description                                                           |
| ------------------------- | --------------------------------------------------------------------- |
| `seismic-alloy-consensus` | Seismic transaction types and consensus logic                         |
| `seismic-alloy-network`   | `SeismicNetwork` trait, `SeismicReth`, `SeismicFoundry` network types |
| `seismic-alloy-provider`  | `SeismicSignedProvider`, `SeismicUnsignedProvider`, filler pipeline   |
| `seismic-alloy-rpc-types` | Seismic-specific RPC request and response types                       |
| `seismic-alloy-genesis`   | Genesis configuration types                                           |
| `seismic-alloy-prelude`   | Convenience re-exports from all crates                                |

### Using Individual Crates

If you only need specific functionality, you can depend on individual crates:

```toml
[dependencies]
seismic-alloy-provider = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-network = { git = "https://github.com/SeismicSystems/seismic-alloy" }
```

## The Prelude

The recommended import pattern uses the `prelude` crate, which re-exports commonly used types:

```rust
use seismic_alloy::prelude::*;
```

This brings into scope:

- `SeismicSignedProvider`, `SeismicUnsignedProvider`
- `SeismicWallet`
- `SeismicReth`, `SeismicFoundry`, `SeismicNetwork`
- `SeismicProviderExt` trait
- Convenience functions: `sreth_signed_provider()`, `sfoundry_signed_provider()`, `sreth_unsigned_provider()`, `sfoundry_unsigned_provider()`
- Seismic transaction types and RPC types

## Key Dependencies

`seismic-alloy` builds on these foundational crates:

| Dependency         | Version | Purpose                                                |
| ------------------ | ------- | ------------------------------------------------------ |
| `alloy`            | 1.1.0   | Core Ethereum toolkit (providers, signers, transports) |
| `alloy-primitives` | --      | With `"seismic"` feature flag enabled                  |
| `seismic-enclave`  | 0.1.0   | TEE key exchange and AES-GCM encryption                |
| `tokio`            | 1.x     | Async runtime                                          |
| `reqwest`          | --      | HTTP transport for RPC calls                           |

## Additional Runtime Dependencies

You will also need a signer crate and an async runtime in your project:

```toml
[dependencies]
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy" }
alloy-signer-local = "1.1"  # For PrivateKeySigner
tokio = { version = "1", features = ["full"] }
```

## Minimal Working Example

Create a new project and verify the installation:

```bash
cargo new my-seismic-app
cd my-seismic-app
```

Set up `Cargo.toml`:

```toml
[package]
name = "my-seismic-app"
version = "0.1.0"
edition = "2021"
rust-version = "1.82"

[dependencies]
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy" }
alloy-signer-local = "1.1"
tokio = { version = "1", features = ["full"] }
```

Write `src/main.rs`:

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;

    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

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

# Or use HTTPS
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy.git" }
```

## Notes

- The workspace version is `0.0.1` -- the API may change before a stable release
- The Rust edition is 2021
- `alloy-primitives` is used with the `"seismic"` feature flag, which adds Seismic-specific primitive types
- All provider operations are async and require a Tokio runtime

## See Also

- [Provider Overview](provider/) -- Signed and unsigned provider types
- [SeismicSignedProvider](provider/seismic-signed-provider.md) -- Full-featured provider
- [SeismicUnsignedProvider](provider/seismic-unsigned-provider.md) -- Read-only provider
- [Encryption](provider/encryption.md) -- How calldata encryption works
