---
description: Encrypted transactions -- lifecycle, security parameters, and the filler pipeline
icon: shield-halved
---

# Shielded Write

---

### How it works

When you build a transaction with `.seismic()` and send it via `send_transaction()`, the SDK:

1. Fetches your nonce and the latest block hash from the node
2. Populates `TxSeismicElements` (encryption nonce, TEE public key reference, block hash, expiry block)
3. Encrypts the calldata using AES-GCM with a shared key derived via ECDH between the provider's ephemeral keypair and the node's TEE public key
4. Signs and broadcasts the transaction as type `0x4A`

The encrypted calldata is bound to the transaction context (chain ID, nonce, block hash, expiry) via AES-GCM additional authenticated data, so it cannot be replayed or tampered with.

---

### Step-by-step

#### 1. Set up a signed provider

Shielded writes require a `SeismicSignedProvider` because you need a private key for both transaction signing and ECDH key derivation.

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::from(signer);
let url: reqwest::Url = "https://node.seismicdev.net/rpc".parse()?;

let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;
```

{% hint style="info" %}
`new()` is async because it fetches the TEE public key from the node and caches it for all subsequent encryption operations.
{% endhint %}

#### 2. Define the contract interface

Use Alloy's `sol!` macro to define your contract interface. Shielded Solidity types (`suint256`, `sbool`, etc.) map to their standard ABI counterparts for encoding:

```rust
use alloy::sol;

sol! {
    interface ISeismicCounter {
        function setNumber(suint256 newNumber) public;
        function increment() public;
        function isOdd() public view returns (bool);
    }
}
```

This generates type-safe Rust structs: `ISeismicCounter::setNumberCall`, `ISeismicCounter::incrementCall`, etc.

#### 3. Encode calldata

Use the generated structs and `abi_encode()` to produce calldata bytes:

```rust
use alloy::sol_types::SolCall;
use alloy_primitives::U256;

let calldata = ISeismicCounter::setNumberCall {
    newNumber: U256::from(42),
}.abi_encode();
```

#### 4. Build the transaction with `.seismic()`

The `.seismic()` method on a `SeismicTransactionRequest` marks it for encryption. Without `.seismic()`, the transaction is sent as a standard Ethereum type with no encryption.

```rust
use alloy_primitives::{Bytes, TxKind};

let tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
    .with_input(Bytes::from(calldata))
    .with_kind(TxKind::Call(contract_address))
    .into()
    .seismic();  // Marks for encryption
```

#### 5. Send and await receipt

```rust
let pending_tx = provider.send_transaction(tx.into()).await?;
let receipt = pending_tx.get_receipt().await?;
```

#### 6. Verify success

```rust
use alloy_network::ReceiptResponse;

assert!(receipt.status());
println!("Transaction hash: {:?}", receipt.transaction_hash);
println!("Block number: {:?}", receipt.block_number());
```

---

### Security parameters

Every shielded transaction includes a block-hash freshness check and an expiry window. The `SeismicElementsFiller` automatically populates these with sane defaults:

| Parameter           | Default                         | Description                                         |
| ------------------- | ------------------------------- | --------------------------------------------------- |
| `encryption_nonce`  | Random 12 bytes                 | Unique per-transaction AES-GCM nonce                |
| `encryption_pubkey` | Provider's ephemeral public key | Client's ECDH public key for key derivation         |
| `blocks_window`     | 100 blocks                      | Freshness window for the block hash                 |
| `expires_at_block`  | `current_block + blocks_window` | Block number after which the transaction is invalid |
| `recent_block_hash` | Latest block hash               | Anchors the transaction to a specific chain state   |

These values are set automatically by the filler pipeline. You do not need to set them manually.

---

### What happens under the hood

The filler pipeline processes your transaction in this order:

```
1. SeismicElementsFiller
   - Attaches encryption_pubkey, encryption_nonce, recent_block_hash, expires_at_block
   - Encrypts calldata with AES-GCM
   - AAD = RLP-encoded TxSeismicMetadata (binds ciphertext to tx context)

2. SeismicGasFiller
   - Estimates gas limit and gas price

3. NonceFiller + ChainIdFiller
   - Sets nonce and chain ID

4. WalletFiller
   - Signs the fully populated transaction
```

You never call encryption functions manually. The `.seismic()` marker tells the filler pipeline to handle everything.

{% hint style="info" %}
You never need to call encryption functions manually. The provider's filler pipeline handles all cryptographic operations when you use `.seismic()`.
{% endhint %}

---

### Create transactions cannot be seismic

Contract deployment (`TxKind::Create`) always uses transparent transactions. The Seismic protocol does not support encrypting deployment bytecode. Deploy your contract with a standard transaction, then interact with it using shielded calls:

```rust
// Deployment -- no .seismic()
let deploy_tx = seismic_foundry_tx_builder()
    .with_input(bytecode)
    .with_kind(TxKind::Create)
    .into();

let receipt = provider.send_transaction(deploy_tx).await?
    .get_receipt().await?;
let contract_address = receipt.contract_address.unwrap();

// Interaction -- with .seismic()
let shielded_tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
    .with_input(Bytes::from(calldata))
    .with_kind(TxKind::Call(contract_address))
    .into()
    .seismic();

provider.send_transaction(shielded_tx.into()).await?;
```

---

### Error handling

Common failure modes and how to handle them:

```rust
use alloy_network::ReceiptResponse;

// Send the transaction
let pending_tx = match provider.send_transaction(tx.into()).await {
    Ok(pending) => pending,
    Err(e) => {
        eprintln!("Failed to send transaction: {e}");
        return Err(e.into());
    }
};

// Wait for the receipt
let receipt = match pending_tx.get_receipt().await {
    Ok(receipt) => receipt,
    Err(e) => {
        eprintln!("Failed to get receipt: {e}");
        return Err(e.into());
    }
};

// Check status
if !receipt.status() {
    eprintln!("Transaction reverted: {:?}", receipt.transaction_hash);
    return Err("transaction reverted".into());
}

println!("Success! Block: {:?}", receipt.block_number());
```

---

### Complete example

```rust
use seismic_alloy::prelude::*;
use alloy::sol;
use alloy::sol_types::SolCall;
use alloy_primitives::{Bytes, TxKind, U256};
use alloy_signer_local::PrivateKeySigner;
use alloy_network::ReceiptResponse;

sol! {
    interface ISeismicCounter {
        function setNumber(suint256 newNumber) public;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Set up signed provider
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    // 2. Encode calldata
    let contract_address = std::env::var("CONTRACT_ADDRESS")?.parse()?;
    let calldata = ISeismicCounter::setNumberCall {
        newNumber: U256::from(42),
    }.abi_encode();

    // 3. Build seismic transaction
    let tx: SeismicTransactionRequest = seismic_foundry_tx_builder()
        .with_input(Bytes::from(calldata))
        .with_kind(TxKind::Call(contract_address))
        .into()
        .seismic();

    // 4. Send and wait
    let pending_tx = provider.send_transaction(tx.into()).await?;
    let receipt = pending_tx.get_receipt().await?;

    // 5. Verify
    assert!(receipt.status());
    println!("Shielded write confirmed: {:?}", receipt.transaction_hash);
    println!("Block: {:?}", receipt.block_number());

    Ok(())
}
```

## See Also

- [Signed Reads](signed-reads.md) - Encrypted `eth_call` operations
- [Shielded Calls](../contract-interaction/shielded-calls.md) - API reference for shielded operations
- [Transparent Calls](../contract-interaction/transparent-calls.md) - Non-encrypted operations
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) - Provider required for shielded writes
- [Encryption](../provider/encryption.md) - Detailed encryption pipeline
