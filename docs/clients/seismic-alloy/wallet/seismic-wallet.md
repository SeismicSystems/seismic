---
description: Multi-signer wallet generic over SeismicNetwork
icon: key
---

# SeismicWallet

`SeismicWallet<N>` is the signing wallet used by `SeismicSignedProvider`. It holds one or more signers, tracks a default signer, and implements Alloy's `NetworkWallet<N>` trait so the filler pipeline can request transaction signatures.

## Overview

`SeismicWallet` is generic over `N: SeismicNetwork`, meaning the same wallet type works transparently with `SeismicReth` (production) and `SeismicFoundry` (local development). Internally, it stores signers in an address-indexed map and maintains a default signer address.

## Definition

```rust
pub struct SeismicWallet<N: SeismicNetwork>
where
    N::UnsignedTx: Send + Sync,
{
    default: Address,
    signers: AddressHashMap<Arc<dyn TxSigner<Signature> + Send + Sync>>,
    _network: std::marker::PhantomData<N>,
}
```

| Field      | Type                                                         | Description                              |
| ---------- | ------------------------------------------------------------ | ---------------------------------------- |
| `default`  | `Address`                                                    | Address of the default signer            |
| `signers`  | `AddressHashMap<Arc<dyn TxSigner<Signature> + Send + Sync>>` | Map of registered signers by address     |
| `_network` | `PhantomData<N>`                                             | Marker for the network type (zero-sized) |

## Creating a Wallet

### From a Single Signer

The most common pattern is to create a wallet from a `PrivateKeySigner`:

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::<SeismicReth>::from(signer);
```

The `From` implementation is available for any type that implements `TxSigner<Signature> + Send + Sync + 'static`. The provided signer becomes both the default and the only registered signer.

### With `new()`

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::<SeismicReth>::new(signer);
```

This is equivalent to using `From` -- the signer is registered and set as the default.

## Methods

### `new`

Create a new wallet with a single signer that becomes the default.

```rust
pub fn new(signer: impl TxSigner<Signature> + Send + Sync + 'static) -> Self
```

| Parameter | Type                                               | Required | Description                       |
| --------- | -------------------------------------------------- | -------- | --------------------------------- |
| `signer`  | `impl TxSigner<Signature> + Send + Sync + 'static` | Yes      | The signer to register as default |

**Returns:** A new `SeismicWallet` with the given signer.

### `register_signer`

Add an additional signer to the wallet without changing the default.

```rust
pub fn register_signer(&mut self, signer: impl TxSigner<Signature> + Send + Sync + 'static)
```

| Parameter | Type                                               | Required | Description       |
| --------- | -------------------------------------------------- | -------- | ----------------- |
| `signer`  | `impl TxSigner<Signature> + Send + Sync + 'static` | Yes      | The signer to add |

### `register_default_signer`

Add a signer and set it as the new default.

```rust
pub fn register_default_signer(&mut self, signer: impl TxSigner<Signature> + Send + Sync + 'static)
```

| Parameter | Type                                               | Required | Description                          |
| --------- | -------------------------------------------------- | -------- | ------------------------------------ |
| `signer`  | `impl TxSigner<Signature> + Send + Sync + 'static` | Yes      | The signer to add and set as default |

### `set_default_signer`

Change the default signer to an already-registered address. Panics if the address is not registered.

```rust
pub fn set_default_signer(&mut self, address: Address)
```

| Parameter | Type      | Required | Description                             |
| --------- | --------- | -------- | --------------------------------------- |
| `address` | `Address` | Yes      | Address of an already-registered signer |

{% hint style="info" %}
This method will panic if you pass an address that has not been registered with `register_signer` or `register_default_signer`. Always register the signer first.
{% endhint %}

### `default_signer`

Get an `Arc` reference to the default signer.

```rust
pub fn default_signer(&self) -> Arc<dyn TxSigner<Signature> + Send + Sync>
```

**Returns:** `Arc<dyn TxSigner<Signature> + Send + Sync>` -- The default signer.

### `signer_by_address`

Look up a signer by its address.

```rust
pub fn signer_by_address(
    &self,
    address: Address,
) -> Option<Arc<dyn TxSigner<Signature> + Send + Sync>>
```

| Parameter | Type      | Required | Description            |
| --------- | --------- | -------- | ---------------------- |
| `address` | `Address` | Yes      | The address to look up |

**Returns:** `Option<Arc<dyn TxSigner<Signature> + Send + Sync>>` -- The signer if registered, `None` otherwise.

## Trait Implementations

### `NetworkWallet<N>`

`SeismicWallet<N>` implements Alloy's `NetworkWallet<N>` trait, which is how the `WalletFiller` interacts with it during the filler pipeline:

- `default_signer_address()` -- returns the default signer's address
- `has_signer_for(address)` -- checks if a signer is registered for the given address
- `signer_addresses()` -- returns an iterator over all registered signer addresses

### `From<S> for SeismicWallet<N>`

Any type `S` that implements `TxSigner<Signature> + Send + Sync + 'static` can be converted into a `SeismicWallet` via `From`:

```rust
let wallet = SeismicWallet::<SeismicReth>::from(signer);
```

This is the most ergonomic way to create a wallet from a single signer.

## Examples

### Basic Wallet Creation

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::<SeismicReth>::from(signer);

// Use with a provider
let url = "https://node.seismicdev.net/rpc".parse()?;
let provider = SeismicSignedProvider::new(wallet, url).await?;
```

### Multi-Account Wallet

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let deployer: PrivateKeySigner = "0xDEPLOYER_KEY".parse()?;
let user: PrivateKeySigner = "0xUSER_KEY".parse()?;

let mut wallet = SeismicWallet::<SeismicReth>::new(deployer);
wallet.register_signer(user);

// The deployer is the default signer.
// Transactions without an explicit `from` will be signed by the deployer.
```

### Switching Default Signer

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let alice: PrivateKeySigner = "0xALICE_KEY".parse()?;
let bob: PrivateKeySigner = "0xBOB_KEY".parse()?;

let alice_addr = alice.address();
let bob_addr = bob.address();

let mut wallet = SeismicWallet::<SeismicReth>::new(alice);
wallet.register_signer(bob);

// Switch default to Bob
wallet.set_default_signer(bob_addr);
```

### Local Development with SeismicFoundry

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

// Well-known Anvil test key
let signer: PrivateKeySigner =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        .parse()?;

let wallet = SeismicWallet::<SeismicFoundry>::from(signer);

let url = "http://127.0.0.1:8545".parse()?;
let provider = sfoundry_signed_provider(wallet, url).await?;
```

## Notes

- `SeismicWallet` is generic over `N: SeismicNetwork` so the same API works for both production and testing networks
- Signers are stored as `Arc<dyn TxSigner<Signature>>`, allowing heterogeneous signer types in a single wallet
- The `PhantomData<N>` field ensures type safety -- a `SeismicWallet<SeismicReth>` cannot be used with a `SeismicSignedProvider<SeismicFoundry>`
- `set_default_signer` panics on unregistered addresses; check with `signer_by_address` first if unsure
- The wallet does not handle encryption -- that is the responsibility of `SeismicElementsFiller`

## See Also

- [Wallet Overview](./) - High-level wallet concepts
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Provider that uses the wallet
- [SeismicNetwork Trait](../network/seismic-network-trait.md) - The trait constraining the `N` parameter
- [SeismicElementsFiller](../fillers/seismic-elements-filler.md) - Encryption filler in the pipeline
- [Installation](../installation.md) - Cargo setup and prerequisites
