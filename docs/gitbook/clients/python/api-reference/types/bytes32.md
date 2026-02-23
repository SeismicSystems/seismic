---
description: Fixed-size 32-byte value type
icon: hash
---

# Bytes32

`Bytes32` is a fixed-size byte type used for hashes, keys, and other 32-byte values.

```python
class Bytes32(_SizedHexBytes):
    _size = 32
```

## Construction

`Bytes32` accepts the same inputs as `HexBytes` (`bytes`, hex `str`, or `int`) and validates length.

- Valid length: exactly 32 bytes
- Error: raises `ValueError` for any other length

## Notes

- `Bytes32` is a subclass of `HexBytes`, so it also behaves like `bytes`.
- Common uses in this SDK: AES keys, block hashes, and fixed-width fields.
