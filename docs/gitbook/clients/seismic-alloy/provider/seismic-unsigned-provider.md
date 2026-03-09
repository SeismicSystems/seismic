---
description: Read-only Seismic provider for public operations without a wallet
icon: globe
---

# SeismicUnsignedProvider

Read-only provider for public operations. No wallet or signing capabilities.

## Overview

`SeismicUnsignedProvider<N: SeismicNetwork>` is a lightweight provider for reading public data from Seismic nodes. It does not carry a wallet, cannot sign transactions, and cannot decrypt responses from `seismic_call`. Use it for:

- Querying block data, balances, and transaction receipts
- Fetching the TEE public key
- Public (unencrypted) contract reads
- WebSocket subscriptions

For full capabilities (shielded writes, signed reads, response decryption), use [SeismicSignedProvider](seismic-signed-provider.md) instead.

## Type Signature

```rust
pub struct SeismicUnsignedProvider<N: SeismicNetwork> {
    // inner provider with filler chain (no wallet filler)
}
```

The generic parameter `N` determines the network type:

- `SeismicReth` -- for Seismic devnet/testnet/mainnet
- `SeismicFoundry` -- for local sfoundry instances

## Constructors

### `new_http()`

Create an unsigned provider using HTTP transport (synchronous).

```rust
pub fn new_http(url: reqwest::Url) -> Self
```

#### Parameters

| Parameter | Type           | Required | Description                               |
| --------- | -------------- | -------- | ----------------------------------------- |
| `url`     | `reqwest::Url` | Yes      | HTTP URL of the Seismic node RPC endpoint |

#### Returns

| Type   | Description                       |
| ------ | --------------------------------- |
| `Self` | The constructed unsigned provider |

#### Example

```rust
use seismic_alloy::prelude::*;

let url = "https://node.seismicdev.net/rpc".parse()?;
let provider = SeismicUnsignedProvider::<SeismicReth>::new_http(url);

let block_number = provider.get_block_number().await?;
println!("Block number: {block_number}");
```

{% hint style="info" %}
`new_http()` is synchronous -- it does not make any RPC calls during construction. The TEE public key is not fetched or cached. This makes it suitable for quick, lightweight provider creation.
{% endhint %}

### `new_ws()`

Create an unsigned provider using WebSocket transport.

```rust
pub async fn new_ws(url: reqwest::Url) -> TransportResult<Self>
```

#### Parameters

| Parameter | Type           | Required | Description                                                              |
| --------- | -------------- | -------- | ------------------------------------------------------------------------ |
| `url`     | `reqwest::Url` | Yes      | WebSocket URL of the Seismic node (e.g., `wss://node.seismicdev.net/ws`) |

#### Returns

| Type                    | Description                                                                      |
| ----------------------- | -------------------------------------------------------------------------------- |
| `TransportResult<Self>` | The constructed provider, or a transport error if the WebSocket connection fails |

#### Example

```rust
use seismic_alloy::prelude::*;

let url = "wss://node.seismicdev.net/ws".parse()?;
let provider = SeismicUnsignedProvider::<SeismicReth>::new_ws(url).await?;

let block_number = provider.get_block_number().await?;
println!("Block number: {block_number}");
```

{% hint style="info" %}
`new_ws()` is async because it establishes the WebSocket connection during construction. Use WebSocket transport when you need persistent connections or event subscriptions.
{% endhint %}

## Convenience Functions

These functions pre-fill the network generic parameter:

### `sreth_unsigned_provider()`

```rust
pub fn sreth_unsigned_provider(url: reqwest::Url) -> SeismicUnsignedProvider<SeismicReth>
```

For Seismic devnet, testnet, or mainnet.

```rust
use seismic_alloy::prelude::*;

let url = "https://node.seismicdev.net/rpc".parse()?;
let provider = sreth_unsigned_provider(url);
```

### `sfoundry_unsigned_provider()`

```rust
pub fn sfoundry_unsigned_provider(url: reqwest::Url) -> SeismicUnsignedProvider<SeismicFoundry>
```

For local sfoundry development instances.

```rust
use seismic_alloy::prelude::*;

let url = "http://localhost:8545".parse()?;
let provider = sfoundry_unsigned_provider(url);
```

## Methods

`SeismicUnsignedProvider` implements `Deref` to the inner Alloy provider, so all standard `Provider<N>` methods are available. It also implements `SeismicProviderExt<N>`, though some methods have limited functionality compared to the signed provider.

### Via `SeismicProviderExt`

#### `get_tee_pubkey()`

Fetch the TEE public key from the node.

```rust
async fn get_tee_pubkey(&self) -> TransportResult<PublicKey>
```

| Returns                      | Description                         |
| ---------------------------- | ----------------------------------- |
| `TransportResult<PublicKey>` | The node's TEE secp256k1 public key |

```rust
use seismic_alloy::prelude::*;

let provider = sreth_unsigned_provider(url);
let tee_pubkey = provider.get_tee_pubkey().await?;
println!("TEE public key: {tee_pubkey}");
```

{% hint style="info" %}
Unlike `SeismicSignedProvider`, the unsigned provider does **not** cache the TEE pubkey. Each call to `get_tee_pubkey()` makes a fresh RPC request to `seismic_getTeePublicKey`.
{% endhint %}

#### `should_encrypt_input()`

Check whether a transaction's calldata should be encrypted.

```rust
fn should_encrypt_input<B: TransactionBuilder<N>>(&self, tx: &B) -> bool
```

| Parameter | Type                                    | Required | Description          |
| --------- | --------------------------------------- | -------- | -------------------- |
| `tx`      | `&B` (where `B: TransactionBuilder<N>`) | Yes      | Transaction to check |

| Returns | Description                                                     |
| ------- | --------------------------------------------------------------- |
| `bool`  | `true` if the transaction has calldata that should be encrypted |

#### `seismic_call()`

Send a call request. Since this is an unsigned provider, the response is **not** decrypted.

```rust
async fn seismic_call(&self, tx: SendableTx<N>) -> TransportResult<Bytes>
```

| Parameter | Type            | Required | Description                   |
| --------- | --------------- | -------- | ----------------------------- |
| `tx`      | `SendableTx<N>` | Yes      | Transaction to send as a call |

| Returns                  | Description                        |
| ------------------------ | ---------------------------------- |
| `TransportResult<Bytes>` | Raw (not decrypted) response bytes |

#### `call_conditionally_signed()`

Send a call without signing (since this is an unsigned provider).

```rust
async fn call_conditionally_signed(&self, tx: SendableTx<N>) -> TransportResult<Bytes>
```

### Via Standard Alloy `Provider`

All standard Alloy provider methods are available through `Deref`:

```rust
// Block queries
let block_number = provider.get_block_number().await?;
let block = provider.get_block_by_number(BlockNumberOrTag::Latest, false).await?;

// Account queries
let balance = provider.get_balance(address).await?;
let nonce = provider.get_transaction_count(address).await?;

// Transaction queries (read-only)
let receipt = provider.get_transaction_receipt(tx_hash).await?;
let tx = provider.get_transaction_by_hash(tx_hash).await?;

// Chain info
let chain_id = provider.get_chain_id().await?;
```

## Filler Chain

The unsigned provider uses a simplified filler chain without wallet signing:

| Order | Filler                  | Purpose                                                                 |
| ----- | ----------------------- | ----------------------------------------------------------------------- |
| 1     | `SeismicElementsFiller` | Populates Seismic-specific fields: encryption nonce, block hash, expiry |
| 2     | `NonceFiller`           | Fetches and sets the transaction nonce                                  |
| 2     | `ChainIdFiller`         | Sets the chain ID from the connected node                               |
| 3     | `SeismicGasFiller`      | Estimates and sets gas limit and gas price                              |

{% hint style="info" %}
The unsigned filler chain places `SeismicElementsFiller` first (before nonce/chain ID), whereas the signed filler chain places it after `WalletFiller` and nonce/chain ID. This ordering difference is because the signed provider needs the nonce to be set before computing Seismic elements that depend on it.
{% endhint %}

## Examples

### Query Block Data

```rust
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let block_number = provider.get_block_number().await?;
    println!("Current block: {block_number}");

    let chain_id = provider.get_chain_id().await?;
    println!("Chain ID: {chain_id}");

    Ok(())
}
```

### Check Account Balance

```rust
use seismic_alloy::prelude::*;
use alloy_primitives::address;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let addr = address!("0x1234567890abcdef1234567890abcdef12345678");
    let balance = provider.get_balance(addr).await?;
    println!("Balance: {balance}");

    Ok(())
}
```

### Fetch TEE Public Key

```rust
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let tee_pubkey = provider.get_tee_pubkey().await?;
    println!("TEE public key: {tee_pubkey}");

    // Pass this to SeismicSignedProvider::new_with_tee_pubkey()
    // for synchronous signed provider construction

    Ok(())
}
```

### WebSocket Connection

```rust
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://node.seismicdev.net/ws".parse()?;
    let provider = SeismicUnsignedProvider::<SeismicReth>::new_ws(url).await?;

    let block_number = provider.get_block_number().await?;
    println!("Block (via WebSocket): {block_number}");

    Ok(())
}
```

### Local sfoundry Development

```rust
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "http://localhost:8545".parse()?;
    let provider = sfoundry_unsigned_provider(url);

    let block = provider.get_block_number().await?;
    println!("Local sfoundry block: {block}");

    Ok(())
}
```

## Limitations

| Limitation                 | Details                                                 |
| -------------------------- | ------------------------------------------------------- |
| **No transaction signing** | Cannot send transactions -- use `SeismicSignedProvider` |
| **No response decryption** | `seismic_call()` returns raw bytes without decryption   |
| **No TEE pubkey caching**  | Each `get_tee_pubkey()` call makes a fresh RPC request  |
| **No calldata encryption** | Cannot encrypt calldata for shielded writes             |

## Notes

- `new_http()` is synchronous and makes no RPC calls at construction
- `new_ws()` is async because it establishes the WebSocket connection
- The unsigned provider is ideal for monitoring, indexing, and read-only applications
- All standard Alloy `Provider` methods work unchanged via `Deref`
- The unsigned provider is lighter weight than the signed provider (no wallet, no ephemeral keypair, no TEE pubkey cache)

## See Also

- [SeismicSignedProvider](seismic-signed-provider.md) -- Full-featured provider with wallet and encryption
- [Encryption](encryption.md) -- How calldata encryption works (signed provider only)
- [Provider Overview](./) -- Comparison of provider types
- [Installation](../installation.md) -- Add seismic-alloy to your project
