---
description: Signed and unsigned provider types for interacting with Seismic nodes
icon: server
---

# Provider

The `seismic-alloy` provider crate provides two provider types for interacting with Seismic nodes:

- **[SeismicSignedProvider](seismic-signed-provider.md)** -- Full capabilities (shielded writes, signed reads, response decryption). Requires a wallet.
- **[SeismicUnsignedProvider](seismic-unsigned-provider.md)** -- Read-only (public queries, block data). No wallet needed.

Both providers are generic over `N: SeismicNetwork`, which determines the network-specific transaction and receipt types.

## Provider Comparison

| Capability                        | SeismicSignedProvider         | SeismicUnsignedProvider                     |
| --------------------------------- | ----------------------------- | ------------------------------------------- |
| **Wallet integration**            | Yes (signs transactions)      | No                                          |
| **Shielded writes**               | Yes                           | No                                          |
| **Signed reads (`seismic_call`)** | Yes (encrypts + decrypts)     | No response decryption                      |
| **Public reads**                  | Yes                           | Yes                                         |
| **Block/transaction queries**     | Yes                           | Yes                                         |
| **TEE pubkey caching**            | Yes (fetched at creation)     | No                                          |
| **Response decryption**           | Yes (ephemeral key + TEE key) | No                                          |
| **Calldata encryption**           | Automatic via filler pipeline | Not applicable                              |
| **WebSocket support**             | No (HTTP only)                | Yes (`new_ws()`)                            |
| **Constructor**                   | `async fn new(wallet, url)`   | `fn new_http(url)` / `async fn new_ws(url)` |

## Quick Start

### Signed Provider

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url = "https://node.seismicdev.net/rpc".parse()?;

// Full-featured provider
let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;
```

### Unsigned Provider

```rust
use seismic_alloy::prelude::*;

let url = "https://node.seismicdev.net/rpc".parse()?;

// Read-only provider (HTTP)
let provider = sreth_unsigned_provider(url);

// Read-only provider (WebSocket)
let ws_url = "wss://node.seismicdev.net/ws".parse()?;
let provider = SeismicUnsignedProvider::<SeismicReth>::new_ws(ws_url).await?;
```

## Filler Pipeline

Both providers use Alloy's filler pipeline to automatically populate transaction fields before sending. The filler chain differs between signed and unsigned providers:

### Signed Provider Filler Chain

```
Request (TransactionRequest)
  |
  v
WalletFiller          -- Signs the transaction with the attached wallet
  |
  v
NonceFiller           -- Fetches and sets the transaction nonce
ChainIdFiller         -- Sets the chain ID
  |
  v
SeismicElementsFiller -- Populates Seismic-specific fields (encryption nonce,
  |                       TEE pubkey, block hash, expiry)
  v
SeismicGasFiller      -- Estimates and sets gas parameters
  |
  v
(encrypt calldata)    -- AES-GCM encryption with ECDH shared secret
  |
  v
Send to node
  |
  v
(decrypt response)    -- AES-GCM decryption for seismic_call responses
```

### Unsigned Provider Filler Chain

```
Request (TransactionRequest)
  |
  v
SeismicElementsFiller -- Populates Seismic-specific fields
  |
  v
NonceFiller           -- Fetches and sets the transaction nonce
ChainIdFiller         -- Sets the chain ID
  |
  v
SeismicGasFiller      -- Estimates and sets gas parameters
  |
  v
Send to node
```

{% hint style="info" %}
The signed provider places `WalletFiller` first because the wallet must sign the fully-populated transaction. The unsigned provider does not have a wallet filler since it cannot sign transactions.
{% endhint %}

## Convenience Constructors

Both provider types have convenience functions that pre-fill the network generic parameter:

### Signed Providers

| Function                                | Network          | Description                        |
| --------------------------------------- | ---------------- | ---------------------------------- |
| `sreth_signed_provider(wallet, url)`    | `SeismicReth`    | For Seismic devnet/testnet/mainnet |
| `sfoundry_signed_provider(wallet, url)` | `SeismicFoundry` | For local sfoundry instances       |

### Unsigned Providers

| Function                          | Network          | Description                        |
| --------------------------------- | ---------------- | ---------------------------------- |
| `sreth_unsigned_provider(url)`    | `SeismicReth`    | For Seismic devnet/testnet/mainnet |
| `sfoundry_unsigned_provider(url)` | `SeismicFoundry` | For local sfoundry instances       |

## SeismicProviderExt Trait

The `SeismicProviderExt` trait extends Alloy's `Provider<N>` with Seismic-specific methods:

```rust
pub trait SeismicProviderExt<N: SeismicNetwork>: Provider<N> {
    async fn seismic_call(&self, tx: SendableTx<N>) -> TransportResult<Bytes>;
    fn should_encrypt_input<B: TransactionBuilder<N>>(&self, tx: &B) -> bool;
    async fn get_tee_pubkey(&self) -> TransportResult<PublicKey>;
    async fn call_conditionally_signed(&self, tx: SendableTx<N>) -> TransportResult<Bytes>;
}
```

| Method                        | Description                                                                 |
| ----------------------------- | --------------------------------------------------------------------------- |
| `seismic_call()`              | Fill, encrypt, send, and decrypt an `eth_call`-style request                |
| `should_encrypt_input()`      | Check if a transaction's calldata should be encrypted                       |
| `get_tee_pubkey()`            | Fetch the TEE public key from the node via `seismic_getTeePublicKey`        |
| `call_conditionally_signed()` | Send a call that is signed if the provider has a wallet, unsigned otherwise |

## Network Types

| Type             | Description                | Use With                 |
| ---------------- | -------------------------- | ------------------------ |
| `SeismicReth`    | Production Seismic network | Devnet, testnet, mainnet |
| `SeismicFoundry` | Local development network  | sfoundry local nodes     |

Both types implement `SeismicNetwork`, which extends Alloy's `Network` trait with Seismic transaction and receipt types.

## Provider Pages

| Page                                                    | Description                                                    |
| ------------------------------------------------------- | -------------------------------------------------------------- |
| [SeismicSignedProvider](seismic-signed-provider.md)     | Full-featured provider with wallet, encryption, and decryption |
| [SeismicUnsignedProvider](seismic-unsigned-provider.md) | Lightweight read-only provider                                 |
| [Encryption](encryption.md)                             | TEE key exchange, ECDH, AES-GCM encryption details             |

## See Also

- [Installation](../installation.md) -- Add seismic-alloy to your project
- [Encryption](encryption.md) -- How calldata encryption works
- [seismic-alloy Overview](../) -- SDK architecture and crate structure
