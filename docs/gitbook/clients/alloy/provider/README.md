---
description: Signed and unsigned provider types for interacting with Seismic nodes
icon: server
---

# Provider

The `seismic-alloy` provider crate provides two provider types for interacting with Seismic nodes:

- **[SeismicSignedProvider](seismic-signed-provider.md)** -- Full capabilities (shielded writes, signed reads, response decryption). Requires a wallet.
- **[SeismicUnsignedProvider](seismic-unsigned-provider.md)** -- Read-only (public queries, block data). No wallet needed.

Both are constructed using `SeismicProviderBuilder` and are generic over `N: SeismicNetwork`, which determines the network-specific transaction and receipt types.

## Provider Comparison

| Capability                        | SeismicSignedProvider             | SeismicUnsignedProvider           |
| --------------------------------- | --------------------------------- | --------------------------------- |
| **Wallet integration**            | Yes (signs transactions)          | No                                |
| **Shielded writes**               | Yes                               | No                                |
| **Signed reads (`seismic_call`)** | Yes (encrypts + decrypts)         | No response decryption            |
| **Public reads**                  | Yes                               | Yes                               |
| **Block/transaction queries**     | Yes                               | Yes                               |
| **TEE pubkey caching**            | Yes (fetched at creation)         | No                                |
| **Response decryption**           | Yes (ephemeral key + TEE key)     | No                                |
| **Calldata encryption**           | Automatic via filler pipeline     | Not applicable                    |
| **HTTP support**                  | Yes                               | Yes                               |
| **WebSocket support**             | Yes                               | Yes                               |

## Quick Start

### Signed Provider

```rust
use seismic_alloy_network::{reth::SeismicReth, wallet::SeismicWallet};
use seismic_alloy_provider::SeismicProviderBuilder;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::<SeismicReth>::from(signer);
let url = "https://gcp-1.seismictest.net/rpc".parse()?;

// Full-featured provider (HTTP)
let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_http(url)
    .await?;

// Full-featured provider (WebSocket)
let ws_url = "wss://gcp-1.seismictest.net/ws".parse()?;
let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_ws(ws_url)
    .await?;
```

### Unsigned Provider

```rust
use seismic_alloy_provider::SeismicProviderBuilder;

let url = "https://gcp-1.seismictest.net/rpc".parse()?;

// Read-only provider (HTTP)
let provider = SeismicProviderBuilder::new()
    .connect_http(url)
    .await?;

// Read-only provider (WebSocket)
let ws_url = "wss://gcp-1.seismictest.net/ws".parse()?;
let provider = SeismicProviderBuilder::new()
    .connect_ws(ws_url)
    .await?;
```

## SeismicProviderBuilder

All providers are constructed via `SeismicProviderBuilder`, which uses a typestate pattern:

```
SeismicProviderBuilder::new()
  │
  ├── .wallet(wallet)         → SeismicProviderBuilderWithWallet<SeismicReth>
  │     ├── .connect_http()   → SeismicSignedProvider<SeismicReth>
  │     └── .connect_ws()     → SeismicSignedProvider<SeismicReth>
  │
  ├── .foundry().wallet(w)    → SeismicProviderBuilderWithWallet<SeismicFoundry>
  │     ├── .connect_http()   → SeismicSignedProvider<SeismicFoundry>
  │     └── .connect_ws()     → SeismicSignedProvider<SeismicFoundry>
  │
  ├── .connect_http(url)      → SeismicUnsignedProvider<SeismicReth>
  └── .connect_ws(url)        → SeismicUnsignedProvider<SeismicReth>
```

The builder defaults to `SeismicReth` as the network type. Use `.foundry()` to switch to `SeismicFoundry` for local development with sanvil.

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

## SeismicProviderExt Trait

The `SeismicProviderExt` trait extends Alloy's `Provider<N>` with Seismic-specific methods:

```rust
pub trait SeismicProviderExt<N: SeismicNetwork>: Provider<N> {
    // High-level contract interaction (integrates with #[sol(rpc)])
    async fn shielded_call<C: SolCall>(&self, addr: Address, call: C) -> TransportResult<C::Return>;
    async fn shielded_send<C: SolCall>(&self, addr: Address, call: C) -> TransportResult<PendingTransactionBuilder<N>>;
    async fn transparent_call<C: SolCall>(&self, addr: Address, call: C) -> TransportResult<C::Return>;
    async fn transparent_send<C: SolCall>(&self, addr: Address, call: C) -> TransportResult<PendingTransactionBuilder<N>>;

    // Low-level methods
    async fn seismic_call(&self, tx: SendableTx<N>) -> TransportResult<Bytes>;
    async fn eip712_send(&self, tx: SendableTx<N>) -> TransportResult<PendingTransactionBuilder<N>>;
    async fn get_tee_pubkey(&self) -> TransportResult<PublicKey>;
}
```

| Method               | Description                                                        |
| -------------------- | ------------------------------------------------------------------ |
| `shielded_call()`    | Encrypted read with response decryption (requires signed provider) |
| `shielded_send()`    | Encrypted write transaction                                        |
| `transparent_call()` | Standard `eth_call` without encryption                             |
| `transparent_send()` | Standard transaction without encryption                            |
| `seismic_call()`     | Low-level encrypted call with raw bytes                            |
| `eip712_send()`      | Send via EIP-712 typed data (for browser wallets)                  |
| `get_tee_pubkey()`   | Fetch the TEE public key from the node                             |

## Network Types

| Type             | Description                | Use With                 |
| ---------------- | -------------------------- | ------------------------ |
| `SeismicReth`    | Production Seismic network | Devnet, testnet, mainnet |
| `SeismicFoundry` | Local development network  | sanvil local nodes       |

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
