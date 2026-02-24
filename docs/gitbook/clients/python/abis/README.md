---
description: Built-in ABI constants and deposit helper functions
icon: file-code
---

# ABIs

The SDK exports protocol ABI/constants under `seismic_web3.abis` plus deposit helper utilities.

## Built-in constants

- `SRC20_ABI`
- `DEPOSIT_CONTRACT_ABI`
- `DEPOSIT_CONTRACT_ADDRESS`
- `DIRECTORY_ABI` (internal)
- `DIRECTORY_ADDRESS` (internal)

## Helper functions

- `make_withdrawal_credentials(address: str) -> bytes`
- `compute_deposit_data_root(...) -> bytes`

## Quick usage

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey, SRC20_ABI

w3 = SEISMIC_TESTNET.wallet_client(PrivateKey.from_hex_str("0x..."))
token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)
```

See individual pages for signatures and required byte lengths.
