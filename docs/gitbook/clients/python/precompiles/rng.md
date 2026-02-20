---
description: Generate random bytes on-chain
icon: dice
---

# rng

Generate random bytes on-chain using Mercury EVM's RNG precompile.

## Overview

`rng()` and `async_rng()` call the RNG precompile at address `0x64` to generate cryptographically secure random bytes directly on the Seismic node. The random value is returned as a Python integer.

## Signature

```python
def rng(
    w3: Web3,
    *,
    num_bytes: int,
    pers: bytes = b"",
) -> int

async def async_rng(
    w3: AsyncWeb3,
    *,
    num_bytes: int,
    pers: bytes = b"",
) -> int
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `w3` | `Web3` or `AsyncWeb3` | Yes | Web3 instance connected to a Seismic node |
| `num_bytes` | `int` | Yes | Number of random bytes to generate (1-32) |
| `pers` | `bytes` | No | Optional personalization bytes to seed the RNG |

## Returns

| Type | Description |
|------|-------------|
| `int` | Random value as a Python integer (derived from `num_bytes` random bytes) |

## Examples

### Basic Usage

```python
from seismic_web3.precompiles import rng
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Generate 32 random bytes
random_value = rng(w3, num_bytes=32)
print(f"Random: {random_value}")
```

### With Personalization

```python
from seismic_web3.precompiles import rng
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Generate random bytes with personalization string
random_value = rng(w3, num_bytes=16, pers=b"my-app-seed")
print(f"Random: {random_value}")
```

### Async Usage

```python
from seismic_web3.precompiles import async_rng
from web3 import AsyncWeb3

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))

    # Generate random bytes asynchronously
    random_value = await async_rng(w3, num_bytes=32)
    print(f"Random: {random_value}")

# Run with asyncio.run(main())
```

### Generate Multiple Random Values

```python
from seismic_web3.precompiles import rng
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Generate multiple random values
randoms = [rng(w3, num_bytes=32) for _ in range(5)]
for i, val in enumerate(randoms):
    print(f"Random {i}: {val}")
```

### Convert to Bytes

```python
from seismic_web3.precompiles import rng
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Generate random value and convert back to bytes
random_int = rng(w3, num_bytes=32)
random_bytes = random_int.to_bytes(32, "big")
print(f"Random bytes: {random_bytes.hex()}")
```

## How It Works

1. **Encode parameters** - `num_bytes` is encoded as a 4-byte big-endian integer, followed by optional `pers` bytes
2. **Call precompile** - Issues an `eth_call` to address `0x64` with estimated gas
3. **Decode result** - Result bytes are padded to 32 bytes and interpreted as a big-endian unsigned integer

## Gas Cost

The gas cost is calculated as:
```python
init_cost = 3500 + (len(pers) // 136) * 5
fill_cost = (num_bytes // 136) * 5
total_gas = init_cost + fill_cost
```

The base cost is 3500 gas, with 5 gas per 136-byte block for personalization and output.

## Notes

- `num_bytes` must be between 1 and 32 (inclusive)
- The RNG uses Strobe128 internally for cryptographic security
- Each call generates independent random values
- Personalization string can be used to domain-separate random values
- The returned integer represents the random bytes as a big-endian unsigned value

## Warnings

- **Range validation** - `num_bytes` outside 1-32 will raise `ValueError`
- **Node connectivity** - Requires a working connection to a Seismic node
- **Not for consensus** - Results may differ across nodes due to timing

## See Also

- [ecdh](ecdh.md) - On-chain ECDH key exchange
- [hkdf](hkdf.md) - On-chain HKDF key derivation
- [Bytes32](../api-reference/types/bytes32.md) - 32-byte value type
