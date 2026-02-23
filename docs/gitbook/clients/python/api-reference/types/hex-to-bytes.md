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

- Removes a lowercase `0x` prefix if present
- Calls `bytes.fromhex(...)` on the remaining string
- Raises `ValueError` if input is not valid hex

## Example

```python
from seismic_web3 import hex_to_bytes

data1 = hex_to_bytes("0xdeadbeef")
data2 = hex_to_bytes("deadbeef")
```
