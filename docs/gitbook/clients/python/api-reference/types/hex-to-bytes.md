---
description: Convert a hex string to raw bytes
icon: arrows-rotate
---

# hex\_to\_bytes

Convert a hex string to raw bytes, stripping an optional `0x` prefix.

## Signature

```python
def hex_to_bytes(hex_string: str) -> bytes
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `hex_string` | `str` | Yes | Hex-encoded string, with or without `"0x"` prefix |

## Returns

| Type | Description |
|------|-------------|
| `bytes` | Decoded raw bytes |

Raises `ValueError` if `hex_string` contains non-hex characters or has odd length.

## Example

```python
from seismic_web3 import hex_to_bytes

data = hex_to_bytes("0xdeadbeef")  # b'\xde\xad\xbe\xef'
data = hex_to_bytes("deadbeef")    # same result
```

## Notes

- Equivalent to `bytes.fromhex(hex_string.removeprefix("0x"))`
- Does not validate length — use [`Bytes32`](bytes32.md) or [`PrivateKey`](private-key.md) for length-checked values
- For loading private keys from hex, prefer [`PrivateKey.from_hex_str()`](private-key.md) which combines decoding and validation

## See Also

- [PrivateKey.from\_hex\_str()](private-key.md) — hex string to private key
- [Bytes32](bytes32.md) — 32-byte value type
