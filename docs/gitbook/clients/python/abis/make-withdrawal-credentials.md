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

## Format

Returns `32` bytes: `0x01 || 11 zero bytes || 20-byte address`.

## Input rules

- Accepts address with or without `0x`/`0X` prefix.
- Raises `ValueError` if decoded address is not exactly 20 bytes.
