---
description: On-chain AES-256-GCM decryption
icon: unlock
---

# AES-GCM Decrypt

Decrypt data on-chain using Mercury EVM's AES-GCM decryption precompile.

## Overview

The AES-GCM decryption precompile at address `0x67` performs AES-256-GCM authenticated decryption. The ciphertext must include the 16-byte authentication tag appended during encryption. If the tag does not verify, the precompile reverts.

## Precompile Address

```
0x0000000000000000000000000000000000000067
```

## Input Encoding

| Field        | Size     | Description                                             |
| ------------ | -------- | ------------------------------------------------------- |
| `key`        | 32 bytes | AES-256 decryption key (must match encryption key)      |
| `nonce`      | 12 bytes | Nonce (must match the nonce used during encryption)     |
| `ciphertext` | Variable | Encrypted data including the 16-byte authentication tag |

The input is the concatenation of `key` (32 bytes) + `nonce` (12 bytes) + `ciphertext` (variable length, includes 16-byte tag).

## Output Format

| Field       | Size                         | Description    |
| ----------- | ---------------------------- | -------------- |
| `plaintext` | `len(ciphertext) - 16` bytes | Decrypted data |

## Parameters

| Parameter    | Type       | Required | Description                                     |
| ------------ | ---------- | -------- | ----------------------------------------------- |
| `key`        | `[u8; 32]` | Yes      | 32-byte AES-256 decryption key                  |
| `nonce`      | `[u8; 12]` | Yes      | 12-byte nonce (must match encryption nonce)     |
| `ciphertext` | `&[u8]`    | Yes      | Ciphertext including 16-byte authentication tag |

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
    let decrypt_address: Address =
        "0x0000000000000000000000000000000000000067".parse()?;

    let key = [0x42u8; 32];
    let nonce = [0u8; 12];
    let plaintext = b"Secret message";

    // Encrypt first
    let mut encrypt_input = Vec::new();
    encrypt_input.extend_from_slice(&key);
    encrypt_input.extend_from_slice(&nonce);
    encrypt_input.extend_from_slice(plaintext);

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

    assert_eq!(&decrypted[..], plaintext);
    println!("Decrypted: {}", String::from_utf8_lossy(&decrypted));

    Ok(())
}
```

### Decrypt Multiple Messages

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
    let messages = ["Message 1", "Message 2", "Message 3"];
    let mut ciphertexts = Vec::new();

    // Encrypt each message with a unique nonce
    for (i, msg) in messages.iter().enumerate() {
        let mut nonce = [0u8; 12];
        nonce[11] = i as u8;

        let mut input = Vec::new();
        input.extend_from_slice(&key);
        input.extend_from_slice(&nonce);
        input.extend_from_slice(msg.as_bytes());

        let ct = provider
            .call(
                &TransactionRequest::default()
                    .to(encrypt_address)
                    .input(Bytes::from(input).into()),
            )
            .await?;

        ciphertexts.push((nonce, ct));
    }

    // Decrypt all messages
    for (i, (nonce, ct)) in ciphertexts.iter().enumerate() {
        let mut input = Vec::new();
        input.extend_from_slice(&key);
        input.extend_from_slice(nonce);
        input.extend_from_slice(ct);

        let decrypted = provider
            .call(
                &TransactionRequest::default()
                    .to(decrypt_address)
                    .input(Bytes::from(input).into()),
            )
            .await?;

        println!("Message {i}: {}", String::from_utf8_lossy(&decrypted));
    }

    Ok(())
}
```

### Handle Decryption Failure

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let decrypt_address: Address =
        "0x0000000000000000000000000000000000000067".parse()?;

    let key = [0x42u8; 32];
    let nonce = [0u8; 12];
    let bad_ciphertext = b"this is not valid ciphertext";

    let mut input = Vec::new();
    input.extend_from_slice(&key);
    input.extend_from_slice(&nonce);
    input.extend_from_slice(bad_ciphertext);

    // Decryption will fail if the authentication tag doesn't verify
    match provider
        .call(
            &TransactionRequest::default()
                .to(decrypt_address)
                .input(Bytes::from(input).into()),
        )
        .await
    {
        Ok(plaintext) => println!("Decrypted: 0x{}", hex::encode(&plaintext)),
        Err(e) => println!("Decryption failed (expected): {e}"),
    }

    Ok(())
}
```

### With ECDH-Derived Key (Bob's Side)

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    // Bob derives the same shared secret that Alice used to encrypt
    let ecdh_address: Address =
        "0x0000000000000000000000000000000000000065".parse()?;

    let bob_sk: [u8; 32] = [/* Bob's private key */; 32];
    let alice_pk: [u8; 33] = [/* Alice's compressed public key */; 33];

    let mut ecdh_input = Vec::with_capacity(65);
    ecdh_input.extend_from_slice(&bob_sk);
    ecdh_input.extend_from_slice(&alice_pk);

    let ecdh_result = provider
        .call(
            &TransactionRequest::default()
                .to(ecdh_address)
                .input(Bytes::from(ecdh_input).into()),
        )
        .await?;

    let aes_key: [u8; 32] = ecdh_result[..32].try_into()?;

    // Decrypt the ciphertext Alice sent
    let decrypt_address: Address =
        "0x0000000000000000000000000000000000000067".parse()?;

    let nonce = [0u8; 12]; // Must match Alice's encryption nonce
    let ciphertext: &[u8] = &[/* ciphertext from Alice */];

    let mut decrypt_input = Vec::new();
    decrypt_input.extend_from_slice(&aes_key);
    decrypt_input.extend_from_slice(&nonce);
    decrypt_input.extend_from_slice(ciphertext);

    let decrypted = provider
        .call(
            &TransactionRequest::default()
                .to(decrypt_address)
                .input(Bytes::from(decrypt_input).into()),
        )
        .await?;

    println!("Bob received: {}", String::from_utf8_lossy(&decrypted));

    Ok(())
}
```

## How It Works

1. **Encode parameters** -- Concatenates 32-byte key + 12-byte nonce + ciphertext (with tag)
2. **Call precompile** -- Issues an `eth_call` to address `0x67` with estimated gas
3. **Decrypt and verify** -- Precompile performs AES-256-GCM decryption and verifies the authentication tag
4. **Return plaintext** -- Returns decrypted data if tag verification succeeds; reverts otherwise

## Gas Cost

Gas cost is calculated as:

```
base_gas = 1000
per_block = 30  // per 16-byte block
num_blocks = (len(ciphertext) + 15) / 16
total_gas = base_gas + (num_blocks * per_block)
```

The gas cost is proportional to ciphertext length (including the 16-byte tag).

## Notes

- Uses AES-256-GCM authenticated decryption
- Nonce must exactly match the nonce used during encryption
- Ciphertext must include the 16-byte authentication tag (appended by encryption)
- Decryption fails (reverts) if the authentication tag does not verify
- Plaintext length = ciphertext length - 16 bytes (authentication tag)

## Warnings

- **Authentication failure** -- If the tag does not verify, the precompile reverts. This can happen with a wrong key, wrong nonce, or tampered ciphertext.
- **Nonce mismatch** -- Using a different nonce than the one used for encryption will cause decryption to fail
- **Key mismatch** -- Using a different key than the one used for encryption will cause authentication failure
- **Ciphertext integrity** -- Any modification to the ciphertext (including the tag) causes authentication failure

## See Also

- [Precompiles Overview](./) -- All precompile reference
- [aes-gcm-encrypt](aes-gcm-encrypt.md) -- Encrypt with AES-GCM
- [ecdh](ecdh.md) -- Derive shared decryption keys
- [Encryption](../provider/encryption.md) -- How the provider uses AES-GCM internally
