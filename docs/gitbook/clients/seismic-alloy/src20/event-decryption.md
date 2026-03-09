---
description: Decrypting encrypted SRC20 Transfer and Approval events
icon: eye
---

# Event Decryption

Decode encrypted Transfer and Approval events emitted by SRC20 contracts.

## Overview

SRC20 events contain encrypted `suint256` values. Unlike standard ERC20, where Transfer and Approval event data is publicly visible on-chain, SRC20 encrypts the amounts so that only authorized parties can read them. To decrypt event data, you need a **viewing key** registered with the Seismic Directory contract.

## How SRC20 Events Work

```
Standard ERC20 Transfer event:
  Transfer(address indexed from, address indexed to, uint256 value)
  -> value is publicly visible in event logs

SRC20 Transfer event:
  Transfer(address indexed from, address indexed to, suint256 value)
  -> value is encrypted in event logs
  -> Only holders of the viewing key can decrypt
```

### What Is Encrypted

| Event Field                   | Encrypted? | Description                                |
| ----------------------------- | ---------- | ------------------------------------------ |
| `from` (indexed)              | No         | Sender address is visible as a topic       |
| `to` (indexed)                | No         | Recipient address is visible as a topic    |
| `value`                       | Yes        | Transfer amount is encrypted in event data |
| `owner` (indexed, Approval)   | No         | Owner address is visible                   |
| `spender` (indexed, Approval) | No         | Spender address is visible                 |

{% hint style="info" %}
Indexed event parameters (addresses) remain visible as log topics. Only the non-indexed `suint256` values in the event data are encrypted.
{% endhint %}

## Subscribing to SRC20 Events

Use an unsigned WebSocket provider to subscribe to events in real time:

```rust
use seismic_alloy::prelude::*;
use alloy::providers::Provider;
use alloy::sol;
use alloy_primitives::Address;
use alloy_rpc_types_eth::Filter;
use futures_util::StreamExt;

sol! {
    interface ISRC20 {
        event Transfer(address indexed from, address indexed to, suint256 value);
        event Approval(address indexed owner, address indexed spender, suint256 value);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ws_url = "wss://node.seismicdev.net/ws".parse()?;
    let provider = SeismicUnsignedProvider::<SeismicReth>::new_ws(ws_url).await?;

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;

    // Build a filter for Transfer events
    let transfer_filter = Filter::new()
        .address(token)
        .event_signature(ISRC20::Transfer::SIGNATURE_HASH);

    // Subscribe to new logs
    let subscription = provider.subscribe_logs(&transfer_filter).await?;
    let mut stream = subscription.into_stream();

    println!("Listening for Transfer events...");

    while let Some(log) = stream.next().await {
        // Indexed parameters are in log.topics
        let from = Address::from_slice(&log.topics()[1].as_slice()[12..]);
        let to = Address::from_slice(&log.topics()[2].as_slice()[12..]);

        // The data field contains the encrypted suint256 value
        let encrypted_value = &log.data().data;

        println!("Transfer event:");
        println!("  From: {from}");
        println!("  To:   {to}");
        println!("  Encrypted value: 0x{}", hex::encode(encrypted_value));
        println!("  (Decryption requires viewing key)");
    }

    Ok(())
}
```

## Fetching Historical Events

Query past Transfer events using `get_logs`:

```rust
use seismic_alloy::prelude::*;
use alloy::providers::Provider;
use alloy::sol;
use alloy_primitives::Address;
use alloy_rpc_types_eth::Filter;

sol! {
    interface ISRC20 {
        event Transfer(address indexed from, address indexed to, suint256 value);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;

    // Query Transfer events from the last 1000 blocks
    let latest_block = provider.get_block_number().await?;
    let from_block = latest_block.saturating_sub(1000);

    let filter = Filter::new()
        .address(token)
        .event_signature(ISRC20::Transfer::SIGNATURE_HASH)
        .from_block(from_block)
        .to_block(latest_block);

    let logs = provider.get_logs(&filter).await?;

    println!("Found {} Transfer events", logs.len());

    for log in &logs {
        let from = Address::from_slice(&log.topics()[1].as_slice()[12..]);
        let to = Address::from_slice(&log.topics()[2].as_slice()[12..]);

        println!(
            "  Block {}: {from} -> {to} (value encrypted)",
            log.block_number.unwrap_or(0),
        );
    }

    Ok(())
}
```

## Filtering by Sender or Recipient

Use indexed topics to filter events for a specific address:

```rust
use seismic_alloy::prelude::*;
use alloy::providers::Provider;
use alloy::sol;
use alloy_primitives::{Address, B256};
use alloy_rpc_types_eth::Filter;

sol! {
    interface ISRC20 {
        event Transfer(address indexed from, address indexed to, suint256 value);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;
    let my_address: Address = "0xYOUR_ADDRESS".parse()?;

    // Pad address to 32 bytes for topic filter
    let mut address_topic = [0u8; 32];
    address_topic[12..].copy_from_slice(my_address.as_slice());
    let address_topic = B256::from(address_topic);

    // Filter for transfers FROM my address
    let sent_filter = Filter::new()
        .address(token)
        .event_signature(ISRC20::Transfer::SIGNATURE_HASH)
        .topic1(address_topic);

    let sent_logs = provider.get_logs(&sent_filter).await?;
    println!("Sent {} transfers", sent_logs.len());

    // Filter for transfers TO my address
    let received_filter = Filter::new()
        .address(token)
        .event_signature(ISRC20::Transfer::SIGNATURE_HASH)
        .topic2(address_topic);

    let received_logs = provider.get_logs(&received_filter).await?;
    println!("Received {} transfers", received_logs.len());

    Ok(())
}
```

## Viewing Keys and Decryption

SRC20 event values are encrypted with the contract's encryption scheme. To decrypt them:

1. **Register a viewing key** with the Seismic Directory contract
2. **Derive the decryption key** from the viewing key and the event's encryption context
3. **Decrypt the event data** using AES-GCM

{% hint style="warning" %}
Viewing key registration and event decryption are protocol-level features. The exact API for registering viewing keys and decrypting events may vary depending on the SRC20 implementation and the Seismic protocol version. Consult the latest protocol documentation for the current viewing key registration process.
{% endhint %}

### Conceptual Flow

```
1. Register viewing key with Directory contract
   -> Directory stores your public key for the token

2. When events are emitted:
   -> Node encrypts suint256 values with viewing key
   -> Encrypted data stored in event logs

3. To decrypt:
   -> Fetch the encrypted event log
   -> Use your viewing key private key + event context
   -> Derive AES key via ECDH + HKDF
   -> Decrypt the suint256 value via AES-GCM
```

## Monitoring Multiple Event Types

Subscribe to both Transfer and Approval events simultaneously:

```rust
use seismic_alloy::prelude::*;
use alloy::providers::Provider;
use alloy::sol;
use alloy_primitives::{Address, B256};
use alloy_rpc_types_eth::Filter;
use futures_util::StreamExt;

sol! {
    interface ISRC20 {
        event Transfer(address indexed from, address indexed to, suint256 value);
        event Approval(address indexed owner, address indexed spender, suint256 value);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ws_url = "wss://node.seismicdev.net/ws".parse()?;
    let provider = SeismicUnsignedProvider::<SeismicReth>::new_ws(ws_url).await?;

    let token: Address = "0xYOUR_TOKEN_ADDRESS".parse()?;

    // Subscribe to all events from the token contract
    let filter = Filter::new().address(token);

    let subscription = provider.subscribe_logs(&filter).await?;
    let mut stream = subscription.into_stream();

    println!("Listening for all SRC20 events...");

    while let Some(log) = stream.next().await {
        let event_sig = log.topics()[0];

        if event_sig == ISRC20::Transfer::SIGNATURE_HASH {
            let from = Address::from_slice(&log.topics()[1].as_slice()[12..]);
            let to = Address::from_slice(&log.topics()[2].as_slice()[12..]);
            println!("Transfer: {from} -> {to} (encrypted value)");
        } else if event_sig == ISRC20::Approval::SIGNATURE_HASH {
            let owner = Address::from_slice(&log.topics()[1].as_slice()[12..]);
            let spender = Address::from_slice(&log.topics()[2].as_slice()[12..]);
            println!("Approval: {owner} approved {spender} (encrypted amount)");
        } else {
            println!("Unknown event: {event_sig}");
        }
    }

    Ok(())
}
```

## Notes

- SRC20 events encrypt only the `suint256` value fields, not the indexed address parameters
- Indexed parameters (addresses) are always visible as log topics
- Event subscription requires a WebSocket provider (`new_ws()`)
- Historical event queries work with HTTP providers via `get_logs()`
- Viewing key registration is done through the Seismic Directory contract
- Each SRC20 contract manages its own encryption context for events

## Warnings

- **Privacy limitations** -- While transfer amounts are encrypted, the sender and recipient addresses are still visible as indexed topics. Transaction metadata (gas, block number, etc.) is also public.
- **Viewing key management** -- Protect your viewing key private key. Anyone with access to it can decrypt your token events.
- **Block range limits** -- Some RPC providers limit the block range for `get_logs` queries. Use pagination for large historical ranges.

## See Also

- [Token Interaction](token-interaction.md) -- Reading balances and metadata
- [Transfers](transfers.md) -- Shielded transfer patterns
- [Precompiles Overview](../precompiles/) -- Cryptographic operations used in decryption
- [Provider Overview](../provider/) -- WebSocket and HTTP providers
