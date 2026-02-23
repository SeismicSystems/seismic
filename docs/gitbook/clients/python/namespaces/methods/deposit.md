---
description: Submit a validator deposit transaction
icon: banknote
---

# deposit

Send a transparent `deposit(...)` transaction to the deposit contract.

## Signatures

```python
# sync
w3.seismic.deposit(
    *,
    node_pubkey: bytes,
    consensus_pubkey: bytes,
    withdrawal_credentials: bytes,
    node_signature: bytes,
    consensus_signature: bytes,
    deposit_data_root: bytes,
    value: int,
    address: str = DEPOSIT_CONTRACT_ADDRESS,
) -> HexBytes

# async
await w3.seismic.deposit(...same args...) -> HexBytes
```

## Required byte lengths

- `node_pubkey`: 32
- `consensus_pubkey`: 48
- `withdrawal_credentials`: 32
- `node_signature`: 64
- `consensus_signature`: 96
- `deposit_data_root`: 32

`ValueError` is raised on length mismatch.

## Notes

- This is a transparent transaction (`eth_sendTransaction`), not shielded.
- `value` is the deposit amount in wei.
