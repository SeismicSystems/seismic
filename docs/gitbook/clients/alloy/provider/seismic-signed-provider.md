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

## Construction

All signed providers are created via `SeismicProviderBuilder`:

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::<SeismicReth>::from(signer);
```

### HTTP

```rust
let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_http("https://gcp-1.seismictest.net/rpc".parse()?)
    .await?;
```

### WebSocket

```rust
let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_ws("wss://gcp-1.seismictest.net/ws".parse()?)
    .await?;
```

### With Pre-fetched TEE Pubkey

If you already have the TEE public key, skip the initial RPC call:

```rust
let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_http_with_tee_pubkey(url, tee_pubkey)
    .await?;
```

{% hint style="info" %}
`connect_http()` and `connect_ws()` are async because they make an RPC call to `seismic_getTeePublicKey`. Use `connect_http_with_tee_pubkey()` or `connect_ws_with_tee_pubkey()` to supply a pre-fetched key and avoid this call.
{% endhint %}

### Local Development with sanvil

Use `.foundry()` to select the `SeismicFoundry` network type:

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::foundry::SeismicFoundry;
use alloy_node_bindings::Anvil;

let anvil = Anvil::at("sanvil").spawn();
let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
let wallet = SeismicWallet::<SeismicFoundry>::from(signer);

let provider = SeismicProviderBuilder::new()
    .foundry()
    .wallet(wallet)
    .connect_http(anvil.endpoint_url())
    .await?;
```

## Contract Interaction

The primary way to interact with contracts is via the `ShieldedCallBuilder`, which integrates with Alloy's `#[sol(rpc)]` macro. Functions with shielded parameters (e.g., `suint256`) auto-encrypt. For functions without shielded parameters, use `.seismic()` to opt in:

```rust
// SeismicCallExt and ShieldedCallExt are included in the prelude

sol! {
    #[sol(rpc)]
    contract SeismicCounter {
        function setNumber(suint256 newNumber) public;
        function isOdd() public view returns (bool);
    }
}

let contract = SeismicCounter::new(address, &provider);

// Shielded read -- isOdd has no shielded params, use .seismic()
let is_odd = contract.isOdd().seismic().call().await?;

// Shielded write -- setNumber has suint256 param, auto-encrypts
let receipt = contract
    .setNumber(U256::from(42).into())
    .send()
    .await?
    .get_receipt()
    .await?;

// Transparent read (no encryption)
let is_odd = contract.isOdd().call().await?;
```

### SecurityParams (Per-Call Overrides)

Customize encryption parameters on individual calls:

```rust
let is_odd = contract.isOdd()
    .seismic()
    .expires_at(current_block + 50)
    .recent_block_hash(block_hash)
    .call()
    .await?;
```

| Method                 | Description                                       |
| ---------------------- | ------------------------------------------------- |
| `.expires_at(block)`   | Set transaction expiration block number            |
| `.recent_block_hash()` | Pin to a specific chain state                      |
| `.encryption_nonce()`  | Override AEAD nonce (testing only)                 |
| `.eip712()`            | Use EIP-712 typed data signing (browser wallets)   |

### EIP-712 (Browser Wallet Compatibility)

For wallets that cannot sign custom RLP-encoded transaction types (e.g., MetaMask):

```rust
// setNumber auto-encrypts (shielded param) -- .eip712() available on ShieldedCallBuilder
contract.setNumber(U256::from(42).into())
    .eip712()
    .send()
    .await?;
```

## Low-Level Trait Methods

For cases where the `#[sol(rpc)]` pattern doesn't fit, use `SeismicProviderExt` directly:

```rust
// SeismicProviderExt is included in the prelude

// Shielded read
let result = provider.shielded_call(addr, MyContract::isOddCall {}).await?;

// Shielded write
let pending = provider.shielded_send(addr, MyContract::setNumberCall {
    newNumber: U256::from(42).into(),
}).await?;

// Transparent operations
let result = provider.transparent_call(addr, MyContract::isOddCall {}).await?;
let pending = provider.transparent_send(addr, MyContract::setNumberCall { ... }).await?;

// Fetch TEE public key
let tee_pubkey = provider.get_tee_pubkey().await?;
```

### Via Standard Alloy `Provider`

All standard Alloy provider methods are available:

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
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

sol! {
    #[sol(rpc)]
    contract SeismicCounter {
        function setNumber(suint256 newNumber) public;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::<SeismicReth>::from(signer);
    let url = "https://gcp-1.seismictest.net/rpc".parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;

    let contract = SeismicCounter::new(contract_address, &provider);

    // setNumber auto-encrypts (suint256 param)
    let receipt = contract
        .setNumber(U256::from(42).into())
        .send()
        .await?
        .get_receipt()
        .await?;
    println!("Shielded write confirmed: {:?}", receipt.transaction_hash);

    Ok(())
}
```

### Signed Read

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

sol! {
    #[sol(rpc)]
    contract SeismicCounter {
        function isOdd() public view returns (bool);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::<SeismicReth>::from(signer);
    let url = "https://gcp-1.seismictest.net/rpc".parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;

    let contract = SeismicCounter::new(contract_address, &provider);

    // Shielded: encrypts calldata, signs, decrypts response
    let is_odd = contract.isOdd().seismic().call().await?;
    println!("Is odd: {is_odd}");

    Ok(())
}
```

### Pre-fetched TEE Pubkey

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url: reqwest::Url = "https://gcp-1.seismictest.net/rpc".parse()?;

    // Fetch TEE pubkey once via an unsigned provider
    let unsigned = SeismicProviderBuilder::new().connect_http(url.clone()).await?;
    let tee_pubkey = unsigned.get_tee_pubkey().await?;

    // Create multiple signed providers without additional RPC calls
    let signer1: PrivateKeySigner = "0xKEY1".parse()?;
    let provider1 = SeismicProviderBuilder::new()
        .wallet(SeismicWallet::<SeismicReth>::from(signer1))
        .connect_http_with_tee_pubkey(url.clone(), tee_pubkey);

    let signer2: PrivateKeySigner = "0xKEY2".parse()?;
    let provider2 = SeismicProviderBuilder::new()
        .wallet(SeismicWallet::<SeismicReth>::from(signer2))
        .connect_http_with_tee_pubkey(url, tee_pubkey);

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
- Both HTTP and WebSocket transports are supported

## See Also

- [SeismicUnsignedProvider](seismic-unsigned-provider.md) -- Read-only provider without wallet
- [Encryption](encryption.md) -- Detailed encryption flow and ECDH key exchange
- [Provider Overview](./) -- Comparison of provider types and filler pipeline
- [Installation](../installation.md) -- Add seismic-alloy to your project
