---
description: Call Mercury EVM precompiles from Rust
icon: microchip
---

# Precompiles

Mercury EVM ships with six cryptographic precompiles at fixed addresses. They are callable via `eth_call` from any provider connected to a Seismic node — no encryption state or wallet is required.

## Convenience Helpers

The `seismic_alloy_provider::precompiles` module provides three layers of helpers:

1. **Address constants** — `precompiles::addresses::RNG`, `precompiles::addresses::ECDH`, etc.
2. **Encode/decode functions** — `precompiles::encode_rng()`, `precompiles::decode_secp256k1_sign()`, etc.
3. **Async call wrappers** — `precompiles::call::rng()`, `precompiles::call::ecdh()`, etc.

```rust
use seismic_alloy_provider::precompiles;

// Generate 32 random bytes
let random = precompiles::call::rng::<SeismicFoundry, _>(&provider, 32, b"my_domain").await?;

// Derive a shared AES key via ECDH
let aes_key = precompiles::call::ecdh::<SeismicFoundry, _>(&provider, &secret_key, &pubkey).await?;

// AES-GCM encrypt/decrypt
let ciphertext = precompiles::call::aes_encrypt::<SeismicFoundry, _>(&provider, &key, &nonce, plaintext).await?;
let plaintext = precompiles::call::aes_decrypt::<SeismicFoundry, _>(&provider, &key, &nonce, &ciphertext).await?;
```

{% hint style="info" %}
Precompile calls are read-only `eth_call` operations. They do not require a `SeismicSignedProvider` — an unsigned provider works fine. However, if you use a signed provider, the call will still succeed.
{% endhint %}

## Manual Calling Pattern

You can also call precompiles directly by building a `TransactionRequest`:

```rust
use alloy_primitives::{address, Bytes};
use alloy_provider::Provider;

let precompile_address = address!("0x0000000000000000000000000000000000000064");
let input = precompiles::encode_rng(32, b"my_domain");

let result = provider.call(
    SeismicTransactionRequest::default().to(precompile_address).into()
).await?;
```

---

## Reference

| Precompile      | Address      | Description                                     | Page                                  |
| --------------- | ------------ | ----------------------------------------------- | ------------------------------------- |
| RNG             | `0x64` (100) | Cryptographically secure random byte generation | [rng](rng.md)                         |
| ECDH            | `0x65` (101) | Elliptic-curve Diffie-Hellman key exchange      | [ecdh](ecdh.md)                       |
| AES-GCM Encrypt | `0x66` (102) | AES-256-GCM authenticated encryption            | [aes-gcm-encrypt](aes-gcm-encrypt.md) |
| AES-GCM Decrypt | `0x67` (103) | AES-256-GCM authenticated decryption            | [aes-gcm-decrypt](aes-gcm-decrypt.md) |
| HKDF            | `0x68` (104) | HMAC-based key derivation (HKDF-SHA256)         | [hkdf](hkdf.md)                       |
| secp256k1 Sign  | `0x69` (105) | On-chain ECDSA signature generation             | [secp256k1-sign](secp256k1-sign.md)   |

## Quick Example

```rust
use seismic_prelude::client::*;
use seismic_alloy_provider::precompiles;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://testnet-1.seismictest.net/rpc".parse()?;
    // Unsigned provider — connect_http is synchronous
    let provider = SeismicProviderBuilder::new().connect_http(url);

    // RNG: generate 32 random bytes with domain separation
    let random = precompiles::call::rng::<SeismicReth, _>(
        &provider, 32, b"my_domain"
    ).await?;
    println!("Random bytes: 0x{}", hex::encode(&random));

    // HKDF: derive an AES-256 key
    let key = precompiles::call::hkdf::<SeismicReth, _>(
        &provider, b"input_key_material"
    ).await?;
    println!("Derived key: 0x{}", hex::encode(key));

    Ok(())
}
```

## Internal Usage

The `SeismicSignedProvider` uses several of these precompiles internally as part of the filler pipeline:

- **ECDH** (`0x65`) — Used during TEE key exchange to derive a shared secret between the client and the Seismic node's TEE
- **AES-GCM Encrypt** (`0x66`) — Used to encrypt calldata before sending shielded transactions
- **AES-GCM Decrypt** (`0x67`) — Used to decrypt responses from signed reads
- **HKDF** (`0x68`) — Used to derive AES keys from ECDH shared secrets

You can also call these precompiles directly for custom cryptographic workflows.

## See Also

- [Provider Overview](../provider/) — Signed and unsigned provider types
- [Encryption](../provider/encryption.md) — How the provider uses precompiles internally
- [Contract Interaction](../contract-interaction/) — Shielded and transparent calls
