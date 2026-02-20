---
description: Transaction filler pipeline for Seismic-specific processing
icon: layer-group
---

# Fillers

Fillers are Alloy's middleware pattern for preparing transactions before they are sent. Each filler in the pipeline examines a transaction request, optionally fetches data (e.g., nonce, gas price, TEE public key), and fills in missing fields. The Seismic SDK adds two custom fillers to handle shielded transaction preparation.

## Overview

When you call a method that sends a transaction through `SeismicSignedProvider`, the transaction request passes through a chain of fillers before it reaches the network. Each filler has two phases:

1. **Prepare** -- Check what data is needed and fetch it (async). For example, fetch the TEE public key from the node.
2. **Fill** -- Apply the prepared data to the transaction request. For example, encrypt the calldata and attach seismic elements.

The full filler chain for `SeismicSignedProvider` is:

```
WalletFiller (signing)
  -> NonceFiller + ChainIdFiller (standard Alloy fillers)
  -> SeismicElementsFiller (encryption elements + calldata encryption)
  -> SeismicGasFiller (gas estimation, deferred for seismic txs)
```

{% hint style="info" %}
Execution order matters. `SeismicGasFiller` must run after `SeismicElementsFiller` because gas estimation for shielded transactions must happen on the encrypted calldata, not the plaintext.
{% endhint %}

## Seismic Fillers

| Filler                                                | Description                                                                                                                     |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| [`SeismicElementsFiller`](seismic-elements-filler.md) | Generates encryption nonce, fetches TEE public key, encrypts calldata, and attaches seismic elements to the transaction request |
| [`SeismicGasFiller`](seismic-gas-filler.md)           | Estimates gas for transactions. Defers estimation for seismic transactions until after encryption.                              |

## How the Pipeline Works

### For Seismic Transactions (type `0x4A`)

1. **NonceFiller** fetches and fills the sender's nonce
2. **ChainIdFiller** fills the chain ID
3. **SeismicElementsFiller**:
   - Fetches the TEE public key from the node (cached after first fetch)
   - Fetches the latest block number for expiration calculation
   - Generates an encryption nonce using ECDH with the ephemeral secret key
   - Encrypts the calldata using AES-GCM
   - Attaches `TxSeismicElements` to the transaction request
4. **SeismicGasFiller** estimates gas on the now-encrypted transaction
5. **WalletFiller** signs the complete transaction

### For Standard Transactions (legacy, EIP-1559, etc.)

1. **NonceFiller** and **ChainIdFiller** work as usual
2. **SeismicElementsFiller** skips (no seismic elements needed)
3. **SeismicGasFiller** delegates to the standard `GasFiller`
4. **WalletFiller** signs as normal

## Configuration

Both fillers are automatically configured when you create a `SeismicSignedProvider`. You do not need to instantiate them manually in most cases.

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "https://node.seismicdev.net/rpc".parse()?;

// Fillers are set up automatically
let provider = sreth_signed_provider(wallet, url).await?;
```

### Custom Configuration

If you need to customize filler behavior, you can construct them manually:

```rust
use seismic_alloy::prelude::*;

// Custom elements filler with cached TEE pubkey
let elements_filler = SeismicElementsFiller::with_tee_pubkey_and_url(tee_pubkey);

// Custom elements filler with signed reads enabled
let elements_filler = SeismicElementsFiller::new()
    .with_signed_read(true);

// Custom elements filler with a different expiration window
let elements_filler = SeismicElementsFiller::new()
    .with_blocks_window(200);

// Custom gas filler with RPC URL for deferred estimation
let gas_filler = SeismicGasFiller::with_url(rpc_url);
```

## Pages

| Page                                                | Description                                                   |
| --------------------------------------------------- | ------------------------------------------------------------- |
| [SeismicElementsFiller](seismic-elements-filler.md) | Encryption elements generation and calldata encryption        |
| [SeismicGasFiller](seismic-gas-filler.md)           | Gas estimation with deferred support for seismic transactions |

## See Also

- [Provider Overview](../provider/) - How fillers fit into the provider architecture
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Provider that uses the filler pipeline
- [Network Overview](../network/) - Network types that fillers operate on
- [Encryption](../provider/encryption.md) - Detailed encryption documentation
