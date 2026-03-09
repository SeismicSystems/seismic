---
description: ECDSA signature components
icon: signature
---

# Signature

ECDSA signature components for signed Seismic transactions.

## Definition

```python
@dataclass(frozen=True)
class Signature:
    v: int
    r: int
    s: int
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `v` | `int` | Recovery identifier (0 or 1, y-parity) |
| `r` | `int` | First 32-byte integer of the signature |
| `s` | `int` | Second 32-byte integer of the signature |

## Example

```python
from eth_keys import keys as eth_keys
from seismic_web3 import Signature, PrivateKey

sk = eth_keys.PrivateKey(bytes(PrivateKey(...)))
sig_obj = sk.sign_msg_hash(b"\xab" * 32)

sig = Signature(v=sig_obj.v, r=sig_obj.r, s=sig_obj.s)
```

## Notes

- `v` is 0 or 1 (EIP-155 y-parity), not the legacy 27/28 values
- Typically created internally by signing functions — most users won't construct this directly

## See Also

- [sign\_seismic\_tx\_eip712](../eip712/sign-seismic-tx-eip712.md) — creates and applies signature
- [UnsignedSeismicTx](unsigned-seismic-tx.md) — transaction before signing
- [PrivateKey](../types/private-key.md) — used to generate signatures
