---
description: Fixed-size 32-byte value type
icon: hash
---

# Bytes32

`Bytes32` is a fixed-size byte type used for hashes, keys, and similar fields.

```python
class Bytes32(_SizedHexBytes):
    _size = 32
```

## Behavior

- Accepts `bytes`, hex `str`, or `int`
- Raises `ValueError` if length is not exactly 32 bytes

## Example

```python
from seismic_web3 import Bytes32

x = Bytes32("0x" + "11" * 32)
print(len(x))
```
