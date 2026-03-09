---
description: Reading and writing SRC20 token balances and allowances
icon: hand-holding-dollar
---

# Token Interaction

Read shielded balances and write shielded state changes on SRC20 tokens using seismic-alloy.

## Overview

SRC20 token interaction in Rust follows the same builder pattern as all seismic-alloy contract calls:

- **Transparent reads** (metadata) -- Use `provider.call()` without `.seismic()`
- **Signed reads** (balances, allowances) -- Use `provider.seismic_call()` with `.seismic()`
- **Shielded writes** (transfers, approvals) -- Use `provider.send_transaction()` with `.seismic()`

## Defining the Interface

```rust
use alloy::sol;

sol! {
    interface ISRC20 {
        function name() public view returns (string);
        function symbol() public view returns (string);
        function decimals() public view returns (uint8);
        function totalSupply() public view returns (uint256);

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

## Reading Token Metadata

Token metadata (name, symbol, decimals, totalSupply) is not shielded. Use a plain transparent read:

```rust
use seismic_alloy::prelude::*;
use alloy::providers::Provider;
use alloy::sol_types::SolCall;
use alloy_primitives::Address;
use alloy_rpc_types_eth::TransactionRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;

    // Read name
    let name_call = ISRC20::nameCall {};
    let result = provider
        .call(
            &TransactionRequest::default()
                .to(token)
                .input(name_call.abi_encode().into()),
        )
        .await?;
    let name = ISRC20::nameCall::abi_decode_returns(&result, true)?;
    println!("Name: {}", name._0);

    // Read symbol
    let symbol_call = ISRC20::symbolCall {};
    let result = provider
        .call(
            &TransactionRequest::default()
                .to(token)
                .input(symbol_call.abi_encode().into()),
        )
        .await?;
    let symbol = ISRC20::symbolCall::abi_decode_returns(&result, true)?;
    println!("Symbol: {}", symbol._0);

    // Read decimals
    let decimals_call = ISRC20::decimalsCall {};
    let result = provider
        .call(
            &TransactionRequest::default()
                .to(token)
                .input(decimals_call.abi_encode().into()),
        )
        .await?;
    let decimals = ISRC20::decimalsCall::abi_decode_returns(&result, true)?;
    println!("Decimals: {}", decimals._0);

    Ok(())
}
```

{% hint style="info" %}
Metadata reads do not require a wallet or signed provider. An unsigned provider works fine since these values are not shielded.
{% endhint %}

## Reading Shielded Balances

`balanceOf()` is a shielded read. The contract uses `msg.sender` to authenticate the caller, so you must use `seismic_call()` (a signed read) on a `SeismicSignedProvider`.

```rust
use seismic_alloy::prelude::*;
use alloy::providers::Provider;
use alloy::sol_types::SolCall;
use alloy_primitives::{Address, U256};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let my_address = provider.default_signer_address();

    // Build the balanceOf call
    let balance_call = ISRC20::balanceOfCall {
        account: my_address,
    };

    // Mark as seismic for signed read
    let tx = TransactionRequest::default()
        .to(token)
        .input(balance_call.abi_encode().into())
        .seismic();

    // Execute signed read
    let result = provider.seismic_call(tx.into()).await?;
    let balance = U256::from_be_slice(&result);
    println!("My balance: {balance}");

    Ok(())
}
```

{% hint style="warning" %}
`balanceOf()` requires a **signed read** via `seismic_call()`. A plain `provider.call()` zeros out the `from` field, causing the contract to see the zero address as the sender and return its balance -- which is almost certainly zero.
{% endhint %}

## Reading Allowances

Allowance reads may also be shielded depending on the contract implementation. Use a signed read to be safe:

```rust
use seismic_alloy::prelude::*;
use alloy::providers::Provider;
use alloy::sol_types::SolCall;
use alloy_primitives::{Address, U256};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let owner = provider.default_signer_address();
    let spender: Address = "0xSPENDER_ADDRESS".parse()?;

    // Build the allowance call
    let allowance_call = ISRC20::allowanceCall { owner, spender };

    let tx = TransactionRequest::default()
        .to(token)
        .input(allowance_call.abi_encode().into())
        .seismic();

    let result = provider.seismic_call(tx.into()).await?;
    let allowance = U256::from_be_slice(&result);
    println!("Allowance: {allowance}");

    Ok(())
}
```

## Shielded Writes

### Transfer Tokens

Send tokens using a shielded write. The transfer amount is encrypted in calldata:

```rust
use seismic_alloy::prelude::*;
use alloy::providers::Provider;
use alloy::sol_types::SolCall;
use alloy_primitives::{Address, U256};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let recipient: Address = "0xRECIPIENT_ADDRESS".parse()?;
    let amount = U256::from(100);

    // Build the transfer call
    let transfer_call = ISRC20::transferCall {
        to: recipient,
        amount,
    };

    // Mark as seismic for encrypted calldata
    let tx = TransactionRequest::default()
        .to(token)
        .input(transfer_call.abi_encode().into())
        .seismic();

    // Send shielded transaction
    let pending_tx = provider.send_transaction(tx.into()).await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Transfer tx: {:?}", receipt.transaction_hash);

    Ok(())
}
```

### Approve a Spender

```rust
use seismic_alloy::prelude::*;
use alloy::providers::Provider;
use alloy::sol_types::SolCall;
use alloy_primitives::{Address, U256};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let spender: Address = "0xSPENDER_ADDRESS".parse()?;
    let amount = U256::from(500);

    // Build the approve call
    let approve_call = ISRC20::approveCall { spender, amount };

    let tx = TransactionRequest::default()
        .to(token)
        .input(approve_call.abi_encode().into())
        .seismic();

    let pending_tx = provider.send_transaction(tx.into()).await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Approve tx: {:?}", receipt.transaction_hash);

    Ok(())
}
```

## Operation Summary

| Operation                 | Method                        | Builder         | Provider Required       |
| ------------------------- | ----------------------------- | --------------- | ----------------------- |
| Read name/symbol/decimals | `provider.call()`             | No `.seismic()` | Any provider            |
| Read balanceOf            | `provider.seismic_call()`     | `.seismic()`    | `SeismicSignedProvider` |
| Read allowance            | `provider.seismic_call()`     | `.seismic()`    | `SeismicSignedProvider` |
| Transfer tokens           | `provider.send_transaction()` | `.seismic()`    | `SeismicSignedProvider` |
| Approve spender           | `provider.send_transaction()` | `.seismic()`    | `SeismicSignedProvider` |

## Notes

- All shielded operations require a `SeismicSignedProvider` with a wallet
- The `.seismic()` builder method marks the transaction for calldata encryption
- The filler pipeline automatically handles encryption nonce, TEE pubkey, and AES-GCM encryption
- Shielded types (`suint256`) appear as their standard counterparts (`uint256`) in the ABI encoding

## See Also

- [Transfers](transfers.md) -- Shielded transfer patterns and multi-step workflows
- [Event Decryption](event-decryption.md) -- Decrypting SRC20 events
- [Contract Interaction](../contract-interaction/) -- General shielded and transparent call patterns
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) -- Provider with signing capabilities
