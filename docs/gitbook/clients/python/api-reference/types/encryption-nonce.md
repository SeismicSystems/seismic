---
description: 12-byte AES-GCM nonce type
icon: lock
---

# EncryptionNonce

`EncryptionNonce` is the fixed-size nonce type used for AES-GCM encryption.

```python
class EncryptionNonce(_SizedHexBytes):
    _size = 12
```

## Validation

Construction enforces an exact length of 12 bytes.

- Valid length: 12 bytes (96 bits)
- Error: raises `ValueError` for invalid length

## Usage

Used in `SeismicElements.encryption_nonce` and encryption/decryption helpers.
