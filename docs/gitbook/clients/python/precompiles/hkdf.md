---
description: On-chain HKDF-SHA256 key derivation
icon: fingerprint
---

# hkdf

Derive a 32-byte key with the HKDF precompile at `0x68`.

## Overview

`hkdf()` and `async_hkdf()` accept arbitrary input key material (`ikm`) and return a `Bytes32` derived key.

## Signature

```python
def hkdf(
    w3: Web3,
    ikm: bytes,
) -> Bytes32

async def async_hkdf(
    w3: AsyncWeb3,
    ikm: bytes,
) -> Bytes32
```

## Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `w3` | `Web3` / `AsyncWeb3` | Yes | Connected Seismic client |
| `ikm` | `bytes` | Yes | Input key material |

## Returns

| Type | Description |
|---|---|
| [`Bytes32`](../api-reference/types/bytes32.md) | 32-byte derived key |

## Examples

### Basic Usage

```python
from seismic_web3 import create_public_client
from seismic_web3 import precompiles as sp

w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

key = sp.hkdf(w3, b"input key material")
print(key.to_0x_hex())
```

### Context Separation

```python
master = b"shared-secret"
encryption_key = sp.hkdf(w3, master + b"\x01encryption")
auth_key = sp.hkdf(w3, master + b"\x02auth")
assert encryption_key != auth_key
```

### Async Usage

```python
from seismic_web3 import create_async_public_client
from seismic_web3 import precompiles as sp

async def main():
    w3 = create_async_public_client("https://gcp-1.seismictest.net/rpc")
    key = await sp.async_hkdf(w3, b"async input")
    print(key.to_0x_hex())
```

## Gas Cost

The SDK computes gas as:

```python
from math import ceil

linear = 3000 + ceil(len(ikm) / 32) * 12
total_gas = 2 * linear + 120
```

Examples:
- `len(ikm)=0`: `6120`
- `len(ikm)=32`: `6144`

## Notes

- Deterministic: the same `ikm` always produces the same output.
- Good for deriving protocol keys from shared secrets.
- Not a password hashing function; use `argon2`/`bcrypt` for password storage.

## See Also

- [ecdh](ecdh.md)
- [aes-gcm-encrypt](aes-gcm-encrypt.md)
- [rng](rng.md)
