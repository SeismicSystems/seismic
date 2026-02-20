---
description: Gas estimation filler with deferred support for seismic transactions
icon: gas-pump
---

# SeismicGasFiller

`SeismicGasFiller` handles gas estimation for both Seismic and standard transactions. For Seismic transactions, it defers gas estimation to the fill phase so that estimation happens on the encrypted calldata rather than the plaintext.

## Overview

Gas estimation for shielded transactions presents a unique challenge: the node needs to estimate gas on the encrypted calldata (because that is what will actually be executed), but the calldata is not encrypted until `SeismicElementsFiller` runs. `SeismicGasFiller` solves this by:

1. **For Seismic transactions** -- Deferring gas estimation to the `fill()` phase, which runs after `SeismicElementsFiller` has encrypted the calldata
2. **For standard transactions** -- Delegating to Alloy's built-in `GasFiller`, which estimates gas in the normal prepare phase

Additionally, `SeismicGasFiller` validates that Seismic transactions are not contract creation (CREATE) transactions, which are not supported for the Seismic transaction type.

## Definition

```rust
pub struct SeismicGasFiller {
    inner: GasFiller,
    rpc_url: Option<reqwest::Url>,
}
```

| Field     | Type                   | Description                                                      |
| --------- | ---------------------- | ---------------------------------------------------------------- |
| `inner`   | `GasFiller`            | Standard Alloy gas filler for non-seismic transactions           |
| `rpc_url` | `Option<reqwest::Url>` | RPC URL used for deferred gas estimation on seismic transactions |

## Constructor

### `with_url`

Create a `SeismicGasFiller` with an RPC URL for deferred gas estimation.

```rust
pub fn with_url(rpc_url: reqwest::Url) -> Self
```

| Parameter | Type           | Required | Description                           |
| --------- | -------------- | -------- | ------------------------------------- |
| `rpc_url` | `reqwest::Url` | Yes      | RPC endpoint for gas estimation calls |

**Returns:** A new `SeismicGasFiller` configured for deferred estimation.

{% hint style="info" %}
The RPC URL is needed because deferred gas estimation happens during the `fill()` phase, which makes a separate RPC call to `eth_estimateGas` with the encrypted calldata. This is typically the same URL used by the provider.
{% endhint %}

## Two-Phase Design

Like all Alloy fillers, `SeismicGasFiller` implements `TxFiller<N>` with prepare and fill phases.

### Prepare Phase

- **For Seismic transactions**: Returns immediately without estimating gas (deferred to fill phase)
- **For standard transactions**: Delegates to the inner `GasFiller`, which calls `eth_estimateGas` as normal

### Fill Phase

- **For Seismic transactions**: Now that the calldata is encrypted (by `SeismicElementsFiller`), makes an `eth_estimateGas` call using `rpc_url` and fills in the gas limit
- **For standard transactions**: Delegates to the inner `GasFiller` to apply the prepared gas estimate

## Validation

`SeismicGasFiller` enforces that Seismic transactions cannot be CREATE transactions (i.e., transactions without a `to` address that deploy new contracts). If a Seismic transaction request has no `to` field, the filler returns an error.

This restriction exists because the Seismic transaction type (`0x4A`) is designed for calling existing contracts with encrypted calldata, not for deploying new contracts.

## Usage

### Default (Automatic)

In most cases, `SeismicGasFiller` is configured automatically by the provider:

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "https://node.seismicdev.net/rpc".parse()?;

// SeismicGasFiller is set up automatically with the same URL
let provider = sreth_signed_provider(wallet, url).await?;
```

### Manual Construction

If building a custom provider stack:

```rust
use seismic_alloy::prelude::*;

let rpc_url: reqwest::Url = "https://node.seismicdev.net/rpc".parse()?;
let gas_filler = SeismicGasFiller::with_url(rpc_url);
```

## Pipeline Position

`SeismicGasFiller` must come after `SeismicElementsFiller` in the filler chain:

```
NonceFiller + ChainIdFiller
  -> SeismicElementsFiller  (encrypts calldata)
  -> SeismicGasFiller       (estimates gas on encrypted calldata)
  -> WalletFiller            (signs the transaction)
```

If `SeismicGasFiller` ran before `SeismicElementsFiller`, it would estimate gas on the plaintext calldata, which would produce an incorrect gas estimate.

## Notes

- For Seismic transactions, gas estimation happens on the encrypted calldata, which may differ slightly from the plaintext gas cost
- The `GasFiller` inner field handles all non-Seismic gas estimation logic
- Seismic CREATE transactions are explicitly rejected with an error
- The `rpc_url` field is `Option` to support cases where deferred estimation is not needed (e.g., when gas is pre-specified)

## See Also

- [Fillers Overview](./) - How fillers compose in the pipeline
- [SeismicElementsFiller](seismic-elements-filler.md) - Encryption filler (runs before this one)
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Provider that uses this filler
- [SeismicNetwork Trait](../network/seismic-network-trait.md) - Network trait methods used for type checking
