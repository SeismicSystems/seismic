---
description: On-chain elliptic-curve Diffie-Hellman key exchange
icon: key
---

# ECDH

Perform on-chain ECDH key exchange using Mercury EVM's ECDH precompile.

## Overview

The ECDH precompile at address `0x65` computes a shared secret from a private key and a public key using elliptic-curve Diffie-Hellman on the secp256k1 curve. The result is a 32-byte shared secret derived via HKDF.

{% hint style="info" %}
The `SeismicSignedProvider` uses this precompile internally during TEE key exchange. You can also call it directly for custom key agreement workflows.
{% endhint %}

## Precompile Address

```
0x0000000000000000000000000000000000000065
```

## Input Encoding

| Field        | Size     | Description                                                    |
| ------------ | -------- | -------------------------------------------------------------- |
| `secret_key` | 32 bytes | secp256k1 private key                                          |
| `public_key` | 33 bytes | Compressed secp256k1 public key (starts with `0x02` or `0x03`) |

The input is the concatenation of `secret_key` (32 bytes) followed by `public_key` (33 bytes), for a total of 65 bytes.

## Output Format

| Field           | Size     | Description                                                             |
| --------------- | -------- | ----------------------------------------------------------------------- |
| `shared_secret` | 32 bytes | HKDF-derived shared secret (x-coordinate of ECDH point, post-processed) |

## Parameters

| Parameter    | Type       | Required | Description                             |
| ------------ | ---------- | -------- | --------------------------------------- |
| `secret_key` | `[u8; 32]` | Yes      | 32-byte secp256k1 private key           |
| `public_key` | `[u8; 33]` | Yes      | 33-byte compressed secp256k1 public key |

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

    let ecdh_address: Address =
        "0x0000000000000000000000000000000000000065".parse()?;

    // Alice's secret key (32 bytes)
    let alice_sk: [u8; 32] = [/* your private key bytes */; 32];

    // Bob's compressed public key (33 bytes, starting with 0x02 or 0x03)
    let bob_pk: [u8; 33] = [/* Bob's compressed public key */; 33];

    // Concatenate sk + pk
    let mut input_bytes = Vec::with_capacity(65);
    input_bytes.extend_from_slice(&alice_sk);
    input_bytes.extend_from_slice(&bob_pk);
    let input = Bytes::from(input_bytes);

    let result = provider
        .call(
            &TransactionRequest::default()
                .to(ecdh_address)
                .input(input.into()),
        )
        .await?;

    let shared_secret: [u8; 32] = result[..32].try_into()?;
    println!("Shared secret: 0x{}", hex::encode(shared_secret));

    Ok(())
}
```

### Two-Party Key Exchange

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let ecdh_address: Address =
        "0x0000000000000000000000000000000000000065".parse()?;

    // Alice's keypair
    let alice_sk: [u8; 32] = [/* ... */; 32];
    let alice_pk: [u8; 33] = [/* Alice's compressed public key */; 33];

    // Bob's keypair
    let bob_sk: [u8; 32] = [/* ... */; 32];
    let bob_pk: [u8; 33] = [/* Bob's compressed public key */; 33];

    // Alice computes shared secret using her sk + Bob's pk
    let mut alice_input = Vec::with_capacity(65);
    alice_input.extend_from_slice(&alice_sk);
    alice_input.extend_from_slice(&bob_pk);

    let alice_result = provider
        .call(
            &TransactionRequest::default()
                .to(ecdh_address)
                .input(Bytes::from(alice_input).into()),
        )
        .await?;

    // Bob computes shared secret using his sk + Alice's pk
    let mut bob_input = Vec::with_capacity(65);
    bob_input.extend_from_slice(&bob_sk);
    bob_input.extend_from_slice(&alice_pk);

    let bob_result = provider
        .call(
            &TransactionRequest::default()
                .to(ecdh_address)
                .input(Bytes::from(bob_input).into()),
        )
        .await?;

    // Both produce the same shared secret
    assert_eq!(&alice_result[..32], &bob_result[..32]);
    println!("Shared secrets match!");

    Ok(())
}
```

### Use with AES Encryption

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

    // Step 2: Encrypt with derived AES key
    let encrypt_address: Address =
        "0x0000000000000000000000000000000000000066".parse()?;

    let nonce = [0u8; 12]; // 12-byte nonce
    let plaintext = b"Secret message for peer";

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

1. **Encode parameters** -- Concatenates 32-byte private key and 33-byte compressed public key
2. **Call precompile** -- Issues an `eth_call` to address `0x65` with 3120 gas
3. **Compute ECDH** -- Precompile performs scalar multiplication on the secp256k1 curve
4. **Derive key** -- Applies HKDF to the ECDH point to produce a 32-byte uniform secret

## Gas Cost

Fixed gas cost: **3120 gas**

- 3000 gas for ECDH scalar multiplication
- 120 gas for HKDF key derivation

## Notes

- Uses the secp256k1 elliptic curve (same as Ethereum)
- Public keys must be in compressed format (33 bytes starting with `0x02` or `0x03`)
- The ECDH point is passed through HKDF for key uniformity
- Both parties compute the same shared secret: `ecdh(sk_A, pk_B) == ecdh(sk_B, pk_A)`
- The shared secret can be used directly as an AES-256 key

## Warnings

- **Private key security** -- Never expose or log private keys
- **Public key validation** -- Invalid public keys will cause the precompile to revert
- **Key reuse** -- Using the same keypair for multiple sessions reduces forward secrecy

## See Also

- [Precompiles Overview](./) -- All precompile reference
- [aes-gcm-encrypt](aes-gcm-encrypt.md) -- Encrypt with derived key
- [aes-gcm-decrypt](aes-gcm-decrypt.md) -- Decrypt with derived key
- [hkdf](hkdf.md) -- Key derivation function
- [Encryption](../provider/encryption.md) -- How the provider uses ECDH internally
