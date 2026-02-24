---
description: SRC20 token usage with shielded reads/writes
icon: coins
---

# SRC20

This section documents core SRC20 token interactions via contract namespaces.

## Core usage

```python
import os
from eth_abi import decode
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract("0xTokenAddress", SRC20_ABI)

name = decode(["string"], bytes(token.tread.name()))[0]
symbol = decode(["string"], bytes(token.tread.symbol()))[0]
decimals = decode(["uint8"], bytes(token.tread.decimals()))[0]

balance = decode(["uint256"], bytes(token.read.balanceOf()))[0]

tx_hash = token.write.transfer("0xRecipient", 100)
w3.eth.wait_for_transaction_receipt(tx_hash)
```

## Notes

- `balanceOf()` in `SRC20_ABI` takes no address argument.
- Use `.read` for shielded reads and `.write` for shielded writes.
- Use `.tread` for transparent metadata reads.
