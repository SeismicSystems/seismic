---
description: CLAUDE.md template for Rust dapp development with seismic-alloy
icon: rust
---

# Seismic Alloy (Rust)

Use this template when your project uses `seismic-alloy` to interact with Seismic contracts from Rust. This SDK extends [Alloy](https://github.com/alloy-rs/alloy) with Seismic transaction types, encrypted calldata, and signed reads.

## The template

Copy the entire block below and save it as `CLAUDE.md` in your project root.

````markdown
# [Your Project Name]

## Seismic Overview

Seismic is an EVM-compatible L1 with on-chain privacy. Nodes run inside TEEs (Intel TDX). The Solidity compiler adds shielded types (`suint256`, `saddress`, `sbool`) that are invisible outside the TEE. Client libraries handle transaction encryption and signed reads automatically.

## Key Concepts

- **Shielded types**: `suint256`, `saddress`, `sbool` — on-chain private state, only readable via signed reads
- **TxSeismic (type 0x4A)**: Encrypts calldata before broadcast. The provider handles this automatically via the filler pipeline.
- **Signed reads**: `eth_call` zeroes `msg.sender` on Seismic. Use `seismic_call()` (via `SeismicProviderExt` trait) for reads that need a valid sender.
- **Encryption pubkeys**: 33-byte compressed secp256k1 keys. The provider fetches and manages these on construction.
- **Legacy gas**: Seismic transactions use `gas_price` + `gas_limit`, NOT EIP-1559.

## SDK: seismic-alloy

### Install

Add to `Cargo.toml`:

```toml
[dependencies]
seismic-alloy = { git = "https://github.com/SeismicSystems/seismic-alloy" }
alloy-signer-local = "1.1"
tokio = { version = "1", features = ["full"] }
```

Requires **Rust 1.82+** and OpenSSL.

### Key imports

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;
```

The prelude re-exports:

- `SeismicSignedProvider`, `SeismicUnsignedProvider`
- `SeismicWallet`
- `SeismicReth`, `SeismicFoundry`, `SeismicNetwork`
- `SeismicProviderExt` trait
- Convenience functions: `sreth_signed_provider()`, `sfoundry_signed_provider()`, `sreth_unsigned_provider()`, `sfoundry_unsigned_provider()`
- Seismic transaction types and RPC types

## Core Patterns

### Create a signed provider (full capabilities)

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "https://node-2.seismicdev.net/rpc".parse()?;

let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;
```

### Create an unsigned provider (read-only)

```rust
use seismic_alloy::prelude::*;

let url = "https://node-2.seismicdev.net/rpc".parse()?;
let provider = sreth_unsigned_provider(url);
```

### Convenience constructors

```rust
// For SeismicReth (devnet/testnet/mainnet)
let provider = sreth_signed_provider(wallet, url).await?;
let provider = sreth_unsigned_provider(url);

// For SeismicFoundry (local sanvil)
let provider = sfoundry_signed_provider(wallet, url).await?;
let provider = sfoundry_unsigned_provider(url);
```

### Contract interaction (shielded write)

```rust
use alloy::sol;

sol! {
    #[sol(rpc)]
    contract MyContract {
        function transfer(address to, uint256 amount) external;
        function getBalance(address account) external view returns (uint256);
    }
}

let contract = MyContract::new(contract_address, &provider);

// Shielded write — calldata is automatically encrypted
let tx_hash = contract.transfer(recipient, amount)
    .gas(500_000)
    .send()
    .await?
    .watch()
    .await?;
```

### Signed read

```rust
// Use seismic_call via SeismicProviderExt for reads that check msg.sender
use seismic_alloy::prelude::SeismicProviderExt;

let balance = provider.seismic_call(
    contract.getBalance(user_address)
).await?;
```

### Network types

| Network type     | Use with               | Description         |
| ---------------- | ---------------------- | ------------------- |
| `SeismicReth`    | Devnet/testnet/mainnet | Production networks |
| `SeismicFoundry` | Local `sanvil`         | Development network |

### Filler pipeline

The `SeismicSignedProvider` automatically applies fillers in order:

```
WalletFiller → NonceFiller + ChainIdFiller → SeismicElementsFiller → SeismicGasFiller → encrypt → send → decrypt
```

You do not need to manually encrypt calldata or set Seismic-specific transaction fields.

## Common Mistakes

1. **Using standard Alloy providers** — `ProviderBuilder::new()` creates a regular Ethereum provider. It won't encrypt calldata. Use `SeismicSignedProvider::new()` or the convenience constructors.
2. **Using EIP-1559 gas params** — Seismic uses legacy gas. Use `.gas(amount)`, not `.max_fee_per_gas()`.
3. **Using plain `eth_call` for shielded reads** — Standard calls zero the sender. Use `provider.seismic_call()` from the `SeismicProviderExt` trait.
4. **Wrong network type** — Use `SeismicReth` for devnet/testnet and `SeismicFoundry` for local `sanvil`. Using the wrong type causes deserialization errors.
5. **Forgetting `.await` on provider creation** — `SeismicSignedProvider::new()` is async (fetches TEE public key). Missing `.await` gives a future, not a provider.
6. **Missing OpenSSL** — The `seismic-enclave` crate requires OpenSSL. Install `libssl-dev` (Debian/Ubuntu) or `openssl` (macOS via brew).

## Networks

| Network        | Chain ID | RPC URL                             | Network type     |
| -------------- | -------- | ----------------------------------- | ---------------- |
| Devnet         | 5124     | `https://node-2.seismicdev.net/rpc` | `SeismicReth`    |
| Devnet (WS)    | 5124     | `wss://node-2.seismicdev.net/ws`    | `SeismicReth`    |
| Local (sanvil) | 31337    | `http://127.0.0.1:8545`             | `SeismicFoundry` |

Faucet: https://faucet-2.seismicdev.net/

## Links

- [seismic-alloy Installation](https://docs.seismic.systems/client-libraries/seismic-alloy/installation)
- [SeismicSignedProvider](https://docs.seismic.systems/client-libraries/seismic-alloy/provider/seismic-signed-provider)
- [SeismicUnsignedProvider](https://docs.seismic.systems/client-libraries/seismic-alloy/provider/seismic-unsigned-provider)
- [Shielded Write Guide](https://docs.seismic.systems/client-libraries/seismic-alloy/guides/shielded-write)
- [Signed Reads Guide](https://docs.seismic.systems/client-libraries/seismic-alloy/guides/signed-reads)
- [GitHub: seismic-alloy](https://github.com/SeismicSystems/seismic-alloy)
````

## What this teaches Claude

- **Correct crate names and import paths** — Claude will use `seismic_alloy::prelude::*` instead of guessing crate names
- **Provider construction** — Claude will use `SeismicSignedProvider::new()` with the correct generic network type
- **Signed reads** — Claude will use `provider.seismic_call()` from `SeismicProviderExt` instead of standard `eth_call`
- **Network type selection** — Claude will pick `SeismicReth` vs. `SeismicFoundry` based on the target network
- **Filler pipeline** — Claude understands that encryption is handled automatically

## Customizing

After pasting the template:

- Replace `[Your Project Name]` with your project name
- Add your `sol!` macro contract definitions
- Add deployed contract addresses
- If you pin to a specific revision, update the `Cargo.toml` example accordingly
