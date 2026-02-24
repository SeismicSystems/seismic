---
description: 12-byte AES-GCM nonce type
icon: lock
---

# EncryptionNonce

Fixed-size nonce type used in AES-GCM encryption.

```python
class EncryptionNonce(_SizedHexBytes):
    _size = 12
```

## Validation

- Must be exactly 12 bytes
- Raises `ValueError` on invalid length

## Example

```python
from seismic_web3 import EncryptionNonce

nonce = EncryptionNonce("0x" + "00" * 12)
```
