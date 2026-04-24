---
description: Chain configurations for Seismic networks
icon: link
---

# Chains

The `seismic-alloy` SDK supports multiple Seismic networks. Each chain has a specific chain ID, RPC endpoint, and recommended network type. This section documents the available chains and how to connect to them.

## Overview

`seismic-alloy` does not use pre-configured chain objects. Pass the RPC URL directly to `SeismicProviderBuilder` and select the appropriate network type (`SeismicReth` or `SeismicFoundry`); the chain ID is fetched automatically from the node.

```rust
use seismic_prelude::client::*;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::<SeismicReth>::from(signer.clone());

// Testnet
let url = "https://testnet-1.seismictest.net/rpc".parse()?;
let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_http(url)
    .await?;

// Local (Sanvil)
let signer2: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet2 = SeismicWallet::<SeismicFoundry>::from(signer2);
let url = "http://127.0.0.1:8545".parse()?;
let provider = SeismicProviderBuilder::new()
    .foundry()
    .wallet(wallet2)
    .connect_http(url)
    .await?;
```

## Supported Chains

| Chain                                 | Chain ID | RPC URL                           | Network Type     | Description                                |
| ------------------------------------- | -------- | --------------------------------- | ---------------- | ------------------------------------------ |
| [Seismic Testnet](seismic-testnet.md) | `5124`   | `https://testnet-1.seismictest.net/rpc` | `SeismicReth`    | Public testnet for development and testing |
| [Sanvil (local)](sanvil.md)           | `31337`  | `http://127.0.0.1:8545`           | `SeismicFoundry` | Local development node                     |

## Choosing a Chain

```
Are you developing and want to test against a live network?
  -> Use Seismic Testnet with SeismicReth

Are you running tests locally or doing rapid iteration?
  -> Use Sanvil with SeismicFoundry
```

## Chain ID Handling

The Alloy provider automatically fetches the chain ID from the connected node via the `ChainIdFiller`. You do not need to specify it manually in most cases. The chain ID is used for:

- EIP-155 replay protection in transaction signing
- EIP-712 typed data signing
- Transaction validation

```rust
use seismic_prelude::client::*;

let url = "https://testnet-1.seismictest.net/rpc".parse()?;
// Unsigned provider — connect_http is synchronous
let provider = SeismicProviderBuilder::new()
    .connect_http(url);

// Chain ID is fetched from the node
let chain_id = provider.get_chain_id().await?;
println!("Chain ID: {chain_id}");  // 5124 for testnet
```

## Pages

| Page                                  | Description                            |
| ------------------------------------- | -------------------------------------- |
| [Seismic Testnet](seismic-testnet.md) | Public testnet configuration and usage |
| [Sanvil](sanvil.md)                   | Local development with Sanvil          |

## See Also

- [Network Overview](../network/) - Network types for each chain
- [SeismicReth](../network/seismic-reth.md) - Production network type
- [SeismicFoundry](../network/seismic-foundry.md) - Development network type
- [Installation](../installation.md) - Cargo setup and prerequisites
