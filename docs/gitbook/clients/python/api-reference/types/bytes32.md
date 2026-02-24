---
description: Fixed-size 32-byte value type
icon: hashtag
---

# Bytes32

Exactly 32 bytes — used for hashes, AES keys, and similar values.

## Definition

```python
class Bytes32(_SizedHexBytes):
    _size = 32
```

## Construction

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `val` | `bytes \| str \| int` | Yes | Hex string, raw bytes, or integer |

Raises `ValueError` if length is not exactly 32 bytes.

## Example

```python
from seismic_web3 import Bytes32

block_hash = Bytes32("0x" + "ab" * 32)
print(len(block_hash))        # 32
print(block_hash.to_0x_hex())
```

## Key properties

- Inherits from `HexBytes` — passes `isinstance(x, bytes)` checks
- Immutable and hashable
- Has `.to_0x_hex()` method for `"0x"`-prefixed hex output

## See Also

- [PrivateKey](private-key.md) — 32-byte subclass for signing keys
- [CompressedPublicKey](compressed-public-key.md) — 33-byte public key
- [EncryptionNonce](encryption-nonce.md) — 12-byte nonce
