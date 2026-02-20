---
icon: download
---

# Installation

## Requirements

- Python 3.10+

## Install

```bash
pip install seismic-web3
```

or with [uv](https://github.com/astral-sh/uv):

```bash
uv add seismic-web3
```

## Verify

```python
from seismic_web3 import create_wallet_client
from seismic_web3.chains import SEISMIC_TESTNET
from eth_account import Account

w3 = create_wallet_client(
    chain=SEISMIC_TESTNET,
    private_key=Account.from_key("0x...").key,
)

print(f"Connected to chain ID: {w3.eth.chain_id}")
print(f"Block number: {w3.eth.block_number}")
print(f"TEE public key: {w3.seismic.get_tee_public_key().hex()}")
```
