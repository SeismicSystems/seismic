---
description: Seismic wallet for transaction signing
icon: wallet
---

# Wallet

The `SeismicWallet` is the signing component of `seismic-alloy`. It manages one or more private key signers and provides the `NetworkWallet` implementation that the provider's filler pipeline uses to sign transactions.

## Overview

In Alloy's architecture, a wallet is responsible for:

1. Holding one or more signers (private keys, hardware wallets, etc.)
2. Providing a default signer address for transactions that do not specify `from`
3. Signing transactions when the `WalletFiller` requests it

`SeismicWallet` extends this pattern for Seismic networks. It is generic over `N: SeismicNetwork`, meaning the same wallet type works with both `SeismicReth` (production) and `SeismicFoundry` (local development).

## Quick Start

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

// Create a wallet from a single private key
let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::<SeismicReth>::from(signer);

// Use it to create a provider
let url = "https://node.seismicdev.net/rpc".parse()?;
let provider = SeismicSignedProvider::new(wallet, url).await?;
```

## Multi-Signer Support

`SeismicWallet` supports multiple signers. This is useful when your application needs to send transactions from different accounts:

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer_a: PrivateKeySigner = "0xKEY_A".parse()?;
let signer_b: PrivateKeySigner = "0xKEY_B".parse()?;

// Create wallet with signer_a as default
let mut wallet = SeismicWallet::<SeismicReth>::new(signer_a);

// Register an additional signer
wallet.register_signer(signer_b);

// Or register and set as new default
// wallet.register_default_signer(signer_b);
```

## Pages

| Page                               | Description                                                 |
| ---------------------------------- | ----------------------------------------------------------- |
| [SeismicWallet](seismic-wallet.md) | Full API reference with all methods, generics, and examples |

## See Also

- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Provider that uses the wallet
- [Network Overview](../network/) - Network types the wallet is generic over
- [Fillers](../fillers/) - The filler pipeline that invokes wallet signing
- [Installation](../installation.md) - Cargo setup and prerequisites
