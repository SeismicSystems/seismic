---
description: 33-byte compressed secp256k1 public key
icon: key
---

# CompressedPublicKey

`CompressedPublicKey` represents a compressed secp256k1 public key.

```python
class CompressedPublicKey(_SizedHexBytes):
    _size = 33
```

## Validation

Construction validates:

- Length is exactly 33 bytes
- First byte is `0x02` or `0x03`

If validation fails, a `ValueError` is raised.

## Usage

This type is used for TEE public keys and encryption public keys in Seismic metadata.
