---
description: Network abstractions for Seismic chain interaction
icon: network-wired
---

# Network

The `seismic-alloy` SDK defines network types that configure how providers encode transactions, parse responses, and interact with Seismic nodes. Every provider is generic over a network type that implements the `SeismicNetwork` trait.

## Overview

Alloy's architecture uses a `Network` trait to associate transaction types, receipt types, and other chain-specific types into a single coherent set. Seismic extends this pattern with the `SeismicNetwork` trait, which adds methods for handling shielded transaction elements, calldata encryption, and Seismic-specific signing.

The SDK provides two concrete network implementations:

| Network                                | Use Case             | Description                                                           |
| -------------------------------------- | -------------------- | --------------------------------------------------------------------- |
| [`SeismicReth`](seismic-reth.md)       | Production / Testnet | Full Seismic node (reth-based). Use for devnet, testnet, and mainnet. |
| [`SeismicFoundry`](seismic-foundry.md) | Local Development    | Sanvil (Seismic Anvil). Use for local testing with `sanvil`.          |

Both implement `SeismicNetwork` and can be used as the generic parameter `N` in `SeismicSignedProvider<N>` and `SeismicUnsignedProvider<N>`.

## Core Abstraction

The [`SeismicNetwork`](seismic-network-trait.md) trait is the foundation of the network layer. It extends Alloy's `Network` and `RecommendedFillers` traits and defines how to:

- Attach and retrieve seismic encryption elements on transaction requests
- Read and write encrypted calldata (`input`) on requests and envelopes
- Sign transactions using a `SeismicWallet`
- Identify Seismic transaction types
- Extract Seismic metadata from signed envelopes

```rust
use seismic_alloy::prelude::*;

// SeismicReth for production
let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

// SeismicFoundry for local dev with Sanvil
let provider = SeismicSignedProvider::<SeismicFoundry>::new(wallet, url).await?;
```

## Choosing a Network Type

```
Are you connecting to a remote Seismic node (devnet/testnet/mainnet)?
  -> Use SeismicReth

Are you testing locally with sanvil?
  -> Use SeismicFoundry
```

{% hint style="info" %}
In most cases, you will use the convenience constructors (`sreth_signed_provider`, `sfoundry_signed_provider`) rather than specifying the network type directly. These functions select the correct network type for you.
{% endhint %}

## Quick Start

### Production / Testnet

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "https://node.seismicdev.net/rpc".parse()?;

// Convenience constructor (selects SeismicReth automatically)
let provider = sreth_signed_provider(wallet, url).await?;
```

### Local Development (Sanvil)

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "http://127.0.0.1:8545".parse()?;

// Convenience constructor (selects SeismicFoundry automatically)
let provider = sfoundry_signed_provider(wallet, url).await?;
```

## Associated Types

Both network types define the same set of associated types required by Alloy's `Network` trait, but with different concrete implementations:

| Associated Type       | SeismicReth                      | SeismicFoundry                          |
| --------------------- | -------------------------------- | --------------------------------------- |
| `TxType`              | `SeismicTxType`                  | `SeismicFoundryTxType`                  |
| `TxEnvelope`          | `SeismicTxEnvelope`              | `SeismicFoundryTxEnvelope`              |
| `UnsignedTx`          | `SeismicTypedTransaction`        | `SeismicFoundryTypedTransaction`        |
| `TransactionRequest`  | `SeismicTransactionRequest`      | `SeismicFoundryTransactionRequest`      |
| `ReceiptEnvelope`     | `SeismicReceiptEnvelope`         | `SeismicFoundryReceiptEnvelope`         |
| `TransactionResponse` | `Transaction<SeismicTxEnvelope>` | `Transaction<SeismicFoundryTxEnvelope>` |

## Pages

| Page                                             | Description                                        |
| ------------------------------------------------ | -------------------------------------------------- |
| [SeismicNetwork Trait](seismic-network-trait.md) | Core trait defining Seismic network behavior       |
| [SeismicReth](seismic-reth.md)                   | Production network type for devnet/testnet/mainnet |
| [SeismicFoundry](seismic-foundry.md)             | Development network type for Sanvil                |

## See Also

- [Provider Overview](../provider/) - Signed and unsigned provider types
- [Fillers](../fillers/) - Transaction filler pipeline
- [Wallet](../wallet/) - SeismicWallet documentation
- [Installation](../installation.md) - Cargo setup and prerequisites
