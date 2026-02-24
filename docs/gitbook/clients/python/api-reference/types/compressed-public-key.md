---
description: 33-byte compressed secp256k1 public key
icon: key
---

# CompressedPublicKey

Represents a compressed secp256k1 public key.

```python
class CompressedPublicKey(_SizedHexBytes):
    _size = 33
```

## Validation

- Length must be 33 bytes
- First byte must be `0x02` or `0x03`

## Example

```python
from seismic_web3 import CompressedPublicKey

pk = CompressedPublicKey("0x02" + "11" * 32)
print(pk.hex())
```
