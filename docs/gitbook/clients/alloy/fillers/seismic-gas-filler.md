---
description: Gas estimation filler with deferred support for seismic transactions
icon: gas-pump
---

# SeismicGasFiller

`SeismicGasFiller` handles gas estimation for both Seismic and standard transactions. For Seismic transactions with a wallet, it defers gas estimation to the fill phase and signs the transaction before calling `eth_estimateGas`, so the node can authenticate the sender and execute gas estimation against `msg.sender`-gated private state.

## Overview

Gas estimation for shielded transactions presents two challenges:

1. The node needs to estimate gas on the **encrypted calldata** (because that is what will actually be executed), but the calldata is not encrypted until `SeismicElementsFiller` runs.
2. When the contract's gas cost depends on private state gated by `msg.sender`, the node must be able to **authenticate the sender** during estimation — an anonymous `eth_estimateGas` would execute against a zero-address caller and produce an incorrect estimate.

`SeismicGasFiller` addresses both:

1. **For Seismic transactions with a wallet** — Defers gas estimation to the `fill()` phase, signs the transaction, and submits the signed envelope to `eth_estimateGas` over a fresh RPC client using the stored URL.
2. **For Seismic transactions without a wallet** — Falls back to Alloy's standard unsigned `GasFiller`. This is the default `RecommendedFillers` path for unsigned providers.
3. **For standard (non-Seismic) transactions** — Always delegates to the inner `GasFiller` for normal prepare-phase estimation.

Additionally, `SeismicGasFiller` rejects Seismic CREATE transactions (no `to` address), which are not supported for the Seismic transaction type.

## Definition

```rust
pub struct SeismicGasFiller<N: SeismicNetwork>
where
    N::UnsignedTx: Send + Sync,
{
    inner: GasFiller,
    rpc_url: Option<reqwest::Url>,
    wallet: Option<SeismicWallet<N>>,
}
```

| Field     | Type                      | Description                                                                    |
| --------- | ------------------------- | ------------------------------------------------------------------------------ |
| `inner`   | `GasFiller`               | Standard Alloy gas filler for non-seismic and unsigned fallback paths          |
| `rpc_url` | `Option<reqwest::Url>`    | RPC URL used for signed `eth_estimateGas` on seismic transactions              |
| `wallet`  | `Option<SeismicWallet<N>>` | Wallet used to sign the tx before submitting to `eth_estimateGas`              |

## Constructors

### `new`

Create a `SeismicGasFiller` with both an RPC URL and a wallet for signed gas estimation.

```rust
pub fn new(rpc_url: reqwest::Url, wallet: SeismicWallet<N>) -> Self
```

| Parameter | Type              | Required | Description                                        |
| --------- | ----------------- | -------- | -------------------------------------------------- |
| `rpc_url` | `reqwest::Url`    | Yes      | RPC endpoint for the `eth_estimateGas` call        |
| `wallet`  | `SeismicWallet<N>` | Yes      | Wallet used to sign the transaction before sending |

**Returns:** A new `SeismicGasFiller` configured for signed gas estimation on Seismic transactions.

### `default`

Create a `SeismicGasFiller` with no RPC URL and no wallet. Gas estimation falls back to the standard unsigned `GasFiller` path.

```rust
impl<N: SeismicNetwork> Default for SeismicGasFiller<N> { ... }
```

This is the variant used by the unsigned provider's `RecommendedFillers` default.

## Two-Phase Design

Like all Alloy fillers, `SeismicGasFiller` implements `TxFiller<N>` with prepare and fill phases.

### Prepare Phase

- **Seismic tx + wallet present**: Returns immediately without estimating gas (deferred to fill phase so the tx can be signed after `SeismicElementsFiller` runs)
- **Seismic tx without a wallet** or **non-Seismic tx**: Delegates to the inner `GasFiller`, which calls `eth_estimateGas` as normal

### Fill Phase

- **Seismic tx + wallet present**: The calldata is now encrypted (by `SeismicElementsFiller`). The filler signs the transaction with the wallet and submits the signed envelope to `eth_estimateGas` via the stored `rpc_url`, then fills in the gas limit.
- **Seismic tx without a wallet** or **non-Seismic tx**: Delegates to the inner `GasFiller` to apply the prepared gas estimate.

## Validation

`SeismicGasFiller` enforces that Seismic transactions cannot be CREATE transactions (i.e., transactions without a `to` address that deploy new contracts). If a Seismic transaction request has no `to` field, the filler returns an error.

This restriction exists because the Seismic transaction type (`0x4A`) is designed for calling existing contracts with encrypted calldata, not for deploying new contracts.

## Usage

### Default (Automatic)

In most cases, `SeismicGasFiller` is configured automatically by the provider:

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::<SeismicReth>::from(signer);
let url = "https://testnet-1.seismictest.net/rpc".parse()?;

// SeismicGasFiller is set up automatically with the same URL and wallet
let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_http(url)
    .await?;
```

### Manual Construction

If building a custom provider stack:

```rust
use seismic_alloy_provider::fillers::SeismicGasFiller;
use seismic_alloy_network::reth::SeismicReth;

let rpc_url: reqwest::Url = "https://testnet-1.seismictest.net/rpc".parse()?;
let gas_filler = SeismicGasFiller::<SeismicReth>::new(rpc_url, wallet);
```

## Pipeline Position

`SeismicGasFiller` is the last filler in the signed provider chain, running after `SeismicElementsFiller`:

```
WalletFiller
  -> NonceFiller + ChainIdFiller
  -> SeismicElementsFiller  (encrypts calldata)
  -> SeismicGasFiller       (signs encrypted tx and estimates gas)
```

If `SeismicGasFiller` ran before `SeismicElementsFiller`, it would estimate gas on the plaintext calldata, which would produce an incorrect gas estimate.

## Notes

- For Seismic transactions, gas estimation happens on the encrypted, signed calldata, which may differ from the plaintext gas cost.
- The signed gas-estimation path authenticates `msg.sender` to the node, so contracts that gate state on `msg.sender` return correct estimates.
- The `GasFiller` inner field handles all non-Seismic gas estimation logic and the unsigned Seismic fallback.
- Seismic CREATE transactions are explicitly rejected with an error.
- `rpc_url` and `wallet` are both `Option` to support the default / unsigned paths.

## See Also

- [Fillers Overview](./) - How fillers compose in the pipeline
- [SeismicElementsFiller](seismic-elements-filler.md) - Encryption filler (runs before this one)
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Provider that uses this filler
- [SeismicNetwork Trait](../network/seismic-network-trait.md) - Network trait methods used for type checking
