---
description: Call Mercury EVM precompiles from Rust
icon: microchip
---

# Precompiles

Mercury EVM ships with six cryptographic precompiles at fixed addresses. They are callable via `eth_call` from any provider connected to a Seismic node -- no encryption state or wallet is required.

## Calling Pattern

Every precompile follows the same pattern: ABI-encode the input, build a `TransactionRequest` targeting the precompile address, and issue an `eth_call`:

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;

// Precompile addresses are 0x64 through 0x69
let precompile_address: Address =
    "0x0000000000000000000000000000000000000064".parse().unwrap();

// Encode input per the precompile's specification
let input = Bytes::from(encoded_params);

// Call via eth_call
let result = provider
    .call(
        &TransactionRequest::default()
            .to(precompile_address)
            .input(input.into()),
    )
    .await?;
```

{% hint style="info" %}
Precompile calls are read-only `eth_call` operations. They do not require a `SeismicSignedProvider` -- an unsigned provider works fine. However, if you use a signed provider, the call will still succeed.
{% endhint %}

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
use seismic_alloy::prelude::*;
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    // RNG precompile: request 32 random bytes
    let rng_address: Address =
        "0x0000000000000000000000000000000000000064".parse()?;

    // Input: 4-byte big-endian num_bytes
    let num_bytes: u32 = 32;
    let input = Bytes::from(num_bytes.to_be_bytes().to_vec());

    let result = provider
        .call(
            &TransactionRequest::default()
                .to(rng_address)
                .input(input.into()),
        )
        .await?;

    println!("Random bytes: 0x{}", hex::encode(&result));
    Ok(())
}
```

## Internal Usage

The `SeismicSignedProvider` uses several of these precompiles internally as part of the filler pipeline:

- **ECDH** (`0x65`) -- Used during TEE key exchange to derive a shared secret between the client and the Seismic node's TEE
- **AES-GCM Encrypt** (`0x66`) -- Used to encrypt calldata before sending shielded transactions
- **AES-GCM Decrypt** (`0x67`) -- Used to decrypt responses from signed reads
- **HKDF** (`0x68`) -- Used to derive AES keys from ECDH shared secrets

You can also call these precompiles directly for custom cryptographic workflows.

## See Also

- [Provider Overview](../provider/) -- Signed and unsigned provider types
- [Encryption](../provider/encryption.md) -- How the provider uses precompiles internally
- [Contract Interaction](../contract-interaction/) -- Shielded and transparent calls
