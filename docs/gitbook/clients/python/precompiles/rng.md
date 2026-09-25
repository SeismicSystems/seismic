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

w3 = create_public_client("https://testnet-1.seismictest.net/rpc")

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
    w3 = create_async_public_client("https://testnet-1.seismictest.net/rpc")
    value = await sp.async_rng(w3, num_bytes=32)
    print(value)
```

## Gas Cost

The SDK matches the node's gas schedule. Each 32-byte block of output is one HKDF expansion round, which hashes a 121-byte domain-separation prefix, the personalization, the previous block and a round counter:

```python
from math import ceil

init_cost = 3500 + ceil(len(pers) / 32) * 5
round_cost = 120 + ceil((121 + len(pers) + 33) / 32) * 24
total_gas = init_cost + ceil(num_bytes / 32) * round_cost
```

Examples:
- `num_bytes=1`, empty `pers`: `3740`
- `num_bytes=32`, empty `pers`: `3740`
- `num_bytes=32`, `len(pers)=33`: `3774`

## Notes

- `num_bytes` outside `1..32` raises `ValueError` before RPC.
- Output is right-padded to 32 bytes and decoded as a `uint256`-style integer.
- This function is for node-provided randomness; do not use it as consensus-critical randomness.

## See Also

- [RNG — Precompiles Reference](../../../reference/precompiles/rng.md)
- [ecdh](ecdh.md)
- [hkdf](hkdf.md)
