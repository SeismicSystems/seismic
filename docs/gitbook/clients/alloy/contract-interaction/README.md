---
description: Interacting with Seismic contracts using seismic-alloy
icon: file-contract
---

# Contract Interaction

Patterns for calling and transacting with smart contracts on Seismic using the Rust SDK.

## Overview

seismic-alloy uses Alloy's `sol!` macro with `#[sol(rpc)]` to define contract interfaces and generate type-safe call builders. Functions with shielded parameters (e.g., `suint256`, `saddress`, `sbool`) auto-encrypt via `ShieldedCallBuilder` — you can call `.send()` or `.call()` directly without `.seismic()`. For non-shielded functions that still need encryption, use `.seismic()` to opt in. Omit `.seismic()` entirely for transparent (unencrypted) operations.

### Key Concepts

- **`sol!` macro with `#[sol(rpc)]`** — Define contract interfaces with Solidity syntax, generate call builders and deploy methods
- **Auto-encryption for shielded params** — Functions with shielded types in their arguments return a `ShieldedCallBuilder` automatically; call `.send()` or `.call()` directly
- **`.seismic()` call builder** — For non-shielded functions that need encryption; converts a `SolCallBuilder` into a `ShieldedCallBuilder`
- **Two traits** — `SeismicCallExt` adds `.seismic()` to `SolCallBuilder`; `ShieldedCallExt` adds `.call()`, `.send()`, and builder methods to `ShieldedCallBuilder`
- **SecurityParams** — Per-call overrides for expiration, block hash, and encryption nonce
- **EIP-712** — `.eip712()` on `ShieldedCallBuilder` for browser wallet compatibility

## Shielded vs. Transparent Operations

| Operation | When | Pattern | Provider Required |
| --- | --- | --- | --- |
| **Shielded Read** | Function has shielded params | `contract.method().call().await?` | `SeismicSignedProvider` |
| **Shielded Read** | Function has no shielded params | `contract.method().seismic().call().await?` | `SeismicSignedProvider` |
| **Shielded Write** | Function has shielded params | `contract.method().send().await?` | `SeismicSignedProvider` |
| **Shielded Write** | Function has no shielded params | `contract.method().seismic().send().await?` | `SeismicSignedProvider` |
| **Transparent Read** | Any | `contract.method().call().await?` | Any provider |
| **Transparent Write** | Any | `contract.method().send().await?` | `SeismicSignedProvider` |

{% hint style="info" %}
Contract deployment (Create transactions) **cannot** be seismic. Deploy your contract with a standard transaction, then interact with it using shielded calls.
{% endhint %}

## Defining Contract Interfaces

Use Alloy's `sol!` macro with `#[sol(rpc)]` to define your contract's interface. This generates type-safe call builders with `.call()` and `.send()` methods:

```rust
use seismic_prelude::client::*; // sol! macro is included in the prelude

sol! {
    #[sol(rpc)]
    contract SeismicCounter {
        event setNumberEmit();
        event incrementEmit();

        function setNumber(suint256 newNumber) public;
        function increment() public;
        function isOdd() public view returns (bool);
    }
}
```

To also include deployment, add the `bytecode` attribute:

```rust
sol! {
    #[sol(rpc, bytecode = "0x60806040...")]
    contract SeismicCounter {
        // ...
    }
}
```

## Quick Example

```rust
use seismic_prelude::client::*;
// The prelude re-exports both SeismicCallExt (adds .seismic() to SolCallBuilder)
// and ShieldedCallExt (adds .call(), .send(), .eip712() to ShieldedCallBuilder),
// so you don't need to worry about which trait to import.
use seismic_alloy_network::reth::SeismicReth;

sol! {
    #[sol(rpc)]
    contract SeismicCounter {
        function setNumber(suint256 newNumber) public;
        function isOdd() public view returns (bool);
    }
}

let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_http(url)
    .await?;

let contract = SeismicCounter::new(address, &provider);

// Shielded read -- isOdd() has no shielded params, so use .seismic()
let is_odd = contract.isOdd().seismic().call().await?;

// Shielded write -- setNumber has a shielded param (suint256), auto-encrypts
contract.setNumber(alloy_primitives::aliases::SUInt(U256::from(42)))
    .send()
    .await?
    .get_receipt()
    .await?;

// Transparent read (standard eth_call)
let is_odd = contract.isOdd().call().await?;
```

## Quick Comparison with Python SDK

| Python SDK                      | Rust SDK (seismic-alloy)                              |
| ------------------------------- | ----------------------------------------------------- |
| `contract.write.setNumber(42)`  | `contract.setNumber(42).send().await?` (auto-encrypts, shielded param) |
| `contract.read.isOdd()`         | `contract.isOdd().seismic().call().await?`             |
| `contract.twrite.setNumber(42)` | N/A (use transparent provider or skip `.seismic()` on non-shielded fn) |
| `contract.tread.isOdd()`        | `contract.isOdd().call().await?`                       |
| `ShieldedContract` wrapper      | No wrapper — auto-encryption for shielded params, `.seismic()` for others |

## Navigation

| Page                                      | Description                                          |
| ----------------------------------------- | ---------------------------------------------------- |
| [Shielded Calls](shielded-calls.md)       | Encrypted writes and signed reads using auto-encryption and `.seismic()` |
| [Transparent Calls](transparent-calls.md) | Standard Ethereum calls without encryption           |

## See Also

- [SeismicSignedProvider](../provider/seismic-signed-provider.md) — Required for shielded operations
- [SeismicUnsignedProvider](../provider/seismic-unsigned-provider.md) — Sufficient for transparent reads
- [Transaction Types](../transaction-types/) — Underlying transaction structs
- [Encryption](../provider/encryption.md) — How calldata encryption works
