---
description: Encrypted writes and signed reads using seismic-alloy
icon: shield-halved
---

# Shielded Calls

Privacy-preserving contract interactions where calldata (and optionally return data) is encrypted using the TEE's public key.

## Overview

Shielded calls use the Seismic transaction type (`0x4A`) to encrypt calldata before it reaches the network. The SDK's filler pipeline handles encryption automatically -- you mark a transaction as seismic with `.seismic()`, and the provider encrypts it before broadcast.

There are two shielded operations:

- **Shielded Write** -- Encrypted `send_transaction` that modifies on-chain state
- **Signed Read** -- Encrypted `eth_call` that reads state without modification

Both require a `SeismicSignedProvider` because they need a private key for ECDH key derivation and transaction signing.

## Defining a Contract Interface

Use Alloy's `sol!` macro to define your contract interface:

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

This generates Rust types for each function:

- `ISeismicCounter::setNumberCall { newNumber: U256 }`
- `ISeismicCounter::incrementCall {}`
- `ISeismicCounter::isOddCall {}`

## Shielded Write

A shielded write sends an encrypted transaction that modifies on-chain state. The calldata is encrypted with AES-GCM using a shared secret derived from the TEE's public key.

### Building the Transaction

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::{Address, U256, TxKind};
use alloy::sol_types::SolCall;

// Encode the function call
let calldata = ISeismicCounter::setNumberCall {
    newNumber: U256::from(42),
}.abi_encode();

// Build a seismic transaction request
let tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
    .with_input(calldata.into())
    .with_kind(TxKind::Call(contract_address))
    .into()
    .seismic();  // Mark as seismic -- enables encryption
```

### Sending the Transaction

```rust
// Send the encrypted transaction
let pending_tx = provider.send_transaction(tx.into()).await?;

// Wait for the receipt
let receipt = pending_tx.get_receipt().await?;
println!("Transaction hash: {:?}", receipt.transaction_hash);
println!("Status: {:?}", receipt.status());
```

### Complete Example

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::{Address, U256, TxKind};
use alloy::sol_types::SolCall;
use alloy_signer_local::PrivateKeySigner;

sol! {
    interface ISeismicCounter {
        function setNumber(suint256 newNumber) public;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up provider
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    let contract_address: Address = "0x1234...".parse()?;

    // Encode calldata
    let calldata = ISeismicCounter::setNumberCall {
        newNumber: U256::from(42),
    }.abi_encode();

    // Build seismic transaction
    let tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(calldata.into())
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    // Send and wait
    let pending_tx = provider.send_transaction(tx.into()).await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Shielded write confirmed: {:?}", receipt.transaction_hash);

    Ok(())
}
```

## Signed Read

A signed read executes an encrypted `eth_call` that proves the caller's identity to the TEE. This is required for reading private state that is access-controlled by `msg.sender`.

### Building the Read Request

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::TxKind;
use alloy::sol_types::SolCall;

// Encode the view function call
let call_input = ISeismicCounter::isOddCall {}.abi_encode().into();

// Build a seismic transaction (signed read)
let tx = seismic_foundry_tx_builder()
    .with_input(call_input)
    .with_kind(TxKind::Call(contract_address))
    .into()
    .seismic();
```

### Executing the Read

```rust
// Use seismic_call() for signed reads
let result = provider.seismic_call(SendableTx::Builder(tx.into())).await?;

// Decode the result
let is_odd: bool = /* decode result bytes */;
println!("Is odd: {is_odd}");
```

### Complete Example

```rust
use seismic_alloy::prelude::*;
use alloy::primitives::{Address, TxKind};
use alloy::sol_types::SolCall;
use alloy_signer_local::PrivateKeySigner;

sol! {
    interface ISeismicCounter {
        function isOdd() public view returns (bool);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up signed provider (required for signed reads)
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    let contract_address: Address = "0x1234...".parse()?;

    // Encode the view call
    let call_input = ISeismicCounter::isOddCall {}.abi_encode().into();

    // Build seismic transaction for signed read
    let tx = seismic_foundry_tx_builder()
        .with_input(call_input)
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    // Execute signed read
    let result = provider.seismic_call(SendableTx::Builder(tx.into())).await?;
    println!("Result: {:?}", result);

    Ok(())
}
```

## How Encryption Works

The filler pipeline handles encryption transparently:

```
1. You build a SeismicTransactionRequest with .seismic()
2. SeismicElementsFiller adds encryption_pubkey, nonce, block hash, expiry
3. SeismicGasFiller estimates gas
4. Provider encrypts calldata with AES-GCM:
   - Shared secret derived via ECDH (client private key + TEE public key)
   - AAD = RLP-encoded TxSeismicMetadata
   - Nonce = 12-byte random value
5. Encrypted transaction is signed and broadcast
6. TEE decrypts calldata inside the enclave
7. For signed reads: TEE encrypts the response, provider decrypts it
```

{% hint style="info" %}
You never need to call encryption functions manually. The provider's filler pipeline handles all cryptographic operations when you use `.seismic()`.
{% endhint %}

## Shielded Write vs. Signed Read

| Aspect             | Shielded Write        | Signed Read          |
| ------------------ | --------------------- | -------------------- |
| Method             | `send_transaction()`  | `seismic_call()`     |
| State changes      | Yes                   | No                   |
| Calldata encrypted | Yes                   | Yes                  |
| Response encrypted | N/A (returns receipt) | Yes (auto-decrypted) |
| Gas cost           | Yes                   | No (simulated)       |
| `signed_read` flag | `false`               | `true`               |

## Important Notes

- **Create transactions cannot be seismic.** Contract deployment must use standard (non-seismic) transactions. Deploy first, then interact with shielded calls. See [Transparent Calls](transparent-calls.md) for deployment patterns.
- **Signed provider required.** Both shielded writes and signed reads need a `SeismicSignedProvider`. An unsigned provider cannot perform shielded operations.
- **Automatic response decryption.** When using `seismic_call()`, the provider automatically decrypts the TEE's encrypted response.
- **Shielded types in ABI.** The `sol!` macro handles shielded Solidity types (`suint256`, `sbool`, `saddress`) -- they map to their standard ABI counterparts for encoding.

## See Also

- [Transparent Calls](transparent-calls.md) -- Non-encrypted contract interactions
- [Contract Interaction Overview](./) -- Comparison of all interaction patterns
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) -- Provider required for shielded operations
- [TxSeismic](../transaction-types/tx-seismic.md) -- Underlying transaction type
- [TxSeismicElements](../transaction-types/tx-seismic-elements.md) -- Encryption metadata
- [Encryption](../provider/encryption.md) -- Detailed encryption pipeline
