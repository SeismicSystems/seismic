---
description: 32-byte secp256k1 private key type
icon: key
---

# PrivateKey

`PrivateKey` is the SDK type for transaction signing keys.

```python
class PrivateKey(Bytes32):
    @staticmethod
    def from_hex_str(hex_string: str) -> PrivateKey:
        ...
```

## Construction

- `PrivateKey(...)` validates 32 bytes
- `PrivateKey.from_hex_str(...)` parses a hex string (with/without lowercase `0x`)

## Example

```python
import os
from seismic_web3 import PrivateKey

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
```
