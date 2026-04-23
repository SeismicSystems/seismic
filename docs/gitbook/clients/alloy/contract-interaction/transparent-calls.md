---
description: Standard Ethereum calls without encryption
icon: eye
---

# Transparent Calls

Non-encrypted contract interactions using standard Ethereum transaction types. Use these for public data, contract deployment, and operations that do not require privacy.

## Overview

Transparent calls are standard Ethereum operations that do not use Seismic encryption. Calldata and return values are visible on-chain, just like any regular Ethereum transaction. For functions without shielded parameters, simply use `.call()` or `.send()` directly (without `.seismic()`). Note that functions with shielded parameters (e.g., `suint256`) auto-encrypt by default — see [Shielded Calls](shielded-calls.md) for details.

## When to Use Transparent Calls

| Scenario                           | Use Transparent |           Use Shielded            |
| ---------------------------------- | :-------------: | :-------------------------------: |
| Contract deployment                |       Yes       | No (Create txs cannot be seismic) |
| Reading public state               |       Yes       |                No                 |
| Writing public state               |       Yes       |             Optional              |
| Reading private state              |       No        |                Yes                |
| Writing private state              |       No        |                Yes                |
| Functions with shielded parameters |       No        |                Yes                |

## Contract Deployment

Contract deployment always uses transparent transactions because Create transactions cannot be seismic. Use the `#[sol(rpc, bytecode = "...")]` attribute to generate a `deploy()` method:

```rust
use seismic_prelude::client::*;

sol! {
    #[sol(rpc, bytecode = "0x6080604052...")]
    contract SeismicCounter {
        function setNumber(suint256 newNumber) public;
        function isOdd() public view returns (bool);
    }
}

// Deploy with generated deploy() method
let contract = SeismicCounter::deploy(&provider).await?;
println!("Deployed to: {:?}", contract.address());

// Now interact with shielded calls
// isOdd has no shielded params, so use .seismic() for encryption
let is_odd = contract.isOdd().seismic().call().await?;
```

{% hint style="info" %}
After deploying a contract transparently, you can immediately interact with it using shielded calls. See [Shielded Calls](shielded-calls.md) for details.
{% endhint %}

## Transparent Write

A transparent write sends a standard `eth_sendTransaction` with unencrypted calldata. Use this for functions that do not handle private data.

```rust
sol! {
    #[sol(rpc)]
    contract MyContract {
        function setPublicValue(uint256 value) public;
    }
}

let contract = MyContract::new(contract_address, &provider);

// No .seismic() -- sent as standard Ethereum transaction
let receipt = contract
    .setPublicValue(U256::from(100))
    .send()
    .await?
    .get_receipt()
    .await?;
```

## Transparent Read

A transparent read executes a standard `eth_call` with unencrypted calldata. Both signed and unsigned providers can perform transparent reads.

```rust
sol! {
    #[sol(rpc)]
    contract MyContract {
        function getPublicValue() public view returns (uint256);
    }
}

let contract = MyContract::new(contract_address, &provider);

// Standard eth_call -- no encryption
let value = contract.getPublicValue().call().await?;
```

### Using an Unsigned Provider

Transparent reads do not require a private key, so you can use an unsigned provider:

```rust
// No private key needed -- connect_http is synchronous for unsigned providers
let url = "https://testnet-1.seismictest.net/rpc".parse()?;
let provider = SeismicProviderBuilder::new().connect_http(url);

let contract = MyContract::new(contract_address, &provider);
let value = contract.getPublicValue().call().await?;
```

## Complete Example: Deploy and Interact

This example demonstrates the typical workflow: deploy a contract transparently, then use both shielded and transparent calls.

```rust
use seismic_prelude::client::*;
use seismic_alloy_network::reth::SeismicReth;

sol! {
    #[sol(rpc, bytecode = "0x6080604052...")]
    contract SeismicCounter {
        function setNumber(suint256 newNumber) public;
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

    // 1. Deploy contract (transparent -- Create txs cannot be seismic)
    let contract = SeismicCounter::deploy(&provider).await?;
    println!("Deployed to: {:?}", contract.address());

    // 2. Shielded write -- setNumber has a shielded param (suint256), auto-encrypts
    contract.setNumber(alloy_primitives::aliases::SUInt(U256::from(42)))
        .send()
        .await?
        .get_receipt()
        .await?;

    // 3. Transparent read (public data, no encryption needed)
    let is_odd = contract.isOdd().call().await?;
    println!("Is odd: {is_odd}");

    Ok(())
}
```

## Transparent vs. Shielded

There are two ways a call becomes shielded:

1. **Auto-encryption**: Functions with shielded parameters (e.g., `suint256`) automatically return a `ShieldedCallBuilder`. Calling `.send()` or `.call()` encrypts automatically.
2. **Manual opt-in**: For functions without shielded parameters, call `.seismic()` to convert a `SolCallBuilder` into a `ShieldedCallBuilder`.

```rust
// Transparent: no shielded params, no .seismic()
let is_odd = contract.isOdd().call().await?;

// Shielded via .seismic(): no shielded params, but need encryption
let is_odd = contract.isOdd().seismic().call().await?;

// Shielded via auto-encryption: setNumber has suint256 param
contract.setNumber(alloy_primitives::aliases::SUInt(U256::from(42))).send().await?;
```

When neither auto-encryption nor `.seismic()` is used:

- No `TxSeismicElements` are attached
- No calldata encryption occurs
- The transaction is sent as a standard Ethereum type
- Any provider type can execute the operation

## See Also

- [Shielded Calls](shielded-calls.md) — Encrypted contract interactions
- [Contract Interaction Overview](./) — Comparison of all interaction patterns
- [SeismicUnsignedProvider](../provider/seismic-unsigned-provider.md) — Read-only provider for transparent operations
- [SeismicSignedProvider](../provider/seismic-signed-provider.md) — Full-featured provider
