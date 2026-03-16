---
description: Interacting with Seismic contracts using seismic-alloy
icon: file-contract
---

# Contract Interaction

Patterns for calling and transacting with smart contracts on Seismic using the Rust SDK.

## Overview

seismic-alloy uses Alloy's `sol!` macro with `#[sol(rpc)]` to define contract interfaces and generate type-safe call builders. The `.seismic()` method on any generated call builder enables encryption for shielded operations. Omit `.seismic()` for transparent (unencrypted) operations.

### Key Concepts

- **`sol!` macro with `#[sol(rpc)]`** -- Define contract interfaces with Solidity syntax, generate call builders and deploy methods
- **`.seismic()` call builder** -- Marks a contract call for encryption; chain with `.call()` for reads or `.send()` for writes
- **SecurityParams** -- Per-call overrides for expiration, block hash, and encryption nonce
- **EIP-712** -- `.seismic().eip712()` for browser wallet compatibility

## Shielded vs. Transparent Operations

| Operation             | Pattern                                        | Encryption                  | Provider Required       |
| --------------------- | ---------------------------------------------- | --------------------------- | ----------------------- |
| **Shielded Read**     | `contract.method().seismic().call().await?`     | Calldata + result encrypted | `SeismicSignedProvider` |
| **Shielded Write**    | `contract.method().seismic().send().await?`     | Calldata encrypted          | `SeismicSignedProvider` |
| **Transparent Read**  | `contract.method().call().await?`               | None                        | Any provider            |
| **Transparent Write** | `contract.method().send().await?`               | None                        | `SeismicSignedProvider` |

{% hint style="info" %}
Contract deployment (Create transactions) **cannot** be seismic. Deploy your contract with a standard transaction, then interact with it using shielded calls.
{% endhint %}

## Defining Contract Interfaces

Use Alloy's `sol!` macro with `#[sol(rpc)]` to define your contract's interface. This generates type-safe call builders with `.call()` and `.send()` methods:

```rust
use alloy_sol_types::sol;

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
use seismic_alloy_provider::{SeismicCallExt, SeismicProviderBuilder};
use seismic_alloy_network::{reth::SeismicReth, wallet::SeismicWallet};
use alloy_signer_local::PrivateKeySigner;
use alloy_primitives::U256;
use alloy_sol_types::sol;

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

// Shielded read
let is_odd = contract.isOdd().seismic().call().await?;

// Shielded write
contract.setNumber(U256::from(42).into())
    .seismic()
    .send()
    .await?
    .get_receipt()
    .await?;

// Transparent read (standard eth_call)
let is_odd = contract.isOdd().call().await?;

// Transparent write (standard transaction)
contract.setNumber(U256::from(42).into())
    .send()
    .await?
    .get_receipt()
    .await?;
```

## Quick Comparison with Python SDK

| Python SDK                      | Rust SDK (seismic-alloy)                              |
| ------------------------------- | ----------------------------------------------------- |
| `contract.write.setNumber(42)`  | `contract.setNumber(42).seismic().send().await?`      |
| `contract.read.isOdd()`         | `contract.isOdd().seismic().call().await?`             |
| `contract.twrite.setNumber(42)` | `contract.setNumber(42).send().await?`                 |
| `contract.tread.isOdd()`        | `contract.isOdd().call().await?`                       |
| `ShieldedContract` wrapper      | No wrapper -- `.seismic()` on any call builder         |

## Navigation

| Page                                      | Description                                          |
| ----------------------------------------- | ---------------------------------------------------- |
| [Shielded Calls](shielded-calls.md)       | Encrypted writes and signed reads using `.seismic()` |
| [Transparent Calls](transparent-calls.md) | Standard Ethereum calls without encryption           |

## See Also

- [SeismicSignedProvider](../provider/seismic-signed-provider.md) -- Required for shielded operations
- [SeismicUnsignedProvider](../provider/seismic-unsigned-provider.md) -- Sufficient for transparent reads
- [Transaction Types](../transaction-types/) -- Underlying transaction structs
- [Encryption](../provider/encryption.md) -- How calldata encryption works
