---
description: 32-byte secp256k1 private key
icon: key
---

# PrivateKey

32-byte secp256k1 private key used for transaction signing and ECDH key derivation.

## Definition

```python
class PrivateKey(Bytes32):
    @staticmethod
    def from_hex_str(hex_string: str) -> PrivateKey:
        ...
```

## Construction

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `val` | `bytes \| str \| int` | Yes | Hex string, raw bytes, or integer |

Raises `ValueError` if length is not exactly 32 bytes.

## Methods

### from\_hex\_str()

Create a `PrivateKey` from a hex string, with or without `0x` prefix. Shorthand for `PrivateKey(hex_to_bytes(hex_string))`.

```python
@staticmethod
def from_hex_str(hex_string: str) -> PrivateKey
```

## Example

```python
import os
from seismic_web3 import PrivateKey

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
```

## Notes

- Subclass of [`Bytes32`](bytes32.md) — inherits immutability, hashability, `.to_0x_hex()`
- Compatible with `eth_keys.PrivateKey` via `eth_keys.PrivateKey(bytes(pk))`
- No validation on mathematical validity (i.e., no check that `0 < key < n`)

## See Also

- [Bytes32](bytes32.md) — parent type
- [CompressedPublicKey](compressed-public-key.md) — corresponding public key type
- [hex\_to\_bytes](hex-to-bytes.md) — general-purpose hex decoding
- [sign\_seismic\_tx\_eip712](../eip712/sign-seismic-tx-eip712.md) — sign transactions with private key
