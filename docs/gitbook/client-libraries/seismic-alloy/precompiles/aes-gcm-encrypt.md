---
description: On-chain AES-256-GCM encryption
icon: lock
---

# AES-GCM Encrypt

Encrypt data on-chain using Mercury EVM's AES-GCM encryption precompile.

## Overview

The AES-GCM encryption precompile at address `0x66` performs AES-256-GCM authenticated encryption. The ciphertext includes a 16-byte authentication tag that ensures integrity and authenticity.

{% hint style="info" %}
The `SeismicSignedProvider` filler pipeline uses this precompile internally to encrypt calldata before sending shielded transactions. You can also call it directly for custom encryption workflows.
{% endhint %}

## Precompile Address

```
0x0000000000000000000000000000000000000066
```

## Input Encoding

| Field       | Size     | Description                                           |
| ----------- | -------- | ----------------------------------------------------- |
| `key`       | 32 bytes | AES-256 encryption key                                |
| `nonce`     | 12 bytes | Unique nonce (must never be reused with the same key) |
| `plaintext` | Variable | Data to encrypt                                       |

The input is the concatenation of `key` (32 bytes) + `nonce` (12 bytes) + `plaintext` (variable length).

## Output Format

| Field        | Size                   | Description                                 |
| ------------ | ---------------------- | ------------------------------------------- |
| `ciphertext` | `len(plaintext)` bytes | Encrypted data                              |
| `tag`        | 16 bytes               | Authentication tag (appended to ciphertext) |

Total output length = `len(plaintext) + 16`.

## Parameters

| Parameter   | Type       | Required | Description                            |
| ----------- | ---------- | -------- | -------------------------------------- |
| `key`       | `[u8; 32]` | Yes      | 32-byte AES-256 encryption key         |
| `nonce`     | `[u8; 12]` | Yes      | 12-byte nonce (must be unique per key) |
| `plaintext` | `&[u8]`    | Yes      | Data to encrypt                        |

## Examples

### Basic Usage

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let encrypt_address: Address =
        "0x0000000000000000000000000000000000000066".parse()?;

    // 32-byte AES key
    let key = [0x42u8; 32]; // Use a proper key in production
    // 12-byte nonce
    let nonce = [0u8; 12];
    // Plaintext to encrypt
    let plaintext = b"Secret message";

    let mut input = Vec::new();
    input.extend_from_slice(&key);
    input.extend_from_slice(&nonce);
    input.extend_from_slice(plaintext);

    let result = provider
        .call(
            &TransactionRequest::default()
                .to(encrypt_address)
                .input(Bytes::from(input).into()),
        )
        .await?;

    println!("Ciphertext ({} bytes): 0x{}", result.len(), hex::encode(&result));
    println!("  Plaintext was {} bytes + 16-byte tag", plaintext.len());

    Ok(())
}
```

### Encrypt with Integer Nonce

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let encrypt_address: Address =
        "0x0000000000000000000000000000000000000066".parse()?;

    let key = [0x42u8; 32];

    // Use a counter as nonce (converted to 12 bytes, big-endian)
    for i in 0u64..5 {
        let mut nonce = [0u8; 12];
        nonce[4..12].copy_from_slice(&i.to_be_bytes());

        let plaintext = format!("Message {i}");

        let mut input = Vec::new();
        input.extend_from_slice(&key);
        input.extend_from_slice(&nonce);
        input.extend_from_slice(plaintext.as_bytes());

        let result = provider
            .call(
                &TransactionRequest::default()
                    .to(encrypt_address)
                    .input(Bytes::from(input).into()),
            )
            .await?;

        println!("Ciphertext {i}: 0x{}", hex::encode(&result));
    }

    Ok(())
}
```

### Encrypt-Decrypt Round Trip

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let encrypt_address: Address =
        "0x0000000000000000000000000000000000000066".parse()?;
    let decrypt_address: Address =
        "0x0000000000000000000000000000000000000067".parse()?;

    let key = [0x42u8; 32];
    let nonce = [0u8; 12];
    let original = b"Round trip test";

    // Encrypt
    let mut encrypt_input = Vec::new();
    encrypt_input.extend_from_slice(&key);
    encrypt_input.extend_from_slice(&nonce);
    encrypt_input.extend_from_slice(original);

    let ciphertext = provider
        .call(
            &TransactionRequest::default()
                .to(encrypt_address)
                .input(Bytes::from(encrypt_input).into()),
        )
        .await?;

    // Decrypt
    let mut decrypt_input = Vec::new();
    decrypt_input.extend_from_slice(&key);
    decrypt_input.extend_from_slice(&nonce);
    decrypt_input.extend_from_slice(&ciphertext);

    let decrypted = provider
        .call(
            &TransactionRequest::default()
                .to(decrypt_address)
                .input(Bytes::from(decrypt_input).into()),
        )
        .await?;

    assert_eq!(&decrypted[..], original);
    println!("Round trip successful: {}", String::from_utf8_lossy(&decrypted));

    Ok(())
}
```

### With ECDH-Derived Key

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    // Step 1: Derive shared key via ECDH
    let ecdh_address: Address =
        "0x0000000000000000000000000000000000000065".parse()?;

    let my_sk: [u8; 32] = [/* ... */; 32];
    let peer_pk: [u8; 33] = [/* ... */; 33];

    let mut ecdh_input = Vec::with_capacity(65);
    ecdh_input.extend_from_slice(&my_sk);
    ecdh_input.extend_from_slice(&peer_pk);

    let ecdh_result = provider
        .call(
            &TransactionRequest::default()
                .to(ecdh_address)
                .input(Bytes::from(ecdh_input).into()),
        )
        .await?;

    let aes_key: [u8; 32] = ecdh_result[..32].try_into()?;

    // Step 2: Encrypt with derived key
    let encrypt_address: Address =
        "0x0000000000000000000000000000000000000066".parse()?;

    let nonce = [0u8; 12];
    let plaintext = b"Encrypted for peer";

    let mut encrypt_input = Vec::new();
    encrypt_input.extend_from_slice(&aes_key);
    encrypt_input.extend_from_slice(&nonce);
    encrypt_input.extend_from_slice(plaintext);

    let ciphertext = provider
        .call(
            &TransactionRequest::default()
                .to(encrypt_address)
                .input(Bytes::from(encrypt_input).into()),
        )
        .await?;

    println!("Ciphertext: 0x{}", hex::encode(&ciphertext));

    Ok(())
}
```

## How It Works

1. **Encode parameters** -- Concatenates 32-byte key + 12-byte nonce + plaintext
2. **Call precompile** -- Issues an `eth_call` to address `0x66` with estimated gas
3. **Encrypt data** -- Precompile performs AES-256-GCM encryption
4. **Return ciphertext** -- Returns encrypted data with 16-byte authentication tag appended

## Gas Cost

Gas cost is calculated as:

```
base_gas = 1000
per_block = 30  // per 16-byte block
num_blocks = (len(plaintext) + 15) / 16
total_gas = base_gas + (num_blocks * per_block)
```

For example:

| Plaintext Size | Gas Cost |
| -------------- | -------- |
| 16 bytes       | 1030     |
| 64 bytes       | 1120     |
| 256 bytes      | 1480     |

## Notes

- Uses AES-256-GCM authenticated encryption
- Nonce must be unique for each encryption with the same key
- Ciphertext length = plaintext length + 16 bytes (authentication tag)
- The authentication tag ensures ciphertext integrity and authenticity
- Reusing a nonce with the same key breaks security

## Warnings

- **Nonce reuse** -- NEVER reuse the same nonce with the same key. This breaks confidentiality and can leak the plaintext.
- **Key security** -- Keep AES keys secure and never expose them in logs or error messages
- **Authentication tag** -- The 16-byte tag is appended to the ciphertext and must be included when decrypting
- **Counter management** -- When using integer nonces, ensure they are sequential and never repeated

## See Also

- [Precompiles Overview](./) -- All precompile reference
- [aes-gcm-decrypt](aes-gcm-decrypt.md) -- Decrypt AES-GCM ciphertext
- [ecdh](ecdh.md) -- Derive shared encryption keys
- [Encryption](../provider/encryption.md) -- How the provider uses AES-GCM internally
