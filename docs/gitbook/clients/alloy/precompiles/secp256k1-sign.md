---
description: On-chain secp256k1 ECDSA signing
icon: signature
---

# secp256k1 Sign

Sign messages on-chain using Mercury EVM's secp256k1 signing precompile.

## Overview

The secp256k1 signing precompile at address `0x69` creates ECDSA signatures on-chain. The message is hashed with the EIP-191 personal-sign prefix before signing, producing a 65-byte signature (r + s + v) compatible with `ecrecover` and standard Ethereum signature verification.

## Precompile Address

```
0x0000000000000000000000000000000000000069
```

## Input Encoding

| Field          | Size     | Description                                     |
| -------------- | -------- | ----------------------------------------------- |
| `secret_key`   | 32 bytes | secp256k1 private key                           |
| `message_hash` | 32 bytes | Keccak-256 hash of the EIP-191 prefixed message |

The input is the concatenation of `secret_key` (32 bytes) + `message_hash` (32 bytes), for a total of 64 bytes.

{% hint style="info" %}
You must hash the message yourself before calling the precompile. The precompile expects the EIP-191 prefixed message hash:
`keccak256("\x19Ethereum Signed Message:\n" + len(message) + message)`
{% endhint %}

## Output Format

| Field | Size     | Description            |
| ----- | -------- | ---------------------- |
| `r`   | 32 bytes | Signature r component  |
| `s`   | 32 bytes | Signature s component  |
| `v`   | 1 byte   | Recovery ID (27 or 28) |

Total output: 65 bytes.

## Parameters

| Parameter      | Type       | Required | Description                                        |
| -------------- | ---------- | -------- | -------------------------------------------------- |
| `secret_key`   | `[u8; 32]` | Yes      | 32-byte secp256k1 private key                      |
| `message_hash` | `[u8; 32]` | Yes      | 32-byte keccak256 hash of EIP-191 prefixed message |

## Examples

### Basic Usage

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes, keccak256};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let sign_address: Address =
        "0x0000000000000000000000000000000000000069".parse()?;

    // Private key (32 bytes)
    let secret_key: [u8; 32] = [/* your private key */; 32];

    // Hash the message with EIP-191 prefix
    let message = "Hello, Seismic!";
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let message_hash = keccak256(
        [prefix.as_bytes(), message.as_bytes()].concat(),
    );

    // Build precompile input: sk (32) + message_hash (32)
    let mut input = Vec::with_capacity(64);
    input.extend_from_slice(&secret_key);
    input.extend_from_slice(message_hash.as_slice());

    let result = provider
        .call(
            &TransactionRequest::default()
                .to(sign_address)
                .input(Bytes::from(input).into()),
        )
        .await?;

    println!("Signature ({} bytes): 0x{}", result.len(), hex::encode(&result));

    Ok(())
}
```

### Extract Signature Components

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes, keccak256};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let sign_address: Address =
        "0x0000000000000000000000000000000000000069".parse()?;

    let secret_key: [u8; 32] = [/* ... */; 32];
    let message = "Extract components";
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let message_hash = keccak256(
        [prefix.as_bytes(), message.as_bytes()].concat(),
    );

    let mut input = Vec::with_capacity(64);
    input.extend_from_slice(&secret_key);
    input.extend_from_slice(message_hash.as_slice());

    let signature = provider
        .call(
            &TransactionRequest::default()
                .to(sign_address)
                .input(Bytes::from(input).into()),
        )
        .await?;

    // Signature is 65 bytes: r (32) + s (32) + v (1)
    let r = &signature[..32];
    let s = &signature[32..64];
    let v = signature[64];

    println!("r: 0x{}", hex::encode(r));
    println!("s: 0x{}", hex::encode(s));
    println!("v: {v}");

    Ok(())
}
```

### Sign Multiple Messages

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes, keccak256};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let sign_address: Address =
        "0x0000000000000000000000000000000000000069".parse()?;

    let secret_key: [u8; 32] = [/* ... */; 32];
    let messages = ["Message 1", "Message 2", "Message 3"];

    for msg in &messages {
        let prefix = format!("\x19Ethereum Signed Message:\n{}", msg.len());
        let message_hash = keccak256(
            [prefix.as_bytes(), msg.as_bytes()].concat(),
        );

        let mut input = Vec::with_capacity(64);
        input.extend_from_slice(&secret_key);
        input.extend_from_slice(message_hash.as_slice());

        let signature = provider
            .call(
                &TransactionRequest::default()
                    .to(sign_address)
                    .input(Bytes::from(input).into()),
            )
            .await?;

        println!(
            "Signature for '{}': 0x{}...",
            msg,
            hex::encode(&signature[..20]),
        );
    }

    Ok(())
}
```

### Sign Structured Data

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes, keccak256};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let sign_address: Address =
        "0x0000000000000000000000000000000000000069".parse()?;

    let secret_key: [u8; 32] = [/* ... */; 32];

    // Sign structured data as JSON
    let context = json!({
        "action": "transfer",
        "amount": 1000,
        "recipient": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
    });
    let message = serde_json::to_string(&context)?;

    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let message_hash = keccak256(
        [prefix.as_bytes(), message.as_bytes()].concat(),
    );

    let mut input = Vec::with_capacity(64);
    input.extend_from_slice(&secret_key);
    input.extend_from_slice(message_hash.as_slice());

    let signature = provider
        .call(
            &TransactionRequest::default()
                .to(sign_address)
                .input(Bytes::from(input).into()),
        )
        .await?;

    println!("Context signature: 0x{}", hex::encode(&signature));

    Ok(())
}
```

## How It Works

1. **Hash message** -- You apply the EIP-191 personal-sign prefix and keccak256 hash:

   ```
   prefix = "\x19Ethereum Signed Message:\n" + len(message)
   message_hash = keccak256(prefix + message)
   ```

2. **Encode parameters** -- Concatenate the 32-byte private key and 32-byte message hash

3. **Call precompile** -- Issues an `eth_call` to address `0x69` with 3000 gas

4. **Sign hash** -- Precompile performs ECDSA signing on the secp256k1 curve

5. **Return signature** -- Returns 65-byte signature (r + s + v)

## Gas Cost

Fixed gas cost: **3000 gas**

The cost is constant regardless of message length (since the message is hashed before signing).

## EIP-191 Message Hashing

The precompile expects the standard EIP-191 personal-sign message hash:

```rust
let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
let message_hash = keccak256([prefix.as_bytes(), message.as_bytes()].concat());
```

This ensures compatibility with standard Ethereum message signing (e.g., MetaMask's `personal_sign`).

## Signature Format

The returned signature is 65 bytes:

| Component | Offset | Size     | Description            |
| --------- | ------ | -------- | ---------------------- |
| `r`       | 0      | 32 bytes | Signature r value      |
| `s`       | 32     | 32 bytes | Signature s value      |
| `v`       | 64     | 1 byte   | Recovery ID (27 or 28) |

This format is compatible with `ecrecover` and standard Ethereum signature verification.

## Notes

- Uses secp256k1 curve (same as Ethereum)
- You must hash the message with EIP-191 prefix before calling the precompile
- Signatures are non-deterministic (random `k` value)
- Compatible with MetaMask and other Ethereum wallets
- Can be verified on-chain using the `ecrecover` precompile

## Warnings

- **Private key security** -- Never expose or log private keys
- **Message format** -- Ensure messages are hashed correctly with EIP-191 prefix before sending to the precompile
- **Signature malleability** -- Standard ECDSA signatures are malleable (use EIP-2098 compact signatures if needed)
- **Non-deterministic** -- Multiple signatures of the same message will differ due to random `k` values

## See Also

- [Precompiles Overview](./) -- All precompile reference
- [ecdh](ecdh.md) -- ECDH key exchange
- [EIP-191](https://eips.ethereum.org/EIPS/eip-191) -- Signed data standard
