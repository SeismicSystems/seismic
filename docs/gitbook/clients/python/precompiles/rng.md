---
description: Generate random bytes on-chain
icon: dice
---

# rng

Generate random bytes on-chain using the RNG precompile at `0x64`.

## Overview

`rng()` and `async_rng()` return randomness as a Python `int`.

Input encoding is:
- `num_bytes` as a 4-byte big-endian `uint32`
- optional `pers` bytes appended after it

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
|---|---|---|---|
| `w3` | `Web3` / `AsyncWeb3` | Yes | Connected Seismic client |
| `num_bytes` | `int` | Yes | Number of random bytes to request (`1..32`) |
| `pers` | `bytes` | No | Optional personalization bytes |

## Returns

| Type | Description |
|---|---|
| `int` | Random value interpreted as big-endian unsigned integer |

## Examples

### Basic Usage

```python
from seismic_web3 import create_public_client
from seismic_web3 import precompiles as sp

w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

value = sp.rng(w3, num_bytes=32)
print(value)
```

### With Personalization

```python
value = sp.rng(w3, num_bytes=16, pers=b"my-domain")
```

### Async Usage

```python
from seismic_web3 import create_async_public_client
from seismic_web3 import precompiles as sp

async def main():
    w3 = create_async_public_client("https://gcp-1.seismictest.net/rpc")
    value = await sp.async_rng(w3, num_bytes=32)
    print(value)
```

## Gas Cost

The SDK uses:

```python
from math import ceil

init_cost = 3500 + ceil(len(pers) / 32) * 5
fill_cost = ceil(num_bytes / 32) * 5
total_gas = init_cost + fill_cost
```

Examples:
- `num_bytes=1`, empty `pers`: `3505`
- `num_bytes=32`, empty `pers`: `3505`
- `num_bytes=16`, `len(pers)=64`: `3515`

## Notes

- `num_bytes` outside `1..32` raises `ValueError` before RPC.
- Output is right-padded to 32 bytes and decoded as a `uint256`-style integer.
- This function is for node-provided randomness; do not use it as consensus-critical randomness.

## See Also

- [ecdh](ecdh.md)
- [hkdf](hkdf.md)
