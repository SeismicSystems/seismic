---
description: Hex string to raw bytes helper
icon: brackets
---

# hex_to_bytes

Convert a hex string to raw bytes.

## Signature

```python
def hex_to_bytes(hex_string: str) -> bytes
```

## Behavior

- Strips optional lowercase `0x`
- Uses `bytes.fromhex(...)`
- Raises `ValueError` on invalid input

## Example

```python
from seismic_web3 import hex_to_bytes

raw = hex_to_bytes("0xdeadbeef")
```
