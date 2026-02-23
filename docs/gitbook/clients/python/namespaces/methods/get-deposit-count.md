---
description: Read deposit count from deposit contract
icon: hash
---

# get_deposit_count

Read the current validator deposit count from the deposit contract.

## Signatures

```python
# sync
w3.seismic.get_deposit_count(*, address: str = DEPOSIT_CONTRACT_ADDRESS) -> int

# async
await w3.seismic.get_deposit_count(*, address: str = DEPOSIT_CONTRACT_ADDRESS) -> int
```

## Parameters

- `address`: deposit contract address (defaults to `DEPOSIT_CONTRACT_ADDRESS`)

## Returns

Deposit count as Python `int`.

## Implementation detail

The contract returns bytes; SDK decodes bytes `[64:72]` as little-endian `uint64`.
