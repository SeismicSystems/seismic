---
description: On-chain HKDF-SHA256 key derivation
icon: wand-magic-sparkles
---

# HKDF

Derive cryptographic keys on-chain using Mercury EVM's HKDF precompile.

## Overview

The HKDF precompile at address `0x68` performs HKDF-SHA256 key derivation on input key material (IKM). It produces a uniformly distributed 32-byte derived key suitable for use as an AES-256 key or other cryptographic purpose.

{% hint style="info" %}
The `SeismicSignedProvider` uses this precompile internally to derive AES keys from ECDH shared secrets during the TEE key exchange. You can also call it directly for custom key derivation workflows.
{% endhint %}

## Precompile Address

```
0x0000000000000000000000000000000000000068
```

## Input Encoding

| Field | Size     | Description                          |
| ----- | -------- | ------------------------------------ |
| `ikm` | Variable | Input key material (arbitrary bytes) |

The input is the raw IKM bytes -- no additional encoding is needed.

## Output Format

| Field         | Size     | Description             |
| ------------- | -------- | ----------------------- |
| `derived_key` | 32 bytes | HKDF-SHA256 derived key |

## Parameters

| Parameter | Type    | Required | Description                          |
| --------- | ------- | -------- | ------------------------------------ |
| `ikm`     | `&[u8]` | Yes      | Input key material (arbitrary bytes) |

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

    let hkdf_address: Address =
        "0x0000000000000000000000000000000000000068".parse()?;

    // Derive key from input material
    let ikm = b"my-input-key-material";
    let input = Bytes::from(ikm.to_vec());

    let result = provider
        .call(
            &TransactionRequest::default()
                .to(hkdf_address)
                .input(input.into()),
        )
        .await?;

    let derived_key: [u8; 32] = result[..32].try_into()?;
    println!("Derived key: 0x{}", hex::encode(derived_key));

    Ok(())
}
```

### Derive Multiple Keys by Context

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let hkdf_address: Address =
        "0x0000000000000000000000000000000000000068".parse()?;

    let master_secret = b"shared-master-key";
    let contexts = [b"encryption" as &[u8], b"authentication", b"signing"];

    for ctx in &contexts {
        let mut ikm = master_secret.to_vec();
        ikm.extend_from_slice(ctx);

        let result = provider
            .call(
                &TransactionRequest::default()
                    .to(hkdf_address)
                    .input(Bytes::from(ikm).into()),
            )
            .await?;

        let key: [u8; 32] = result[..32].try_into()?;
        println!(
            "Key for {}: 0x{}",
            String::from_utf8_lossy(ctx),
            hex::encode(key),
        );
    }

    Ok(())
}
```

### Use as AES Key

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    // Step 1: Derive AES key via HKDF
    let hkdf_address: Address =
        "0x0000000000000000000000000000000000000068".parse()?;

    let ikm = b"my-master-secret";
    let hkdf_result = provider
        .call(
            &TransactionRequest::default()
                .to(hkdf_address)
                .input(Bytes::from(ikm.to_vec()).into()),
        )
        .await?;

    let aes_key: [u8; 32] = hkdf_result[..32].try_into()?;

    // Step 2: Use derived key for encryption
    let encrypt_address: Address =
        "0x0000000000000000000000000000000000000066".parse()?;

    let nonce = [0u8; 12];
    let plaintext = b"Encrypted with derived key";

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

### Derive from ECDH Output

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    // Step 1: Perform ECDH
    let ecdh_address: Address =
        "0x0000000000000000000000000000000000000065".parse()?;

    let my_sk: [u8; 32] = [/* ... */; 32];
    let peer_pk: [u8; 33] = [/* ... */; 33];

    let mut ecdh_input = Vec::with_capacity(65);
    ecdh_input.extend_from_slice(&my_sk);
    ecdh_input.extend_from_slice(&peer_pk);

    let shared_secret = provider
        .call(
            &TransactionRequest::default()
                .to(ecdh_address)
                .input(Bytes::from(ecdh_input).into()),
        )
        .await?;

    // Step 2: Further derive key from shared secret + application context
    let hkdf_address: Address =
        "0x0000000000000000000000000000000000000068".parse()?;

    let mut ikm = shared_secret.to_vec();
    ikm.extend_from_slice(b"application-context-v1");

    let result = provider
        .call(
            &TransactionRequest::default()
                .to(hkdf_address)
                .input(Bytes::from(ikm).into()),
        )
        .await?;

    let derived_key: [u8; 32] = result[..32].try_into()?;
    println!("Derived key: 0x{}", hex::encode(derived_key));

    Ok(())
}
```

### Deterministic Key Derivation

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let hkdf_address: Address =
        "0x0000000000000000000000000000000000000068".parse()?;

    // Same input always produces the same output
    let ikm = b"deterministic-input";
    let input = Bytes::from(ikm.to_vec());

    let key1 = provider
        .call(
            &TransactionRequest::default()
                .to(hkdf_address)
                .input(input.clone().into()),
        )
        .await?;

    let key2 = provider
        .call(
            &TransactionRequest::default()
                .to(hkdf_address)
                .input(input.into()),
        )
        .await?;

    assert_eq!(&key1[..32], &key2[..32]);
    println!("Deterministic key: 0x{}", hex::encode(&key1[..32]));

    Ok(())
}
```

## How It Works

1. **Encode parameters** -- Passes input key material as-is
2. **Call precompile** -- Issues an `eth_call` to address `0x68` with gas proportional to input length
3. **HKDF derivation** -- Precompile performs HKDF-SHA256 extract and expand phases
4. **Return key** -- Returns first 32 bytes of derived key material

## Gas Cost

Gas cost is calculated as:

```
sha256_base = 60
sha256_per_word = 12  // per 32-byte word
hkdf_expand = 120

// Extract phase (twice)
extract_cost = 2 * (sha256_base + (len(ikm) / 32) * sha256_per_word + 3000)

// Total
total_gas = extract_cost + hkdf_expand
```

For example:

| IKM Size  | Gas Cost |
| --------- | -------- |
| 32 bytes  | ~6144    |
| 64 bytes  | ~6168    |
| 128 bytes | ~6216    |

## Notes

- Uses HKDF-SHA256 from RFC 5869
- Always returns exactly 32 bytes regardless of input length
- Input key material can be any length
- The derivation is deterministic: same IKM always produces the same output
- Internally performs both HKDF-Extract and HKDF-Expand phases
- The derived key has uniform distribution suitable for cryptographic use

## Use Cases

- Derive encryption keys from shared secrets
- Convert non-uniform entropy into uniform keys
- Key separation: derive multiple keys from one master secret
- Post-process ECDH output for additional security

## Warnings

- **Not for password hashing** -- Use proper password hashing algorithms (bcrypt, argon2) for passwords
- **Input entropy** -- Output security depends entirely on input entropy
- **Deterministic** -- Same input always yields the same output (no salt or randomness is added)

## See Also

- [Precompiles Overview](./) -- All precompile reference
- [ecdh](ecdh.md) -- Often used before HKDF to derive keys
- [aes-gcm-encrypt](aes-gcm-encrypt.md) -- Use derived keys for encryption
- [rng](rng.md) -- Generate random input material
