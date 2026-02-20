---
description: ECDSA signature components
icon: signature
---

# Signature

ECDSA signature components (v, r, s) for signed transactions.

## Overview

`Signature` is an immutable dataclass containing the three components of an ECDSA signature. It's used when manually signing transactions or working with raw signature data.

## Definition

```python
@dataclass(frozen=True)
class Signature:
    """ECDSA signature components (all-or-nothing).

    Attributes:
        v: Recovery identifier (0 or 1, per EIP-155 / y-parity).
        r: First 32-byte integer of the signature.
        s: Second 32-byte integer of the signature.
    """
    v: int
    r: int
    s: int
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `v` | `int` | Recovery identifier (0 or 1, per EIP-155 y-parity) |
| `r` | `int` | First 32-byte integer of the signature |
| `s` | `int` | Second 32-byte integer of the signature |

## Examples

### Create from Signature Components

```python
from seismic_web3 import Signature

sig = Signature(
    v=0,
    r=0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef,
    s=0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321,
)
```

### Extract from eth_keys Signature

```python
from eth_keys import keys as eth_keys
from seismic_web3 import Signature, PrivateKey

private_key = PrivateKey(...)
sk = eth_keys.PrivateKey(bytes(private_key))

# Sign a message hash
msg_hash = b'\x12\x34' * 16
sig_obj = sk.sign_msg_hash(msg_hash)

# Convert to Signature dataclass
sig = Signature(
    v=sig_obj.v,
    r=sig_obj.r,
    s=sig_obj.s,
)
```

### Use in Transaction Serialization

```python
from seismic_web3.transaction.serialize import serialize_signed
from seismic_web3 import Signature, UnsignedSeismicTx

unsigned_tx = UnsignedSeismicTx(...)
signature = Signature(v=0, r=..., s=...)

# Serialize with signature
signed_tx_bytes = serialize_signed(unsigned_tx, signature)
```

## Properties

- **Immutable** - Cannot be modified after construction (`frozen=True`)
- **Hashable** - Can be used as dictionary keys
- **Type-safe** - All fields are validated at construction

## Recovery Identifier (v)

The `v` value in Seismic transactions:
- **0 or 1** - Represents y-parity (EIP-155 style)
- Used to recover the public key from the signature
- Different from legacy Ethereum `v` values (27 or 28)

## Notes

- All three components must be provided together
- Typically created automatically by signing functions
- Used internally by [`sign_seismic_tx_eip712`](../eip712/sign-seismic-tx-eip712.md)
- Part of the final RLP-encoded transaction

## See Also

- [sign_seismic_tx_eip712](../eip712/sign-seismic-tx-eip712.md) - Creates and applies signature
- [UnsignedSeismicTx](unsigned-seismic-tx.md) - Transaction before signing
- [PrivateKey](../types/private-key.md) - Used to generate signatures
