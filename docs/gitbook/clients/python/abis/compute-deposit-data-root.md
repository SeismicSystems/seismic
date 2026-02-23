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

## Example

```python
from seismic_web3 import (
    compute_deposit_data_root,
    make_withdrawal_credentials,
)

withdrawal_credentials = make_withdrawal_credentials(
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
)

root = compute_deposit_data_root(
    node_pubkey=bytes.fromhex("00" * 32),
    consensus_pubkey=bytes.fromhex("00" * 48),
    withdrawal_credentials=withdrawal_credentials,
    node_signature=bytes.fromhex("00" * 64),
    consensus_signature=bytes.fromhex("00" * 96),
    amount_gwei=32_000_000_000,
)

print(root.hex())
```
