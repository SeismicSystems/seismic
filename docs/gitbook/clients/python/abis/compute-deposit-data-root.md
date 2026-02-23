---
description: Compute SSZ-style deposit data root
icon: calculator
---

# compute_deposit_data_root

Compute the 32-byte deposit data root expected by the deposit contract.

## Signature

```python
def compute_deposit_data_root(
    *,
    node_pubkey: bytes,
    consensus_pubkey: bytes,
    withdrawal_credentials: bytes,
    node_signature: bytes,
    consensus_signature: bytes,
    amount_gwei: int,
) -> bytes
```

## Required byte lengths

- `node_pubkey`: 32
- `consensus_pubkey`: 48
- `withdrawal_credentials`: 32
- `node_signature`: 64
- `consensus_signature`: 96

Raises `ValueError` on length mismatch.

## Notes

- `amount_gwei` is encoded little-endian `uint64` in the hash construction.
- Output is a 32-byte SHA-256 root that must match the contract verification logic.
