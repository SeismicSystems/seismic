---
description: 12-byte AES-GCM encryption nonce
icon: shield
---

# EncryptionNonce

12-byte AES-GCM encryption nonce used for calldata encryption.

## Definition

```python
class EncryptionNonce(_SizedHexBytes):
    _size = 12
```

## Construction

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `val` | `bytes \| str \| int` | Yes | Hex string, raw bytes, or integer |

Raises `ValueError` if length is not exactly 12 bytes.

## Example

```python
import os
from seismic_web3 import EncryptionNonce

nonce = EncryptionNonce(os.urandom(12))
print(nonce.to_0x_hex())
```

## Auto-generation

In most cases you don't need to create nonces manually — the SDK generates a random one automatically. Only provide explicit nonces when testing with specific values or implementing custom encryption flows.

## See Also

- [SeismicElements](../transaction-types/seismic-elements.md) — uses EncryptionNonce
- [SeismicSecurityParams](../transaction-types/seismic-security-params.md) — optional nonce override
- [Bytes32](bytes32.md) — 32-byte type
