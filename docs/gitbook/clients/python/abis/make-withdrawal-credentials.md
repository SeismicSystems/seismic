---
description: Build 32-byte ETH1 withdrawal credentials
icon: wrench
---

# make_withdrawal_credentials

Build withdrawal credentials from an Ethereum address.

## Signature

```python
def make_withdrawal_credentials(address: str) -> bytes
```

## Output format

`0x01 || 11 zero bytes || 20-byte address` (32 bytes total).

## Input rules

- Accepts address with or without `0x`/`0X` prefix
- Raises `ValueError` if decoded address is not exactly 20 bytes

## Example

```python
from seismic_web3 import make_withdrawal_credentials

withdrawal_credentials = make_withdrawal_credentials(
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
)
print(len(withdrawal_credentials))  # 32
```
