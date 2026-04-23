---
description: Signed and unsigned provider types for interacting with Seismic nodes
icon: server
---

# Provider

The `seismic-alloy` provider crate exposes two provider types for interacting with Seismic nodes:

- **[SeismicSignedProvider](seismic-signed-provider.md)** — Full capabilities (shielded writes, signed reads, response decryption). Requires a wallet.
- **[SeismicUnsignedProvider](seismic-unsigned-provider.md)** — Read-only (public queries, block data). No wallet needed.

Both are constructed using `SeismicProviderBuilder` and are generic over `N: SeismicNetwork`, which determines the network-specific transaction and receipt types.

## Choosing a Provider

Pick **`SeismicSignedProvider`** if the application needs to:

- write to shielded state (call a function with `suint*` / `saddress` / `sbool` args, or any write that must be encrypted),
- issue signed reads (`eth_call` with `msg.sender` authenticated, e.g., to fetch a caller's shielded balance), or
- decrypt TEE responses.

Pick **`SeismicUnsignedProvider`** for read-only work where the caller's identity doesn't matter: indexers, monitoring dashboards, block/receipt queries, fetching the TEE public key. It's lighter (no keypair generation, no decryption layer) and has no wallet to manage.

Starting with the unsigned provider and upgrading later is rarely what you want: the two types implement different traits, so code written against `SignedProviderExt` won't compile against an unsigned provider. Decide up front, or build a tiny wrapper that abstracts the bits you share.

## Provider Comparison

| Capability                        | SeismicSignedProvider             | SeismicUnsignedProvider           |
| --------------------------------- | --------------------------------- | --------------------------------- |
| **Wallet integration**            | Yes (signs transactions)          | No                                |
| **Shielded writes**               | Yes                               | No                                |
| **Signed reads (`SignedProviderExt`)** | Yes (encrypts + decrypts)    | No (not available)                |
| **Public reads**                  | Yes                               | Yes                               |
| **Block/transaction queries**     | Yes                               | Yes                               |
| **TEE pubkey caching**            | Yes (fetched at creation)         | No                                |
| **Response decryption**           | Yes (provider key + TEE key)      | No                                |
| **Calldata encryption**           | Automatic via filler pipeline     | Not applicable                    |
| **HTTP support**                  | Yes                               | Yes                               |
| **WebSocket support**             | Yes                               | Yes                               |

## Quick Start

### Signed Provider

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::<SeismicReth>::from(signer);
let url = "https://testnet-1.seismictest.net/rpc".parse()?;

// Full-featured provider (HTTP)
let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_http(url)
    .await?;

// Full-featured provider (WebSocket)
let ws_url = "wss://testnet-1.seismictest.net/ws".parse()?;
let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_ws(ws_url)
    .await?;
```

### Unsigned Provider

```rust
use seismic_prelude::client::*;

let url = "https://testnet-1.seismictest.net/rpc".parse()?;

// Read-only provider (HTTP) — synchronous, no .await
let provider = SeismicProviderBuilder::new()
    .connect_http(url);

// Read-only provider (WebSocket) — async due to WS handshake
let ws_url = "wss://testnet-1.seismictest.net/ws".parse()?;
let provider = SeismicProviderBuilder::new()
    .connect_ws(ws_url)
    .await?;
```

## SeismicProviderBuilder

All providers are constructed via `SeismicProviderBuilder`, which uses a typestate pattern:

```
SeismicProviderBuilder::new()
  │
  ├── .wallet(wallet)          → SeismicProviderBuilderWithWallet<SeismicReth>
  │     ├── .connect_http()    → SeismicSignedProvider<SeismicReth>
  │     └── .connect_ws()      → SeismicSignedProvider<SeismicReth>
  │
  ├── .foundry().wallet(w)     → SeismicProviderBuilderWithWallet<SeismicFoundry>
  │     ├── .connect_http()    → SeismicSignedProvider<SeismicFoundry>
  │     └── .connect_ws()      → SeismicSignedProvider<SeismicFoundry>
  │
  ├── .network::<N>().wallet(w) → SeismicProviderBuilderWithWallet<N>   // custom network
  │     ├── .connect_http()    → SeismicSignedProvider<N>
  │     └── .connect_ws()      → SeismicSignedProvider<N>
  │
  ├── .connect_http(url)       → SeismicUnsignedProvider<SeismicReth>
  └── .connect_ws(url)         → SeismicUnsignedProvider<SeismicReth>
```

The builder defaults to `SeismicReth` as the network type. Use `.foundry()` to switch to `SeismicFoundry` for sanvil, or `.network::<N>()` to plug in any `N: SeismicNetwork`.

## Filler Pipeline

Both providers use Alloy's filler pipeline to automatically populate transaction fields before sending. The filler chain differs between signed and unsigned providers:

### Signed Provider Filler Chain

Shown in **operational order** — what happens to a transaction as it flows out to the node.

```
Request (TransactionRequest)
  |
  v
NonceFiller           — Fetches and sets the transaction nonce
ChainIdFiller         — Sets the chain ID
  |
  v
SeismicElementsFiller — Populates Seismic-specific fields AND encrypts
  |                      calldata (AES-GCM with ECDH-derived key)
  v
SeismicGasFiller      — Signs the encrypted tx and calls eth_estimateGas
  |                      (so the node can authenticate msg.sender)
  v
WalletFiller          — Final signature over the fully-filled tx
  |
  v
Send to node
  |
  v
(decrypt response)    — AES-GCM decryption for seismic_call responses
```

{% hint style="info" %}
In source (`crates/provider/src/builder.rs`) the chain is composed `Wallet → (Nonce + ChainId) → SeismicElements → Gas`. Chain position doesn't equal execution order: each filler's `status()` gate decides when it fires, and `WalletFiller` only reports ready once all the other fields are populated — so signing ends up last in practice.
{% endhint %}

### Unsigned Provider Filler Chain

```
Request (TransactionRequest)
  |
  v
NonceFiller           — Fetches and sets the transaction nonce
ChainIdFiller         — Sets the chain ID
  |
  v
GasFiller             — Estimates and sets gas parameters
  |
  v
Send to node
```

{% hint style="info" %}
The unsigned provider uses a minimal chain of standard Alloy fillers (`NonceFiller`, `ChainIdFiller`, `GasFiller`). It does not include `SeismicElementsFiller` or `SeismicGasFiller` because it cannot perform shielded operations. The signed provider additionally includes `WalletFiller`, `SeismicElementsFiller`, and `SeismicGasFiller`.
{% endhint %}

## Provider Traits

### SeismicProviderExt (base trait, all providers)

`SeismicProviderExt` is available on both signed and unsigned providers:

```rust
pub trait SeismicProviderExt<N: SeismicNetwork>: Provider<N> {
    async fn transparent_call<C: SolCall>(&self, addr: Address, call: C) -> TransportResult<C::Return>;
    async fn transparent_send<C: SolCall>(&self, addr: Address, call: C) -> TransportResult<PendingTransactionBuilder<N>>;
    async fn get_tee_pubkey(&self) -> TransportResult<PublicKey>;
}
```

| Method               | Description                                    |
| -------------------- | ---------------------------------------------- |
| `transparent_call()` | Standard `eth_call` without encryption         |
| `transparent_send()` | Standard transaction without encryption. **Requires a wallet** — calling on `SeismicUnsignedProvider` fails at runtime. |
| `get_tee_pubkey()`   | Fetch the TEE public key from the node         |

### SignedProviderExt (sealed trait, signed provider only)

`SignedProviderExt` is only available on `SeismicSignedProvider`:

```rust
pub trait SignedProviderExt<N: SeismicNetwork>: SeismicProviderExt<N> {
    // High-level (ABI encodes/decodes automatically)
    async fn seismic_call<C: SolCall>(&self, addr: Address, call: C) -> TransportResult<C::Return>;
    async fn seismic_send<C: SolCall>(&self, addr: Address, call: C) -> TransportResult<PendingTransactionBuilder<N>>;

    // High-level with SecurityParams overrides
    async fn seismic_call_with<C: SolCall>(&self, addr: Address, call: C, params: SecurityParams) -> TransportResult<C::Return>;
    async fn seismic_send_with<C: SolCall>(&self, addr: Address, call: C, params: SecurityParams) -> TransportResult<PendingTransactionBuilder<N>>;

    // Low-level (raw bytes, no ABI decoding)
    async fn seismic_call_raw(&self, tx: SendableTx<N>) -> TransportResult<Bytes>;
    async fn eip712_send(&self, tx: SendableTx<N>) -> TransportResult<PendingTransactionBuilder<N>>;
}
```

| Method                  | Description                                                        |
| ----------------------- | ------------------------------------------------------------------ |
| `seismic_call()`        | Encrypted read with ABI encoding/decoding and response decryption  |
| `seismic_send()`        | Encrypted write with ABI encoding                                  |
| `seismic_call_with()`   | Encrypted read with `SecurityParams` overrides                     |
| `seismic_send_with()`   | Encrypted write with `SecurityParams` overrides                    |
| `seismic_call_raw()`    | Low-level encrypted call with raw bytes                            |
| `eip712_send()`         | Send via EIP-712 typed data (for browser wallets)                  |

## Errors

Provider methods return `TransportResult<_>`, but the underlying `TransportError` can wrap a structured `SeismicProviderError`. It's re-exported from `seismic_alloy_provider::SeismicProviderError`.

| Variant                     | When it fires                                                                                     |
| --------------------------- | ------------------------------------------------------------------------------------------------- |
| `AbiDecode(err)`            | Response decoded past decryption but the bytes didn't match the call's return type                |
| `Decryption(msg)`           | ECDH / AES-GCM decrypt failed — AAD mismatch, wrong key, or corrupted ciphertext                 |
| `MissingSender`             | A filled transaction reached the decryption layer without a `from` field                          |
| `MetadataCreation(msg)`     | Couldn't build `TxSeismicMetadata` for AAD construction from the filled transaction               |
| `NotSeismicEnvelope`        | Decryption asked for a seismic envelope but got a non-seismic transaction type                    |
| `Eip712NotSeismicEnvelope`  | `eip712_send` received a filled tx that isn't an EIP-712 seismic envelope                         |
| `Eip712GotBuilder`          | `eip712_send` got back a builder instead of a fully-filled, signed envelope                       |
| `PrecompileOutput(msg)`     | One of the [precompile helpers](../precompiles/) returned output in an unexpected format          |

Typical mitigation:

- `AbiDecode` and `Decryption` usually mean a provider/wallet mismatch or a tampered ciphertext — rebuild the provider against the same node and retry.
- `MissingSender` / `MetadataCreation` / `NotSeismicEnvelope` signal a filler pipeline misconfiguration; check that you built the provider via `SeismicProviderBuilder` rather than assembling fillers by hand.
- `Eip712*` errors only surface in the `eip712_send` path and usually mean the tx wasn't flagged as EIP-712 (missing `message_version >= 2` or missing `.eip712()` on the call builder).

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

- [Installation](../installation.md) — Add seismic-alloy to your project
- [Encryption](encryption.md) — How calldata encryption works
- [seismic-alloy Overview](../) — SDK architecture and crate structure
