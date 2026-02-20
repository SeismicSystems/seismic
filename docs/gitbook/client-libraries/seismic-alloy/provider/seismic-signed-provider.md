---
description: Full-featured Seismic provider with wallet integration, encryption, and response decryption
icon: lock
---

# SeismicSignedProvider

Full-featured provider with wallet integration, automatic calldata encryption, and response decryption.

## Overview

`SeismicSignedProvider<N: SeismicNetwork>` is the primary provider type for interacting with Seismic nodes. It wraps an Alloy provider with a filler chain that automatically:

1. Signs transactions with the attached wallet
2. Populates nonce, chain ID, and Seismic-specific fields
3. Encrypts calldata using AES-GCM with an ECDH shared secret
4. Decrypts `seismic_call` responses using the same shared secret

At creation time, the provider generates an ephemeral secp256k1 keypair, fetches the TEE public key from the node, and caches it for all subsequent operations.

## Type Signature

```rust
pub struct SeismicSignedProvider<N: SeismicNetwork> {
    // inner provider with filler chain
    // ephemeral_secret_key: secp256k1 secret key
    // tee_pubkey: cached TEE public key
}
```

The generic parameter `N` determines the network type:

- `SeismicReth` -- for Seismic devnet/testnet/mainnet
- `SeismicFoundry` -- for local sfoundry instances

## Constructors

### `new()`

Create a signed provider by fetching the TEE public key from the node.

```rust
pub async fn new(
    wallet: impl Into<SeismicWallet<N>>,
    url: reqwest::Url,
) -> TransportResult<Self>
```

#### Parameters

| Parameter | Type                          | Required | Description                                                                                                     |
| --------- | ----------------------------- | -------- | --------------------------------------------------------------------------------------------------------------- |
| `wallet`  | `impl Into<SeismicWallet<N>>` | Yes      | Wallet for signing transactions. Accepts any type that converts into `SeismicWallet` (e.g., `PrivateKeySigner`) |
| `url`     | `reqwest::Url`                | Yes      | HTTP URL of the Seismic node RPC endpoint                                                                       |

#### Returns

| Type                    | Description                                                                  |
| ----------------------- | ---------------------------------------------------------------------------- |
| `TransportResult<Self>` | The constructed provider, or a transport error if the TEE pubkey fetch fails |

#### Example

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "https://node.seismicdev.net/rpc".parse()?;

let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;
```

{% hint style="info" %}
`new()` is async because it makes an RPC call to `seismic_getTeePublicKey` to fetch and cache the node's TEE public key. If you already have the TEE pubkey, use `new_with_tee_pubkey()` for synchronous construction.
{% endhint %}

### `new_with_tee_pubkey()`

Create a signed provider with a pre-fetched TEE public key (synchronous).

```rust
pub fn new_with_tee_pubkey(
    wallet: impl Into<SeismicWallet<N>>,
    url: reqwest::Url,
    tee_pubkey: PublicKey,
) -> Self
```

#### Parameters

| Parameter    | Type                          | Required | Description                               |
| ------------ | ----------------------------- | -------- | ----------------------------------------- |
| `wallet`     | `impl Into<SeismicWallet<N>>` | Yes      | Wallet for signing transactions           |
| `url`        | `reqwest::Url`                | Yes      | HTTP URL of the Seismic node RPC endpoint |
| `tee_pubkey` | `PublicKey`                   | Yes      | Pre-fetched TEE public key (secp256k1)    |

#### Returns

| Type   | Description              |
| ------ | ------------------------ |
| `Self` | The constructed provider |

#### Example

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

// Fetch TEE pubkey separately (e.g., from config or another provider)
let unsigned = sreth_unsigned_provider(url.clone());
let tee_pubkey = unsigned.get_tee_pubkey().await?;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);

// Synchronous construction with pre-fetched key
let provider = SeismicSignedProvider::<SeismicReth>::new_with_tee_pubkey(
    wallet,
    url,
    tee_pubkey,
);
```

## Convenience Functions

These functions pre-fill the network generic parameter for common network types:

### `sreth_signed_provider()`

```rust
pub async fn sreth_signed_provider(
    wallet: impl Into<SeismicWallet<SeismicReth>>,
    url: reqwest::Url,
) -> TransportResult<SeismicSignedProvider<SeismicReth>>
```

For Seismic devnet, testnet, or mainnet.

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "https://node.seismicdev.net/rpc".parse()?;

let provider = sreth_signed_provider(wallet, url).await?;
```

### `sfoundry_signed_provider()`

```rust
pub async fn sfoundry_signed_provider(
    wallet: impl Into<SeismicWallet<SeismicFoundry>>,
    url: reqwest::Url,
) -> TransportResult<SeismicSignedProvider<SeismicFoundry>>
```

For local sfoundry development instances.

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "http://localhost:8545".parse()?;

let provider = sfoundry_signed_provider(wallet, url).await?;
```

## Methods

`SeismicSignedProvider` implements `Deref` to the inner Alloy provider, so all standard `Provider<N>` methods are available directly. It also implements `SeismicProviderExt<N>` for Seismic-specific operations.

### Via `SeismicProviderExt`

#### `seismic_call()`

Fill, encrypt, send, and decrypt a call request. This is the primary method for signed reads.

```rust
async fn seismic_call(&self, tx: SendableTx<N>) -> TransportResult<Bytes>
```

| Parameter | Type            | Required | Description                                                             |
| --------- | --------------- | -------- | ----------------------------------------------------------------------- |
| `tx`      | `SendableTx<N>` | Yes      | Transaction to send as a call. Calldata will be encrypted automatically |

| Returns                  | Description                                    |
| ------------------------ | ---------------------------------------------- |
| `TransportResult<Bytes>` | Decrypted response bytes, or a transport error |

```rust
use seismic_alloy::prelude::*;
use alloy_primitives::{address, bytes};

let provider = sreth_signed_provider(wallet, url).await?;

// Build a call request
let tx = TransactionRequest::default()
    .to(address!("0x1234567890abcdef1234567890abcdef12345678"))
    .input(bytes!("0x12345678").into());

// seismic_call encrypts calldata, sends, and decrypts the response
let result = provider.seismic_call(tx.into()).await?;
println!("Decrypted result: {result}");
```

#### `get_tee_pubkey()`

Fetch the TEE public key from the node.

```rust
async fn get_tee_pubkey(&self) -> TransportResult<PublicKey>
```

| Returns                      | Description                         |
| ---------------------------- | ----------------------------------- |
| `TransportResult<PublicKey>` | The node's TEE secp256k1 public key |

{% hint style="info" %}
For `SeismicSignedProvider`, the TEE pubkey is fetched once at construction and cached. This method is still available but typically not needed -- the cached key is used automatically for all encryption operations.
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

#### `call_conditionally_signed()`

Send a call that is signed (since this is a signed provider).

```rust
async fn call_conditionally_signed(&self, tx: SendableTx<N>) -> TransportResult<Bytes>
```

| Parameter | Type            | Required | Description         |
| --------- | --------------- | -------- | ------------------- |
| `tx`      | `SendableTx<N>` | Yes      | Transaction to send |

| Returns                  | Description    |
| ------------------------ | -------------- |
| `TransportResult<Bytes>` | Response bytes |

### Via Standard Alloy `Provider`

All standard Alloy provider methods are available through `Deref`:

```rust
// Block queries
let block_number = provider.get_block_number().await?;
let block = provider.get_block_by_number(BlockNumberOrTag::Latest, false).await?;

// Account queries
let balance = provider.get_balance(address).await?;
let nonce = provider.get_transaction_count(address).await?;

// Transaction operations
let tx_hash = provider.send_transaction(tx).await?.watch().await?;
let receipt = provider.get_transaction_receipt(tx_hash).await?;

// Chain info
let chain_id = provider.get_chain_id().await?;
```

## Filler Chain

The signed provider assembles its filler chain in this order:

| Order | Filler                  | Purpose                                                                                                    |
| ----- | ----------------------- | ---------------------------------------------------------------------------------------------------------- |
| 1     | `WalletFiller`          | Signs the transaction with the attached wallet                                                             |
| 2     | `NonceFiller`           | Fetches and sets the transaction nonce                                                                     |
| 2     | `ChainIdFiller`         | Sets the chain ID from the connected node                                                                  |
| 3     | `SeismicElementsFiller` | Populates Seismic-specific fields: encryption nonce, TEE pubkey reference, recent block hash, expiry block |
| 4     | `SeismicGasFiller`      | Estimates and sets gas limit and gas price                                                                 |

After all fillers run, calldata is encrypted with AES-GCM before the transaction is sent to the node.

## Examples

### Shielded Write

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;
use alloy_primitives::{address, U256};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;

    let provider = sreth_signed_provider(wallet, url).await?;

    // Build a shielded write transaction
    // The filler pipeline automatically:
    //   1. Populates nonce, chain_id, seismic elements
    //   2. Encrypts the calldata
    //   3. Signs the transaction
    let tx = TransactionRequest::default()
        .to(address!("0x1234567890abcdef1234567890abcdef12345678"))
        .input(bytes!("0x60fe47b10000000000000000000000000000000000000000000000000000000000000042").into())
        .value(U256::ZERO);

    let pending = provider.send_transaction(tx).await?;
    let tx_hash = pending.watch().await?;
    println!("Transaction hash: {tx_hash}");

    Ok(())
}
```

### Signed Read

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;
use alloy_primitives::address;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;

    let provider = sreth_signed_provider(wallet, url).await?;

    // Build a read request
    let tx = TransactionRequest::default()
        .to(address!("0x1234567890abcdef1234567890abcdef12345678"))
        .input(bytes!("0x3fb5c1cb").into());

    // seismic_call: encrypts calldata, sends signed call, decrypts response
    let result = provider.seismic_call(tx.into()).await?;
    println!("Decrypted value: {result}");

    Ok(())
}
```

### Local Development with sfoundry

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Default sfoundry private key
    let signer: PrivateKeySigner = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "http://localhost:8545".parse()?;

    // Use SeismicFoundry network type for local development
    let provider = sfoundry_signed_provider(wallet, url).await?;

    let block = provider.get_block_number().await?;
    println!("Local block: {block}");

    Ok(())
}
```

### Pre-fetched TEE Pubkey

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url: reqwest::Url = "https://node.seismicdev.net/rpc".parse()?;

    // Fetch TEE pubkey once
    let unsigned = sreth_unsigned_provider(url.clone());
    let tee_pubkey = unsigned.get_tee_pubkey().await?;

    // Create multiple signed providers without additional RPC calls
    let signer1: PrivateKeySigner = "0xKEY1".parse()?;
    let provider1 = SeismicSignedProvider::<SeismicReth>::new_with_tee_pubkey(
        SeismicWallet::from(signer1),
        url.clone(),
        tee_pubkey,
    );

    let signer2: PrivateKeySigner = "0xKEY2".parse()?;
    let provider2 = SeismicSignedProvider::<SeismicReth>::new_with_tee_pubkey(
        SeismicWallet::from(signer2),
        url,
        tee_pubkey,
    );

    Ok(())
}
```

## How It Works

1. **Construction** -- Generates an ephemeral secp256k1 keypair and fetches (or accepts) the TEE public key. Computes the ECDH shared secret between the ephemeral key and the TEE pubkey.

2. **Transaction building** -- The filler chain populates all transaction fields: nonce, chain ID, Seismic elements (encryption nonce, block hash, expiry), and gas.

3. **Encryption** -- Before sending, calldata is encrypted using AES-GCM. The encryption key is derived from the ECDH shared secret. Additional Authenticated Data (AAD) binds the ciphertext to the transaction context.

4. **Sending** -- The encrypted transaction is signed by the wallet and sent to the node.

5. **Response decryption** -- For `seismic_call()` requests, the response is decrypted using the same ECDH shared secret.

## Notes

- The ephemeral keypair is generated once at provider creation and reused for all operations
- The TEE public key is cached after the initial fetch
- All standard Alloy `Provider` methods work unchanged -- only transactions with calldata are encrypted
- `SeismicSignedProvider` implements `Deref` to the inner provider, so you can use it anywhere a `Provider<N>` is expected
- HTTP transport only -- WebSocket is not supported for signed providers

## See Also

- [SeismicUnsignedProvider](seismic-unsigned-provider.md) -- Read-only provider without wallet
- [Encryption](encryption.md) -- Detailed encryption flow and ECDH key exchange
- [Provider Overview](./) -- Comparison of provider types and filler pipeline
- [Installation](../installation.md) -- Add seismic-alloy to your project
