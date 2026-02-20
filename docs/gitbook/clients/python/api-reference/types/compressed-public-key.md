---
description: 33-byte compressed secp256k1 public key
icon: unlock
---

# CompressedPublicKey

33-byte compressed secp256k1 public key with validated prefix byte.

## Overview

`CompressedPublicKey` is a fixed-size byte type representing a compressed secp256k1 public key. It validates both the length (33 bytes) and the prefix byte (`0x02` or `0x03`) at construction time.

## Definition

```python
class CompressedPublicKey(HexBytes):
    """33-byte compressed secp256k1 public key (0x02 / 0x03 prefix)."""
    _size = 33
```

## Inheritance

- Inherits from `HexBytes` (from the `hexbytes` package)
- `HexBytes` itself inherits from Python's `bytes` type
- Fully compatible with any API expecting `bytes`

## Construction

`CompressedPublicKey` can be constructed from:
- Hex strings (with or without `"0x"` prefix)
- Raw `bytes` objects
- Integer values

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `val` | `bytes \| str \| int` | Yes | The value to convert to a 33-byte compressed public key |

### Raises

- `ValueError` - If the length is not exactly 33 bytes
- `ValueError` - If the prefix byte is not `0x02` or `0x03`

## Examples

### From Hex String

```python
from seismic_web3 import CompressedPublicKey

# Valid compressed public key (0x02 prefix)
pubkey = CompressedPublicKey(
    "0x02" + "1234567890abcdef" * 4
)

# Valid with 0x03 prefix
pubkey = CompressedPublicKey(
    "0x03" + "fedcba0987654321" * 4
)
```

### From Raw Bytes

```python
from seismic_web3 import CompressedPublicKey

# 33 bytes with valid prefix
raw_bytes = b'\x02' + b'\x12\x34' * 16
pubkey = CompressedPublicKey(raw_bytes)
```

### Derive from Private Key

```python
from eth_keys import keys as eth_keys
from seismic_web3 import PrivateKey, CompressedPublicKey

private_key = PrivateKey(os.urandom(32))
sk = eth_keys.PrivateKey(bytes(private_key))
public_key = CompressedPublicKey(sk.public_key.to_compressed_bytes())
```

### Use in ECDH

```python
from seismic_web3.precompiles import ecdh
from seismic_web3 import CompressedPublicKey, PrivateKey

private_key = PrivateKey(os.urandom(32))
tee_pubkey = CompressedPublicKey(...)  # From w3.seismic.get_tee_public_key()

shared_secret = ecdh(private_key, tee_pubkey)
```

### Validation Errors

```python
from seismic_web3 import CompressedPublicKey

# Wrong length
try:
    invalid = CompressedPublicKey("0x" + "12" * 32)  # 32 bytes, not 33
except ValueError as e:
    print(e)  # "CompressedPublicKey: expected 33 bytes, got 32"

# Invalid prefix
try:
    invalid = CompressedPublicKey("0x01" + "12" * 32)  # Wrong prefix byte
except ValueError as e:
    print(e)  # "Compressed public key must start with 0x02 or 0x03, got 0x01"
```

## Methods

### to_0x_hex()

Returns the hex string representation with `"0x"` prefix.

```python
pubkey = CompressedPublicKey(...)
hex_str = pubkey.to_0x_hex()  # "0x02abcd...ef"
```

## Properties

- **Immutable** - Cannot be modified after construction
- **Hashable** - Can be used as dictionary keys or in sets
- **33 bytes** - Always exactly 33 bytes
- **Valid prefix** - Always starts with `0x02` or `0x03`
- **Compatible with bytes** - Passes `isinstance(x, bytes)` checks

## Compressed vs Uncompressed

Seismic uses **compressed** public keys exclusively:
- **Compressed**: 33 bytes (1-byte prefix + 32-byte x-coordinate)
- **Uncompressed**: 65 bytes (1-byte prefix + 32-byte x-coordinate + 32-byte y-coordinate)

The prefix byte indicates the y-coordinate's parity:
- `0x02` - y-coordinate is even
- `0x03` - y-coordinate is odd

## Notes

- Used in [`SeismicElements`](../transaction-types/seismic-elements.md) for `encryption_pubkey`
- Required for ECDH key derivation in precompiles
- Obtained from `w3.seismic.get_tee_public_key()` for transaction encryption
- Prefix validation ensures only valid compressed keys are accepted

## See Also

- [PrivateKey](private-key.md) - Corresponding private key type
- [Bytes32](bytes32.md) - General 32-byte type
- [SeismicElements](../transaction-types/seismic-elements.md) - Uses CompressedPublicKey for encryption
- [ecdh](../../precompiles/ecdh.md) - ECDH key derivation with public keys
- [get_tee_public_key](../../namespaces/methods/get-tee-public-key.md) - Retrieve TEE public key
