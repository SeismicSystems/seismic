---
description: Core Python SDK types and EIP-712 helpers
icon: code
---

# API Reference

Low-level types and helpers exported by `seismic_web3`.

## Sections

- [Types](types/bytes32.md)
- [Transaction Types](transaction-types/signature.md)
- [EIP-712](eip712/sign-seismic-tx-eip712.md)

## Quick example

```python
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str("0x...")
w3 = SEISMIC_TESTNET.wallet_client(pk)
token = w3.seismic.contract("0xTokenAddress", SRC20_ABI)
```
