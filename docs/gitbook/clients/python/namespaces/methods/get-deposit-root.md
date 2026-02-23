---
description: Read deposit Merkle root from deposit contract
icon: hash
---

# get_deposit_root

Read the current deposit Merkle root from the deposit contract.

## Signatures

```python
# sync
w3.seismic.get_deposit_root(*, address: str = DEPOSIT_CONTRACT_ADDRESS) -> bytes

# async
await w3.seismic.get_deposit_root(*, address: str = DEPOSIT_CONTRACT_ADDRESS) -> bytes
```

## Parameters

- `address`: deposit contract address (defaults to `DEPOSIT_CONTRACT_ADDRESS`)

## Returns

32-byte root as `bytes`.
