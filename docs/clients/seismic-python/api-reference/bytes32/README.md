---
description: Fixed-size 32-byte value type
icon: hashtag
---

# Types

Exactly 32 bytes — used for hashes, AES keys, and similar values.

## Overview

`Bytes32` is a fixed-size byte type that validates its length at construction time. It's used throughout the SDK for values that must be exactly 32 bytes, such as block hashes, AES-GCM keys, and Keccak-256 hashes.

## Definition

```python
class Bytes32(HexBytes):
    """Exactly 32 bytes — used for hashes, AES keys, and similar values."""
    _size = 32
```

## Inheritance

* Inherits from `HexBytes` (from the `hexbytes` package)
* `HexBytes` itself inherits from Python's `bytes` type
* Fully compatible with any API expecting `bytes`

## Construction

`Bytes32` can be constructed from:

* Hex strings (with or without `"0x"` prefix)
* Raw `bytes` objects
* Integer values

### Parameters

| Parameter | Type                  | Required | Description                      |
| --------- | --------------------- | -------- | -------------------------------- |
| `val`     | `bytes \| str \| int` | Yes      | The value to convert to 32 bytes |

### Raises

* `ValueError` - If the resulting length is not exactly 32 bytes

## Examples

### From Hex String

```python
from seismic_web3 import Bytes32

# With 0x prefix
block_hash = Bytes32("0x1234567890abcdef" + "0" * 48)

# Without 0x prefix
key = Bytes32("1234567890abcdef" + "0" * 48)
```

### From Raw Bytes

```python
import os
from seismic_web3 import Bytes32

# Generate random 32 bytes
random_key = Bytes32(os.urandom(32))
```

### From Integer

```python
from seismic_web3 import Bytes32

# Convert integer to 32-byte big-endian representation
value = Bytes32(12345)
```

### Length Validation

```python
from seismic_web3 import Bytes32

# This will raise ValueError
try:
    invalid = Bytes32("0x1234")  # Too short
except ValueError as e:
    print(e)  # "Bytes32: expected 32 bytes, got 2"
```

## Methods

### to\_0x\_hex()

Returns the hex string representation with `"0x"` prefix.

```python
key = Bytes32(os.urandom(32))
hex_str = key.to_0x_hex()  # "0xabcd...ef"
```

## Properties

* **Immutable** - Cannot be modified after construction
* **Hashable** - Can be used as dictionary keys or in sets
* **Compatible with bytes** - Passes `isinstance(x, bytes)` checks

## Notes

* All `Bytes32` instances have `len() == 32`
* Type validation happens at construction time
* Compatible with all web3.py APIs expecting bytes
* Commonly used for block hashes in [`SeismicElements`](../signature/seismic-elements.md)

## See Also

* [PrivateKey](private-key.md) - 32-byte private key (subclass of Bytes32)
* [CompressedPublicKey](compressed-public-key.md) - 33-byte public key
* [EncryptionNonce](encryption-nonce.md) - 12-byte nonce
* [SeismicElements](../signature/seismic-elements.md) - Uses Bytes32 for `recent_block_hash`
