---
description: 33-byte compressed secp256k1 public key
icon: unlock
---

# CompressedPublicKey

33-byte compressed secp256k1 public key with validated prefix byte.

## Definition

```python
class CompressedPublicKey(_SizedHexBytes):
    _size = 33
```

## Construction

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `val` | `bytes \| str \| int` | Yes | Hex string, raw bytes, or integer |

Raises `ValueError` if:
- Length is not exactly 33 bytes
- First byte is not `0x02` or `0x03`

## Example

```python
from eth_keys import keys as eth_keys
from seismic_web3 import PrivateKey, CompressedPublicKey
import os

private_key = PrivateKey(os.urandom(32))
sk = eth_keys.PrivateKey(bytes(private_key))
pubkey = CompressedPublicKey(sk.public_key.to_compressed_bytes())
```

## Compressed format

Seismic uses compressed public keys exclusively:
- 1-byte prefix (`0x02` for even y, `0x03` for odd y) + 32-byte x-coordinate
- 33 bytes total, versus 65 bytes for uncompressed keys

## See Also

- [PrivateKey](private-key.md) — corresponding private key type
- [SeismicElements](../transaction-types/seismic-elements.md) — uses CompressedPublicKey for `encryption_pubkey`
- [ecdh](../precompiles/ecdh.md) — ECDH key derivation with public keys
