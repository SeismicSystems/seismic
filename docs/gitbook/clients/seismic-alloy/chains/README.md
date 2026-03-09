---
description: Chain configurations for Seismic networks
icon: link
---

# Chains

The `seismic-alloy` SDK supports multiple Seismic networks. Each chain has a specific chain ID, RPC endpoint, and recommended network type. This section documents the available chains and how to connect to them.

## Overview

Unlike the Python SDK, `seismic-alloy` does not use pre-configured chain objects. Instead, you pass the RPC URL directly to the provider constructor and select the appropriate network type (`SeismicReth` or `SeismicFoundry`). The chain ID is fetched automatically from the node.

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);

// Testnet
let url = "https://node.seismicdev.net/rpc".parse()?;
let provider = sreth_signed_provider(wallet, url).await?;

// Local (Sanvil)
let url = "http://127.0.0.1:8545".parse()?;
let provider = sfoundry_signed_provider(wallet, url).await?;
```

## Supported Chains

| Chain                                 | Chain ID | RPC URL                           | Network Type     | Description                                |
| ------------------------------------- | -------- | --------------------------------- | ---------------- | ------------------------------------------ |
| [Seismic Testnet](seismic-testnet.md) | `5124`   | `https://node.seismicdev.net/rpc` | `SeismicReth`    | Public testnet for development and testing |
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
use seismic_alloy::prelude::*;

let url = "https://node.seismicdev.net/rpc".parse()?;
let provider = sreth_unsigned_provider(url);

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
