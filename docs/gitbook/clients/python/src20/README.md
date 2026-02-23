---
description: SRC20 viewing keys, event watching, and decoded log types
icon: coins
---

# SRC20

SRC20 helpers in the Python SDK focus on viewing keys and event decryption.

## What this section covers

- Directory key management
- Event watchers (sync + async)
- Typed decrypted log objects

## Quick start

```python
import os
from eth_abi import decode
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)
token = w3.seismic.contract("0xTokenAddress", SRC20_ABI)

raw = token.read.balanceOf()
balance = decode(["uint256"], bytes(raw))[0]
print(balance)
```

## Sections

- [Directory](directory/README.md)
- [Event Watching](event-watching/README.md)
- [Types](types/README.md)
