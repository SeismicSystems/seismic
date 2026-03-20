---
description: Encrypted transactions -- lifecycle, security parameters, and the filler pipeline
icon: shield-halved
---

# Shielded Write

---

### How it works

When you send a shielded write (either via auto-encryption for functions with shielded params, or via `.seismic().send()` for other functions), the SDK:

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
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
let wallet = SeismicWallet::<SeismicReth>::from(signer);
let url: reqwest::Url = "https://gcp-1.seismictest.net/rpc".parse()?;

let provider = SeismicProviderBuilder::new()
    .wallet(wallet)
    .connect_http(url)
    .await?;
```

{% hint style="info" %}
`connect_http()` is async because it fetches the TEE public key from the node and caches it for all subsequent encryption operations.
{% endhint %}

#### 2. Define the contract interface

Use Alloy's `sol!` macro with `#[sol(rpc)]` to define your contract interface. Shielded Solidity types (`suint256`, `sbool`, etc.) map to their standard ABI counterparts for encoding:

```rust
// sol! is included in the prelude

sol! {
    #[sol(rpc)]
    contract SeismicCounter {
        function setNumber(suint256 newNumber) public;
        function increment() public;
        function isOdd() public view returns (bool);
    }
}
```

This generates type-safe call builders with `.call()` and `.send()` methods.

#### 3. Build and send

Functions with shielded parameters (like `setNumber(suint256)`) auto-encrypt -- just call `.send()` directly. For functions without shielded parameters (like `increment()`), use `.seismic().send()` to opt into encryption:

```rust
let contract = SeismicCounter::new(contract_address, &provider);

// setNumber has a shielded param (suint256) -- auto-encrypts
let receipt = contract
    .setNumber(U256::from(42).into())
    .send()
    .await?
    .get_receipt()
    .await?;

// increment has no shielded params -- use .seismic() to encrypt
let receipt = contract
    .increment()
    .seismic()
    .send()
    .await?
    .get_receipt()
    .await?;
```

#### 4. Verify success

```rust
// ReceiptResponse is included in the prelude

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

These values are set automatically by the filler pipeline. You can override them per-call:

```rust
// setNumber auto-encrypts (shielded param) -- security params available directly
let receipt = contract
    .setNumber(U256::from(42).into())
    .expires_at(current_block + 50)        // Custom expiration
    .recent_block_hash(specific_hash)       // Pin to specific chain state
    .send()
    .await?
    .get_receipt()
    .await?;
```

---

### What happens under the hood

The filler pipeline processes your transaction in this order:

```
1. WalletFiller
   - Sets the `from` field on the transaction

2. NonceFiller + ChainIdFiller
   - Sets nonce and chain ID

3. SeismicElementsFiller
   - Attaches encryption_pubkey, encryption_nonce, recent_block_hash, expires_at_block
   - Encrypts calldata with AES-GCM
   - AAD = RLP-encoded TxSeismicMetadata (binds ciphertext to tx context)

4. SeismicGasFiller
   - Estimates gas limit and gas price
```

You never call encryption functions manually. For functions with shielded parameters, the `ShieldedCallBuilder` handles everything automatically. For other functions, the `.seismic()` marker tells the filler pipeline to handle everything.

{% hint style="info" %}
You never need to call encryption functions manually. The provider's filler pipeline handles all cryptographic operations -- either automatically for functions with shielded parameters, or when you use `.seismic()`.
{% endhint %}

---

### Create transactions cannot be seismic

Contract deployment (`TxKind::Create`) always uses transparent transactions. The Seismic protocol does not support encrypting deployment bytecode. Deploy your contract first, then interact with it using shielded calls:

```rust
// Deploy with #[sol(rpc, bytecode = "...")] generated deploy() method
let contract = SeismicCounter::deploy(&provider).await?;

// Now interact with shielded calls
// setNumber auto-encrypts (shielded param)
contract.setNumber(U256::from(42).into())
    .send()
    .await?;
```

---

### Error handling

Common failure modes and how to handle them:

```rust
use alloy_network::ReceiptResponse;

// Send the shielded transaction
// setNumber auto-encrypts (shielded param)
let pending_tx = match contract.setNumber(U256::from(42).into()).send().await {
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
    // 1. Set up signed provider
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = SeismicWallet::<SeismicReth>::from(signer);
    let url: reqwest::Url = std::env::var("RPC_URL")?.parse()?;

    let provider = SeismicProviderBuilder::new()
        .wallet(wallet)
        .connect_http(url)
        .await?;

    // 2. Shielded write -- setNumber auto-encrypts (shielded param)
    let contract_address = std::env::var("CONTRACT_ADDRESS")?.parse()?;
    let contract = SeismicCounter::new(contract_address, &provider);

    let receipt = contract
        .setNumber(U256::from(42).into())
        .send()
        .await?
        .get_receipt()
        .await?;

    // 3. Verify
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
