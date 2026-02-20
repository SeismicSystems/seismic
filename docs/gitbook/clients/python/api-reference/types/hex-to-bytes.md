---
description: Convert a hex string to raw bytes
icon: arrows-rotate
---

# hex\_to\_bytes

Convert a hex string to raw bytes, stripping an optional `0x` prefix.

## Overview

`hex_to_bytes()` is a convenience wrapper around `bytes.fromhex()` that accepts both `"0xabcd…"` and `"abcd…"` formats. It handles the common case of hex-encoded data that may or may not include an Ethereum-style `0x` prefix.

## Signature

```python
def hex_to_bytes(hex_string: str) -> bytes:
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `hex_string` | `str` | Yes | Hex-encoded string, with or without a `"0x"` prefix |

## Returns

| Type | Description |
|------|-------------|
| `bytes` | Decoded raw bytes |

## Raises

- `ValueError` — If `hex_string` contains non-hex characters or has odd length after prefix removal

## Examples

### Basic Usage

```python
from seismic_web3 import hex_to_bytes

# With 0x prefix
data = hex_to_bytes("0xdeadbeef")  # b'\xde\xad\xbe\xef'

# Without prefix
data = hex_to_bytes("deadbeef")    # b'\xde\xad\xbe\xef'
```

### Load Key from Environment

```python
import os
from seismic_web3 import hex_to_bytes, Bytes32

# Load a 32-byte viewing key from an environment variable
viewing_key = Bytes32(hex_to_bytes(os.environ["VIEWING_KEY"]))
```

### Validate Hex Input

```python
from seismic_web3 import hex_to_bytes

try:
    data = hex_to_bytes("not-valid-hex")
except ValueError as e:
    print(f"Invalid hex: {e}")
```

## Notes

- Equivalent to `bytes.fromhex(hex_string.removeprefix("0x"))`
- Does **not** validate length — use [`Bytes32`](bytes32.md) or [`PrivateKey`](private-key.md) for length-checked values
- For loading private keys from hex, prefer [`PrivateKey.from_hex_str()`](private-key.md#from_hex_str) which combines decoding and validation

## See Also

- [PrivateKey.from\_hex\_str()](private-key.md#from_hex_str) — Load a private key from a hex string
- [Bytes32](bytes32.md) — 32-byte value type
- [PrivateKey](private-key.md) — 32-byte private key type
