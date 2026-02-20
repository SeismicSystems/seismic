---
description: Generate random bytes on-chain
icon: dice
---

# RNG

Generate cryptographically secure random bytes on-chain using Mercury EVM's RNG precompile.

## Overview

The RNG precompile at address `0x64` generates random bytes directly on the Seismic node using the Strobe128 construction. The random value is returned as raw bytes (padded to 32 bytes).

## Precompile Address

```
0x0000000000000000000000000000000000000064
```

## Input Encoding

| Field             | Size                     | Description                                    |
| ----------------- | ------------------------ | ---------------------------------------------- |
| `num_bytes`       | 4 bytes (big-endian u32) | Number of random bytes to generate (1--32)     |
| `personalization` | Variable (optional)      | Optional personalization bytes to seed the RNG |

The input is the concatenation of `num_bytes` followed by the optional `personalization` bytes.

## Output Format

| Field          | Size     | Description                                    |
| -------------- | -------- | ---------------------------------------------- |
| `random_bytes` | 32 bytes | Random output (padded to 32 bytes, big-endian) |

## Parameters

| Parameter         | Type    | Required | Description                                                      |
| ----------------- | ------- | -------- | ---------------------------------------------------------------- |
| `num_bytes`       | `u32`   | Yes      | Number of random bytes to generate (1--32)                       |
| `personalization` | `&[u8]` | No       | Optional personalization string to domain-separate random values |

## Examples

### Basic Usage

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes, U256};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let rng_address: Address =
        "0x0000000000000000000000000000000000000064".parse()?;

    // Request 32 random bytes
    let num_bytes: u32 = 32;
    let input = Bytes::from(num_bytes.to_be_bytes().to_vec());

    let result = provider
        .call(
            &TransactionRequest::default()
                .to(rng_address)
                .input(input.into()),
        )
        .await?;

    let random_value = U256::from_be_slice(&result);
    println!("Random value: {random_value}");

    Ok(())
}
```

### With Personalization

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes, U256};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let rng_address: Address =
        "0x0000000000000000000000000000000000000064".parse()?;

    // Request 16 random bytes with personalization
    let num_bytes: u32 = 16;
    let pers = b"my-app-seed";

    let mut input_bytes = num_bytes.to_be_bytes().to_vec();
    input_bytes.extend_from_slice(pers);
    let input = Bytes::from(input_bytes);

    let result = provider
        .call(
            &TransactionRequest::default()
                .to(rng_address)
                .input(input.into()),
        )
        .await?;

    let random_value = U256::from_be_slice(&result);
    println!("Random (with pers): {random_value}");

    Ok(())
}
```

### Generate Multiple Random Values

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes, U256};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let rng_address: Address =
        "0x0000000000000000000000000000000000000064".parse()?;

    let num_bytes: u32 = 32;
    let input = Bytes::from(num_bytes.to_be_bytes().to_vec());

    for i in 0..5 {
        let result = provider
            .call(
                &TransactionRequest::default()
                    .to(rng_address)
                    .input(input.clone().into()),
            )
            .await?;

        let random_value = U256::from_be_slice(&result);
        println!("Random {i}: {random_value}");
    }

    Ok(())
}
```

### Convert to Fixed-Size Array

```rust
use alloy::providers::Provider;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_types_eth::TransactionRequest;
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let rng_address: Address =
        "0x0000000000000000000000000000000000000064".parse()?;

    let num_bytes: u32 = 32;
    let input = Bytes::from(num_bytes.to_be_bytes().to_vec());

    let result = provider
        .call(
            &TransactionRequest::default()
                .to(rng_address)
                .input(input.into()),
        )
        .await?;

    // Convert to a [u8; 32] array
    let mut random_bytes = [0u8; 32];
    random_bytes.copy_from_slice(&result[..32]);
    println!("Random bytes: 0x{}", hex::encode(random_bytes));

    Ok(())
}
```

## How It Works

1. **Encode parameters** -- `num_bytes` is encoded as a 4-byte big-endian integer, followed by optional `personalization` bytes
2. **Call precompile** -- Issues an `eth_call` to address `0x64` with estimated gas
3. **Generate randomness** -- The precompile uses Strobe128 internally for cryptographic security
4. **Decode result** -- Result bytes are padded to 32 bytes and returned as big-endian

## Gas Cost

The gas cost is calculated as:

```
init_cost = 3500 + (len(pers) / 136) * 5
fill_cost = (num_bytes / 136) * 5
total_gas = init_cost + fill_cost
```

The base cost is **3500 gas**, with 5 gas per 136-byte block for personalization and output.

## Notes

- `num_bytes` must be between 1 and 32 (inclusive)
- The RNG uses Strobe128 internally for cryptographic security
- Each call generates independent random values
- The personalization string can be used to domain-separate random values
- The result is padded to 32 bytes and returned as a big-endian value

## Warnings

- **Range validation** -- `num_bytes` outside 1--32 will cause the precompile to revert
- **Node connectivity** -- Requires a working connection to a Seismic node
- **Not for consensus** -- Results may differ across nodes due to timing

## See Also

- [Precompiles Overview](./) -- All precompile reference
- [ecdh](ecdh.md) -- On-chain ECDH key exchange
- [hkdf](hkdf.md) -- On-chain HKDF key derivation
