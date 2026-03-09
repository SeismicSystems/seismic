---
description: Privacy-preserving ERC20 tokens with shielded balances
icon: coins
---

# SRC20 Tokens

SRC20 is Seismic's privacy-preserving ERC20 standard. Balances and transfer amounts use shielded types (`suint256`), so they are hidden from external observers. The protocol ensures that only authorized parties -- the token holder or those with a viewing key -- can read balances and decode transfer events.

## What Makes SRC20 Different from ERC20

| Feature              | ERC20                           | SRC20                                                            |
| -------------------- | ------------------------------- | ---------------------------------------------------------------- |
| **Balances**         | Public (anyone can read)        | Shielded (`suint256`) -- only the owner can read via signed read |
| **Transfer amounts** | Public in transaction data      | Encrypted calldata via shielded writes                           |
| **Events**           | Public (Transfer, Approval)     | Encrypted -- require viewing key to decrypt                      |
| **Allowances**       | Public                          | Shielded (`suint256`)                                            |
| **Token metadata**   | Public (name, symbol, decimals) | Public (not shielded)                                            |

{% hint style="info" %}
SRC20 contracts are deployed like any other contract on Seismic. The shielding is handled at the EVM level -- the contract uses `suint256` for sensitive fields, and the Seismic node's TEE ensures values are encrypted in storage and transit.
{% endhint %}

## Contract Interface

Define the SRC20 interface using Alloy's `sol!` macro:

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

## Architecture

```
SRC20 Token Contract (on-chain, Mercury EVM)
  |
  |-- Public metadata: name(), symbol(), decimals(), totalSupply()
  |     -> Transparent reads (no encryption needed)
  |
  |-- Shielded balances: balanceOf(address)
  |     -> Signed reads via seismic_call() (identity-proven eth_call)
  |     -> Contract uses msg.sender to gate access
  |
  |-- Shielded writes: transfer(), approve(), transferFrom()
  |     -> Calldata encrypted via .seismic() builder
  |     -> Amounts invisible to observers
  |
  |-- Encrypted events: Transfer, Approval
  |     -> Event data contains encrypted suint256 values
  |     -> Viewing key required to decrypt
```

## Quick Start

```rust
use seismic_alloy::prelude::*;
use alloy::sol;
use alloy::providers::Provider;
use alloy_primitives::{Address, U256};
use alloy_signer_local::PrivateKeySigner;

sol! {
    interface ISRC20 {
        function name() public view returns (string);
        function symbol() public view returns (string);
        function decimals() public view returns (uint8);
        function balanceOf(address account) public view returns (suint256);
        function transfer(address to, suint256 amount) public returns (bool);
        function approve(address spender, suint256 amount) public returns (bool);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = SeismicSignedProvider::<SeismicReth>::new(wallet, url).await?;

    let token_address: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;

    // Read public metadata (transparent read)
    let name_call = ISRC20::nameCall {};
    let name_result = provider
        .call(
            &alloy_rpc_types_eth::TransactionRequest::default()
                .to(token_address)
                .input(name_call.abi_encode().into()),
        )
        .await?;
    println!("Token name: {:?}", String::from_utf8_lossy(&name_result));

    // Read shielded balance (signed read)
    let balance_call = ISRC20::balanceOfCall {
        account: provider.default_signer_address(),
    };
    let tx = alloy_rpc_types_eth::TransactionRequest::default()
        .to(token_address)
        .input(balance_call.abi_encode().into())
        .seismic();
    let balance_result = provider.seismic_call(tx.into()).await?;
    let balance = U256::from_be_slice(&balance_result);
    println!("Balance: {balance}");

    Ok(())
}
```

## Navigation

| Page                                      | Description                                                       |
| ----------------------------------------- | ----------------------------------------------------------------- |
| [Token Interaction](token-interaction.md) | Reading and writing SRC20 balances, signed reads, shielded writes |
| [Transfers](transfers.md)                 | Shielded transfer patterns, approvals, and multi-step workflows   |
| [Event Decryption](event-decryption.md)   | Decrypting encrypted Transfer and Approval events                 |

## Key Concepts

### Signed Reads for Balances

Unlike ERC20 where `balanceOf()` is a simple public read, SRC20's `balanceOf()` uses `msg.sender` to authenticate the caller. This means you must use `seismic_call()` (a signed read) rather than a plain `eth_call`. A plain `eth_call` zeros out the `from` field, so the contract sees the zero address as the sender and returns its balance -- which is almost certainly zero.

### Shielded Writes

Transfers and approvals use `.seismic()` to mark the transaction for calldata encryption. The `SeismicSignedProvider` filler pipeline automatically handles the encryption before the transaction reaches the node.

### Encrypted Events

SRC20 Transfer and Approval events contain encrypted `suint256` values. To decode the actual transfer amounts, you need a viewing key registered with the Directory contract.

## See Also

- [Contract Interaction](../contract-interaction/) -- General shielded and transparent call patterns
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) -- Required for shielded operations
- [Encryption](../provider/encryption.md) -- How calldata encryption works
