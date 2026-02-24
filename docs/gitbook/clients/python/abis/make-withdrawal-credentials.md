---
description: Build 32-byte ETH1 withdrawal credentials from an address
icon: key
---

# make\_withdrawal\_credentials

Build 32-byte ETH1 withdrawal credentials from an Ethereum address. The returned value is passed as `withdrawal_credentials` when calling [`compute_deposit_data_root`](compute-deposit-data-root.md) and the deposit contract's `deposit()` function.

## Signature

```python
def make_withdrawal_credentials(address: str) -> bytes
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `address` | `str` | Hex-encoded Ethereum address (with or without `0x` prefix) |

## Returns

`bytes` — 32-byte withdrawal credentials.

Raises `ValueError` if the decoded address is not exactly 20 bytes.

## Output Format

```
┌──────┬──────────────┬──────────────────────┐
│ 0x01 │ 11 zero bytes│ 20-byte address      │
└──────┴──────────────┴──────────────────────┘
 1 byte  11 bytes       20 bytes = 32 total
```

- **`0x01`** — ETH1-style withdrawal credential type byte
- **11 zero bytes** — reserved padding
- **20-byte address** — the Ethereum address that can withdraw funds

## Example

```python
from seismic_web3 import make_withdrawal_credentials

credentials = make_withdrawal_credentials(
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
)

assert len(credentials) == 32
assert credentials[0] == 0x01
assert credentials[1:12] == b"\x00" * 11
print(credentials.hex())
```

## Notes

- Accepts addresses with or without `0x`/`0X` prefix
- Case-insensitive — both checksummed and lowercase addresses produce the same result
- This function only creates ETH1 credentials (type `0x01`), not BLS credentials (type `0x00`)

## See Also

- [compute\_deposit\_data\_root](compute-deposit-data-root.md) — Uses `withdrawal_credentials` as input
- [Deposit Contract](deposit-contract.md) — ABI, address, and deposit requirements
