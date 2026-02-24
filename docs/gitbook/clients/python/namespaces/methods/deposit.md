---
description: Submit a validator deposit transaction
icon: coins
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

| Parameter | Length |
|-----------|--------|
| `node_pubkey` | 32 |
| `consensus_pubkey` | 48 |
| `withdrawal_credentials` | 32 |
| `node_signature` | 64 |
| `consensus_signature` | 96 |
| `deposit_data_root` | 32 |

`ValueError` is raised on length mismatch.

## Example

```python
tx_hash = w3.seismic.deposit(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    deposit_data_root=deposit_data_root,
    value=32 * 10**18,
)
```

## Notes

- Uses transparent `eth_sendTransaction` semantics — deposit data is public, not shielded
- `value` is deposit amount in wei (typically `32 * 10**18` for a full validator)
- All parameters are keyword-only to prevent mix-ups between the many `bytes` arguments

## See Also

- [get_deposit_count](get-deposit-count.md) — Read total deposits
- [get_deposit_root](get-deposit-root.md) — Read deposit Merkle root
