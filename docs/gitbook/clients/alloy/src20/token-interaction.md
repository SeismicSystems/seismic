---
description: Reading and writing SRC20 token balances and allowances
icon: hand-holding-dollar
---

# Token Interaction

Read shielded balances and write shielded state changes on SRC20 tokens using seismic-alloy.

## Overview

SRC20 token interaction in Rust follows the same builder pattern as all seismic-alloy contract calls:

- **Transparent reads** (metadata) — Use `contract.name().call()` without `.seismic()`
- **Signed reads** (balances, allowances) — Use `contract.balanceOf(addr).seismic().call()` with `.seismic()` (no shielded params in arguments)
- **Shielded writes** (transfers, approvals) — Use `contract.transfer(to, amount).send()` directly (auto-encrypts because `suint256` is a shielded param)

## Defining the Interface

```rust
use seismic_prelude::client::*;

sol! {
    #[sol(rpc)]
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
use seismic_prelude::client::*;

sol! {
    #[sol(rpc)]
    interface ISRC20 {
        function name() public view returns (string);
        function symbol() public view returns (string);
        function decimals() public view returns (uint8);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://testnet-1.seismictest.net/rpc".parse()?;
    // Unsigned provider -- connect_http is synchronous
    let provider = SeismicProviderBuilder::new().connect_http(url);

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let contract = ISRC20::new(token, &provider);

    // Read name
    let name = contract.name().call().await?;
    println!("Name: {}", name._0);

    // Read symbol
    let symbol = contract.symbol().call().await?;
    println!("Symbol: {}", symbol._0);

    // Read decimals
    let decimals = contract.decimals().call().await?;
    println!("Decimals: {}", decimals._0);

    Ok(())
}
```

{% hint style="info" %}
Metadata reads do not require a wallet or signed provider. An unsigned provider works fine since these values are not shielded.
{% endhint %}

## Reading Shielded Balances

`balanceOf()` is a shielded read. The contract uses `msg.sender` to authenticate the caller, so you must use `.seismic().call()` (a signed read) on a provider created with a wallet.

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

sol! {
    #[sol(rpc)]
    interface ISRC20 {
        function balanceOf(address account) public view returns (suint256);
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

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let my_address = provider.default_signer_address();
    let contract = ISRC20::new(token, &provider);

    // Signed read via .seismic()
    let balance = contract.balanceOf(my_address).seismic().call().await?;
    println!("My balance: {}", balance._0);

    Ok(())
}
```

{% hint style="warning" %}
`balanceOf()` requires a **signed read** via `.seismic().call()`. A plain `.call()` without `.seismic()` zeros out the `from` field, causing the contract to see the zero address as the sender and return its balance — which is almost certainly zero.
{% endhint %}

## Reading Allowances

Allowance reads may also be shielded depending on the contract implementation. Use a signed read to be safe:

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

sol! {
    #[sol(rpc)]
    interface ISRC20 {
        function allowance(address owner, address spender) public view returns (suint256);
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

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let owner = provider.default_signer_address();
    let spender: Address = "0xSPENDER_ADDRESS".parse()?;
    let contract = ISRC20::new(token, &provider);

    // Signed read via .seismic()
    let allowance = contract.allowance(owner, spender).seismic().call().await?;
    println!("Allowance: {}", allowance._0);

    Ok(())
}
```

## Shielded Writes

### Transfer Tokens

Send tokens using a shielded write. The transfer amount is encrypted in calldata:

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

sol! {
    #[sol(rpc)]
    interface ISRC20 {
        function transfer(address to, suint256 amount) public returns (bool);
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

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let recipient: Address = "0xRECIPIENT_ADDRESS".parse()?;
    let amount = U256::from(100);
    let contract = ISRC20::new(token, &provider);

    // transfer has suint256 param -- auto-encrypts
    let pending_tx = contract.transfer(recipient, amount).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Transfer tx: {:?}", receipt.transaction_hash);

    Ok(())
}
```

### Approve a Spender

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

sol! {
    #[sol(rpc)]
    interface ISRC20 {
        function approve(address spender, suint256 amount) public returns (bool);
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

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let spender: Address = "0xSPENDER_ADDRESS".parse()?;
    let amount = U256::from(500);
    let contract = ISRC20::new(token, &provider);

    // approve has suint256 param -- auto-encrypts
    let pending_tx = contract.approve(spender, amount).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Approve tx: {:?}", receipt.transaction_hash);

    Ok(())
}
```

## Operation Summary

| Operation                 | Method                                     | Builder         | Provider Required |
| ------------------------- | ------------------------------------------ | --------------- | ----------------- |
| Read name/symbol/decimals | `contract.name().call()`                   | No encryption   | Any provider      |
| Read balanceOf            | `contract.balanceOf(addr).seismic().call()` | `.seismic()`    | Signed provider   |
| Read allowance            | `contract.allowance(o, s).seismic().call()` | `.seismic()`    | Signed provider   |
| Transfer tokens           | `contract.transfer(to, amt).send()`         | Auto-encrypts   | Signed provider   |
| Approve spender           | `contract.approve(s, amt).send()`           | Auto-encrypts   | Signed provider   |

## Notes

- All shielded operations require a provider built with a wallet via `SeismicProviderBuilder`
- Functions with shielded parameters (`suint256`) like `transfer` and `approve` auto-encrypt via `ShieldedCallBuilder`
- For functions without shielded params (like `balanceOf` and `allowance`), use `.seismic()` to opt into encryption
- The filler pipeline automatically handles encryption nonce, TEE pubkey, and AES-GCM encryption
- Shielded types (`suint256`) appear as their standard counterparts (`uint256`) in the ABI encoding

## See Also

- [Transfers](transfers.md) — Shielded transfer patterns and multi-step workflows
- [Event Decryption](event-decryption.md) — Decrypting SRC20 events
- [Contract Interaction](../contract-interaction/) — General shielded and transparent call patterns
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) — Provider with signing capabilities
