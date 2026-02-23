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

- `PrivateKey(...)`: accepts `bytes`, hex `str`, or `int` and enforces 32 bytes.
- `PrivateKey.from_hex_str(...)`: helper that parses a hex string (with or without lowercase `0x`) and returns `PrivateKey`.

## Example

```python
import os
from seismic_web3 import PrivateKey

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
```

## Notes

- This type is also a `HexBytes` subtype.
- Never log private keys in production code.
