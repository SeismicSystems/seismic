---
description: Read deposit count from deposit contract
icon: hashtag
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

## Example

```python
count = w3.seismic.get_deposit_count()
print(count)
```

## Implementation detail

SDK decodes bytes `[64:72]` as little-endian `uint64`.

## See Also

- [get_deposit_root](get-deposit-root.md) — Read the deposit Merkle root
- [deposit](deposit.md) — Submit a validator deposit
