---
description: Encrypted writes and signed reads using seismic-alloy
icon: shield-halved
---

# Shielded Calls

Privacy-preserving contract interactions where calldata (and optionally return data) is encrypted using the TEE's public key.

## Overview

Shielded calls use the Seismic transaction type (`0x4A`) to encrypt calldata before it reaches the network. Functions with shielded parameters (e.g., `suint256`, `saddress`) auto-encrypt via `ShieldedCallBuilder` -- you can call `.send()` or `.call()` directly. For non-shielded functions that still need encryption (e.g., `isOdd()`, `increment()`), use `.seismic()` to opt in.

There are two shielded operations:

- **Shielded Write** -- Encrypted `send_transaction` that modifies on-chain state
- **Signed Read** -- Encrypted `eth_call` that reads state without modification

Both require a `SeismicSignedProvider` because they need a private key for ECDH key derivation and transaction signing.

## Defining a Contract Interface

Use Alloy's `sol!` macro with `#[sol(rpc)]` to define your contract interface:

```rust
use seismic_prelude::client::*;
// The prelude includes both SeismicCallExt (adds .seismic() to SolCallBuilder)
// and ShieldedCallExt (adds .call(), .send(), .eip712() to ShieldedCallBuilder).
// You don't need to import them individually.

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

This generates type-safe call builders for each function. The `#[sol(rpc)]` attribute adds `.call()` and `.send()` methods. Functions with shielded parameters (like `setNumber(suint256)`) automatically return a `ShieldedCallBuilder` that encrypts on `.call()` or `.send()`. For functions without shielded parameters (like `isOdd()` and `increment()`), use `SeismicCallExt` to call `.seismic()` and opt into encryption.

{% hint style="warning" %}
Calling `.seismic()` on a function that already has shielded parameters is `#[deprecated]` and produces a compiler warning -- it's redundant because the call builder is already a `ShieldedCallBuilder`. Call `.call()` or `.send()` directly on those functions.
{% endhint %}

## Shielded Write

A shielded write sends an encrypted transaction that modifies on-chain state. The calldata is encrypted with AES-GCM using a shared secret derived from the TEE's public key.

```rust
let contract = SeismicCounter::new(contract_address, &provider);

// setNumber has a shielded param (suint256), so it auto-encrypts -- no .seismic() needed
let receipt = contract
    .setNumber(alloy_primitives::aliases::SUInt(U256::from(42)))
    .send()
    .await?
    .get_receipt()
    .await?;

println!("Transaction hash: {:?}", receipt.transaction_hash);
```

### Complete Example

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;
use alloy_network::ReceiptResponse;

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
    let url = "https://testnet-1.seismictest.net/rpc".parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;

    let contract = SeismicCounter::new(contract_address, &provider);

    // setNumber has a shielded param (suint256) -- auto-encrypts
    let receipt = contract
        .setNumber(alloy_primitives::aliases::SUInt(U256::from(42)))
        .send()
        .await?
        .get_receipt()
        .await?;
    println!("Shielded write confirmed: {:?} (status: {})", receipt.transaction_hash, receipt.status());

    Ok(())
}
```

## Signed Read

A signed read executes an encrypted `eth_call` that proves the caller's identity to the TEE. This is required for reading private state that is access-controlled by `msg.sender`.

```rust
let contract = SeismicCounter::new(contract_address, &provider);

// Encrypted call + automatic response decryption
let is_odd = contract.isOdd().seismic().call().await?;
println!("Is odd: {is_odd}");
```

### Complete Example

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
    let url = "https://testnet-1.seismictest.net/rpc".parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;

    let contract = SeismicCounter::new(contract_address, &provider);

    let is_odd = contract.isOdd().seismic().call().await?;
    println!("Is odd: {is_odd}");

    Ok(())
}
```

## SecurityParams (Per-Call Overrides)

Customize encryption parameters on individual calls:

```rust
let is_odd = contract.isOdd()
    .seismic()
    .expires_at(current_block + 50)       // Custom expiration
    .recent_block_hash(block_hash)         // Pin to specific chain state
    .call()
    .await?;
```

| Method                       | Description                                    | Default                     |
| ---------------------------- | ---------------------------------------------- | --------------------------- |
| `.expires_at(block_number)`  | Set transaction expiration block               | Current block + 100         |
| `.recent_block_hash(hash)`   | Pin to a specific chain state                  | Latest block hash           |
| `.encryption_nonce(nonce)`   | Override the AEAD nonce (testing only)          | Random                      |

You can also use `SecurityParams` directly with the provider-level `_with` methods:

```rust
use seismic_prelude::client::*;

let result = provider.seismic_call_with(addr, MyContract::isOddCall {},
    SecurityParams::default().expires_at(current_block + 10)
).await?;

let receipt = provider.seismic_send_with(addr, MyContract::setNumberCall { newNumber: alloy_primitives::aliases::SUInt(U256::from(42)) },
    SecurityParams::default().expires_at(current_block + 50)
).await?.get_receipt().await?;
```

## EIP-712 (Browser Wallet Compatibility)

For wallets that cannot sign custom RLP-encoded transaction types (e.g., MetaMask), use EIP-712 typed data signing:

```rust
// EIP-712 shielded write -- setNumber auto-encrypts (shielded param)
contract.setNumber(alloy_primitives::aliases::SUInt(U256::from(42)))
    .eip712()
    .send()
    .await?;

// EIP-712 shielded read -- isOdd needs .seismic() (no shielded params)
let is_odd = contract.isOdd()
    .seismic()
    .eip712()
    .call()
    .await?;
```

## How Encryption Works

The filler pipeline handles encryption transparently:

```
1. You call .call() or .send() on a ShieldedCallBuilder (auto or via .seismic())
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
You never need to call encryption functions manually. The provider's filler pipeline handles all cryptographic operations when you use `.seismic()` or when the function has shielded parameters.
{% endhint %}

## Shielded Write vs. Signed Read

| Aspect             | Shielded Write        | Signed Read          |
| ------------------ | --------------------- | -------------------- |
| Method (no shielded params) | `.seismic().send()` | `.seismic().call()` |
| Method (shielded params) | `.send()` (auto-encrypts) | `.call()` (auto-encrypts) |
| State changes      | Yes                   | No                   |
| Calldata encrypted | Yes                   | Yes                  |
| Response encrypted | N/A (returns receipt) | Yes (auto-decrypted) |
| Gas cost           | Yes                   | No (simulated)       |
| `signed_read` flag | `false`               | `true`               |

{% hint style="info" %}
For functions with shielded parameters (e.g., `suint256`, `saddress`), the `sol!` macro automatically returns a `ShieldedCallBuilder` -- you can call `.send()` or `.call()` directly without `.seismic()`. Use `.seismic()` only for functions without shielded parameters that still need encryption.
{% endhint %}

## Low-Level Alternative

If you need direct control without the `#[sol(rpc)]` call builder pattern, use `SignedProviderExt` methods:

```rust
// SignedProviderExt is included in the prelude

// Shielded read (high-level -- ABI encodes/decodes automatically)
let is_odd: bool = provider.seismic_call(addr, SeismicCounter::isOddCall {}).await?;

// Shielded write (high-level -- ABI encodes automatically)
let pending = provider.seismic_send(addr, SeismicCounter::setNumberCall {
    newNumber: alloy_primitives::aliases::SUInt(U256::from(42)),
}).await?;

// With custom SecurityParams
use seismic_prelude::client::*;

let result = provider.seismic_call_with(addr, SeismicCounter::isOddCall {},
    SecurityParams::default().expires_at(current_block + 10)
).await?;
```

`seismic_call_raw` is the low-level variant that takes a `SendableTx` and returns raw `Bytes` without ABI decoding.

## Important Notes

- **Create transactions cannot be seismic.** Contract deployment must use standard (non-seismic) transactions. Deploy first, then interact with shielded calls. See [Transparent Calls](transparent-calls.md) for deployment patterns.
- **Signed provider required.** Both shielded writes and signed reads need a `SeismicSignedProvider`. An unsigned provider cannot perform shielded operations.
- **Automatic response decryption.** When using `.call()` on a `ShieldedCallBuilder`, the provider automatically decrypts the TEE's encrypted response.
- **Shielded types in ABI.** The `sol!` macro handles shielded Solidity types (`suint256`, `sbool`, `saddress`) -- they map to their standard ABI counterparts for encoding. Functions with shielded types in their arguments automatically return a `ShieldedCallBuilder`, so no `.seismic()` is needed.

## See Also

- [Transparent Calls](transparent-calls.md) -- Non-encrypted contract interactions
- [Contract Interaction Overview](./) -- Comparison of all interaction patterns
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) -- Provider required for shielded operations
- [TxSeismic](../transaction-types/tx-seismic.md) -- Underlying transaction type
- [TxSeismicElements](../transaction-types/tx-seismic-elements.md) -- Encryption metadata
- [Encryption](../provider/encryption.md) -- Detailed encryption pipeline
