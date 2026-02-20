---
description: Shielded transfer patterns for SRC20 tokens
icon: arrow-right-arrow-left
---

# Transfers

Send SRC20 tokens privately using shielded transfers, approvals, and `transferFrom` patterns.

## Overview

SRC20 transfers work similarly to ERC20 transfers, but with calldata encryption to hide amounts from observers. The `SeismicSignedProvider` handles encryption automatically when you mark a transaction with `.seismic()`.

## Prerequisites

All transfer examples require a `SeismicSignedProvider` and the SRC20 interface definition:

```rust
use seismic_alloy::prelude::*;
use alloy::sol;
use alloy::providers::Provider;
use alloy::sol_types::SolCall;
use alloy_primitives::{Address, U256};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;

sol! {
    interface ISRC20 {
        function balanceOf(address account) public view returns (suint256);
        function transfer(address to, suint256 amount) public returns (bool);
        function approve(address spender, suint256 amount) public returns (bool);
        function allowance(address owner, address spender) public view returns (suint256);
        function transferFrom(address from, address to, suint256 amount) public returns (bool);

        event Transfer(address indexed from, address indexed to, suint256 value);
        event Approval(address indexed owner, address indexed spender, suint256 value);
    }
}
```

## Direct Transfer

The simplest pattern: transfer tokens directly from your wallet to a recipient.

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let recipient: Address = "0xRECIPIENT_ADDRESS".parse()?;
    let amount = U256::from(100);

    // Build and send the shielded transfer
    let transfer_call = ISRC20::transferCall {
        to: recipient,
        amount,
    };

    let tx = TransactionRequest::default()
        .to(token)
        .input(transfer_call.abi_encode().into())
        .seismic();

    let pending_tx = provider.send_transaction(tx.into()).await?;
    let receipt = pending_tx.get_receipt().await?;

    println!("Transfer sent: {:?}", receipt.transaction_hash);
    println!("Status: {:?}", receipt.status());

    Ok(())
}
```

## Approval + TransferFrom

The two-step pattern for delegated transfers: first approve a spender, then the spender calls `transferFrom`.

### Step 1: Owner Approves Spender

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Owner's provider
    let owner_signer: PrivateKeySigner = "0xOWNER_PRIVATE_KEY".parse()?;
    let owner_wallet = SeismicWallet::from(owner_signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let owner_provider =
        SeismicSignedProvider::<SeismicReth>::new(owner_wallet, url).await?;

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let spender: Address = "0xSPENDER_ADDRESS".parse()?;
    let approval_amount = U256::from(1000);

    // Approve the spender
    let approve_call = ISRC20::approveCall {
        spender,
        amount: approval_amount,
    };

    let tx = TransactionRequest::default()
        .to(token)
        .input(approve_call.abi_encode().into())
        .seismic();

    let pending_tx = owner_provider.send_transaction(tx.into()).await?;
    let receipt = pending_tx.get_receipt().await?;

    println!("Approval tx: {:?}", receipt.transaction_hash);

    Ok(())
}
```

### Step 2: Spender Executes TransferFrom

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Spender's provider
    let spender_signer: PrivateKeySigner = "0xSPENDER_PRIVATE_KEY".parse()?;
    let spender_wallet = SeismicWallet::from(spender_signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let spender_provider =
        SeismicSignedProvider::<SeismicReth>::new(spender_wallet, url).await?;

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let owner: Address = "0xOWNER_ADDRESS".parse()?;
    let recipient: Address = "0xRECIPIENT_ADDRESS".parse()?;
    let amount = U256::from(250);

    // Transfer on behalf of the owner
    let transfer_from_call = ISRC20::transferFromCall {
        from: owner,
        to: recipient,
        amount,
    };

    let tx = TransactionRequest::default()
        .to(token)
        .input(transfer_from_call.abi_encode().into())
        .seismic();

    let pending_tx = spender_provider.send_transaction(tx.into()).await?;
    let receipt = pending_tx.get_receipt().await?;

    println!("TransferFrom tx: {:?}", receipt.transaction_hash);

    Ok(())
}
```

## Check Balance Before Transfer

Always verify sufficient balance before sending a transfer to avoid wasted gas:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let my_address = provider.default_signer_address();
    let recipient: Address = "0xRECIPIENT_ADDRESS".parse()?;
    let amount = U256::from(100);

    // Step 1: Check balance via signed read
    let balance_call = ISRC20::balanceOfCall {
        account: my_address,
    };

    let balance_tx = TransactionRequest::default()
        .to(token)
        .input(balance_call.abi_encode().into())
        .seismic();

    let balance_result = provider.seismic_call(balance_tx.into()).await?;
    let balance = U256::from_be_slice(&balance_result);

    if balance < amount {
        println!("Insufficient balance: have {balance}, need {amount}");
        return Ok(());
    }

    // Step 2: Execute the transfer
    let transfer_call = ISRC20::transferCall {
        to: recipient,
        amount,
    };

    let tx = TransactionRequest::default()
        .to(token)
        .input(transfer_call.abi_encode().into())
        .seismic();

    let pending_tx = provider.send_transaction(tx.into()).await?;
    let receipt = pending_tx.get_receipt().await?;

    println!("Transfer successful: {:?}", receipt.transaction_hash);

    Ok(())
}
```

## Full Approve-Check-Transfer Workflow

A complete workflow showing approval, allowance check, and delegated transfer:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup owner and spender providers
    let owner_signer: PrivateKeySigner = "0xOWNER_PRIVATE_KEY".parse()?;
    let owner_wallet = SeismicWallet::from(owner_signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let owner_provider =
        SeismicSignedProvider::<SeismicReth>::new(owner_wallet, url.clone()).await?;

    let spender_signer: PrivateKeySigner = "0xSPENDER_PRIVATE_KEY".parse()?;
    let spender_wallet = SeismicWallet::from(spender_signer);
    let spender_provider =
        SeismicSignedProvider::<SeismicReth>::new(spender_wallet, url).await?;

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let owner_address = owner_provider.default_signer_address();
    let spender_address = spender_provider.default_signer_address();
    let recipient: Address = "0xRECIPIENT_ADDRESS".parse()?;

    // Step 1: Owner approves spender for 1000 tokens
    let approve_call = ISRC20::approveCall {
        spender: spender_address,
        amount: U256::from(1000),
    };

    let approve_tx = TransactionRequest::default()
        .to(token)
        .input(approve_call.abi_encode().into())
        .seismic();

    let pending = owner_provider.send_transaction(approve_tx.into()).await?;
    pending.get_receipt().await?;
    println!("Approved spender for 1000 tokens");

    // Step 2: Check allowance via signed read
    let allowance_call = ISRC20::allowanceCall {
        owner: owner_address,
        spender: spender_address,
    };

    let allowance_tx = TransactionRequest::default()
        .to(token)
        .input(allowance_call.abi_encode().into())
        .seismic();

    let allowance_result = owner_provider.seismic_call(allowance_tx.into()).await?;
    let allowance = U256::from_be_slice(&allowance_result);
    println!("Current allowance: {allowance}");

    // Step 3: Spender transfers 250 from owner to recipient
    let transfer_from_call = ISRC20::transferFromCall {
        from: owner_address,
        to: recipient,
        amount: U256::from(250),
    };

    let transfer_tx = TransactionRequest::default()
        .to(token)
        .input(transfer_from_call.abi_encode().into())
        .seismic();

    let pending = spender_provider.send_transaction(transfer_tx.into()).await?;
    let receipt = pending.get_receipt().await?;
    println!("TransferFrom tx: {:?}", receipt.transaction_hash);

    Ok(())
}
```

## Batch Transfers

Send tokens to multiple recipients in sequence:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;

    let recipients: Vec<(Address, U256)> = vec![
        ("0xADDRESS_1".parse()?, U256::from(100)),
        ("0xADDRESS_2".parse()?, U256::from(200)),
        ("0xADDRESS_3".parse()?, U256::from(300)),
    ];

    for (recipient, amount) in &recipients {
        let transfer_call = ISRC20::transferCall {
            to: *recipient,
            amount: *amount,
        };

        let tx = TransactionRequest::default()
            .to(token)
            .input(transfer_call.abi_encode().into())
            .seismic();

        let pending_tx = provider.send_transaction(tx.into()).await?;
        let receipt = pending_tx.get_receipt().await?;

        println!(
            "Sent {amount} to {recipient}: {:?}",
            receipt.transaction_hash,
        );
    }

    Ok(())
}
```

## Notes

- All transfer amounts are encrypted via the `.seismic()` builder
- The `SeismicSignedProvider` filler pipeline handles calldata encryption automatically
- Transaction receipts are returned as normal -- only the calldata is encrypted
- `transferFrom` requires prior approval from the token owner
- Each transaction has its own encryption nonce managed by the filler pipeline

## Warnings

- **Insufficient balance** -- The transaction will revert on-chain if the sender does not have enough tokens. Check the balance first to avoid wasted gas.
- **Insufficient allowance** -- `transferFrom` reverts if the spender's allowance is less than the transfer amount
- **Nonce management** -- When sending multiple transactions rapidly, the `NonceFiller` handles nonce assignment. Await each transaction's receipt before sending the next to avoid nonce conflicts.

## See Also

- [Token Interaction](token-interaction.md) -- Reading balances and metadata
- [Event Decryption](event-decryption.md) -- Decrypting Transfer events
- [Contract Interaction](../contract-interaction/) -- General call patterns
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) -- Required provider type
