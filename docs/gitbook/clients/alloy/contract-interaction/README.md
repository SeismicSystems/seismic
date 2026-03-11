---
description: Interacting with Seismic contracts using seismic-alloy
icon: file-contract
---

# Contract Interaction

Patterns for calling and transacting with smart contracts on Seismic using the Rust SDK.

## Overview

seismic-alloy uses Alloy's `sol!` macro to define contract interfaces and transaction builders to construct calls. Unlike the Python SDK's `ShieldedContract` wrapper with `.write` / `.read` namespaces, seismic-alloy uses a builder pattern: you construct a transaction request and mark it as seismic with `.seismic()` to enable encryption, or omit it for transparent operations.

### Key Concepts

- **`sol!` macro** -- Define contract interfaces with Solidity syntax directly in Rust
- **`.seismic()` builder method** -- Converts a `TransactionRequest` into a `SeismicTransactionRequest` with encryption enabled
- **Filler pipeline** -- Encryption, nonce, gas, and chain ID are filled automatically by the provider
- **`seismic_call()`** -- Signed read method on `SeismicSignedProvider` for encrypted `eth_call`

## Shielded vs. Transparent Operations

| Operation             | Method                                          | Encryption                  | Provider Required       | Use Case                                  |
| --------------------- | ----------------------------------------------- | --------------------------- | ----------------------- | ----------------------------------------- |
| **Shielded Write**    | `send_transaction(tx.into())` with `.seismic()` | Calldata encrypted          | `SeismicSignedProvider` | Privacy-preserving state changes          |
| **Signed Read**       | `seismic_call(tx)`                              | Calldata + result encrypted | `SeismicSignedProvider` | Reading private state                     |
| **Transparent Write** | `send_transaction(tx)` without `.seismic()`     | None                        | `SeismicSignedProvider` | Contract deployment, public state changes |
| **Transparent Read**  | `provider.call(tx)`                             | None                        | Any provider            | Reading public state                      |

{% hint style="info" %}
Contract deployment (Create transactions) **cannot** be seismic. Deploy your contract with a standard transaction, then interact with it using shielded calls.
{% endhint %}

## Defining Contract Interfaces

Use Alloy's `sol!` macro to define your contract's interface. Shielded Solidity types (`suint256`, `sbool`, etc.) appear as their standard counterparts in the ABI:

```rust
use alloy::sol;

sol! {
    interface ISeismicCounter {
        event setNumberEmit();
        event incrementEmit();

        function setNumber(suint256 newNumber) public;
        function increment() public;
        function isOdd() public view returns (bool);
    }
}
```

The macro generates type-safe Rust structs for each function call (e.g., `ISeismicCounter::setNumberCall`, `ISeismicCounter::isOddCall`), which you use to encode calldata.

## Quick Comparison with Python SDK

| Python SDK                      | Rust SDK (seismic-alloy)                                 |
| ------------------------------- | -------------------------------------------------------- |
| `contract.write.setNumber(42)`  | Build tx with `.seismic()`, then `send_transaction()`    |
| `contract.read.isOdd()`         | Build tx with `.seismic()`, then `seismic_call()`        |
| `contract.twrite.setNumber(42)` | Build tx without `.seismic()`, then `send_transaction()` |
| `contract.tread.isOdd()`        | Build tx without `.seismic()`, then `provider.call()`    |
| `ShieldedContract` wrapper      | No wrapper -- use builder pattern directly               |

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
