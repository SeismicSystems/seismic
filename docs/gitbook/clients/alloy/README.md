---
description: Rust SDK for Seismic, built on Alloy
icon: rust
---

# seismic-alloy

Rust SDK for [Seismic](https://seismic.systems), built on [Alloy](https://github.com/alloy-rs/alloy) (v1.1.0). Requires **Rust 1.82+**.

```toml
[dependencies]
seismic-prelude        = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-network  = { git = "https://github.com/SeismicSystems/seismic-alloy" }
seismic-alloy-provider = { git = "https://github.com/SeismicSystems/seismic-alloy" }
alloy-provider         = "1.1"
# ... plus the [patch.crates-io] block documented in Installation
```

See [Installation](installation.md) for the full `Cargo.toml`, including the required `[patch.crates-io]` block.

## Quick Example

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;
use alloy_provider::Provider;

sol! {
    #[sol(rpc)]
    contract SeismicCounter {
        function setNumber(suint256 newNumber) public;
        function isOdd() public view returns (bool);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::<SeismicReth>::from(signer);
    let url: reqwest::Url = "https://testnet-1.seismictest.net/rpc".parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;

    let contract_address: Address = "0xYourContractAddress".parse()?;
    let contract = SeismicCounter::new(contract_address, &provider);

    // Shielded read (encrypted call + response decryption)
    let is_odd = contract.isOdd().seismic().call().await?;

    // Shielded write (encrypted transaction)
    // setNumber has a shielded param (suint256), so it auto-encrypts -- no .seismic() needed
    contract.setNumber(alloy_primitives::aliases::SUInt(U256::from(42)))
        .send()
        .await?
        .get_receipt()
        .await?;

    Ok(())
}
```

```rust
// Unsigned provider -- read-only (no private key needed)
// Note: connect_http is synchronous for unsigned providers (no .await)
let provider = SeismicProviderBuilder::new()
    .connect_http("https://testnet-1.seismictest.net/rpc".parse()?);

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
- **Builder** -> `SeismicProviderBuilder::new().wallet(wallet).connect_http(url)`
- **Encryption** -> [Encryption](provider/encryption.md)

## Features

- **Shielded Transactions** — Encrypt calldata with TEE public key via AES-GCM
- **Signed Reads** — Prove identity in `eth_call` with `seismic_call()`
- **Auto-Encryption for Shielded Params** — Functions with shielded types (`suint256`, `saddress`, etc.) in their arguments auto-encrypt via `ShieldedCallBuilder` — no `.seismic()` needed
- **`.seismic()` Call Builder** — `contract.method().seismic().call()` / `.send()` for non-shielded functions that need encryption
- **EIP-712 Support** — `.eip712()` on `ShieldedCallBuilder` for browser wallet compatibility (MetaMask)
- **SecurityParams** — Per-call `.expires_at()`, `.recent_block_hash()`, `.encryption_nonce()` overrides
- **Builder Pattern** — `SeismicProviderBuilder` with typestate for signed/unsigned HTTP/WS providers
- **Precompile Helpers** — Encode/decode/call wrappers for Seismic's 6 custom precompiles
- **Type 0x4A Transactions** — Native support for Seismic transaction type
- **Full Alloy Compatibility** — All standard Alloy `Provider` methods work unchanged

## Architecture

The SDK extends Alloy's provider model with a filler pipeline that automatically handles Seismic-specific transaction fields and encryption:

```
seismic-alloy (workspace)
├── consensus    -- Seismic transaction types and consensus logic
├── network      -- SeismicNetwork trait, SeismicReth, SeismicFoundry
├── provider     -- SeismicProviderBuilder, fillers, precompile helpers
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

SeismicUnsignedProvider filler chain:
  NonceFiller + ChainIdFiller
  -> GasFiller
```

## Network Types

The SDK defines two network configurations:

| Network          | Type        | Description                                |
| ---------------- | ----------- | ------------------------------------------ |
| `SeismicReth`    | Production  | Seismic devnet/testnet/mainnet             |
| `SeismicFoundry` | Development | Local Seismic Foundry (sanvil) instances   |

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

1. **[Install seismic-alloy](installation.md)** — Add the crate to your project
2. **[Create a signed provider](provider/seismic-signed-provider.md)** — Connect with full capabilities
3. **[Understand encryption](provider/encryption.md)** — Learn how calldata encryption works
4. **[Explore unsigned providers](provider/seismic-unsigned-provider.md)** — Read-only access
5. **[Follow a guide](guides/)** — Step-by-step shielded write or signed read tutorial
6. **[Run an example](examples/)** — Complete, runnable code examples
