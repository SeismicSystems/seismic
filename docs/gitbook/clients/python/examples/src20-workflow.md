---
description: Read metadata, check balance, approve, and transfer with SRC20
icon: coins
---

# SRC20 Workflow

```python
import os
from eth_abi import decode
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)

# 1) read token metadata (transparent reads)
name = decode(["string"], bytes(token.tread.name()))[0]
symbol = decode(["string"], bytes(token.tread.symbol()))[0]
decimals = decode(["uint8"], bytes(token.tread.decimals()))[0]
print(name, symbol, decimals)

# 2) read your current balance (signed read)
balance_raw = token.read.balanceOf()
balance = decode(["uint256"], bytes(balance_raw))[0]
print("balance", balance)

# 3) approve and transfer (shielded writes)
approve_tx = token.write.approve("0xSpenderAddress", 500)
transfer_tx = token.write.transfer("0xRecipientAddress", 100)

approve_receipt = w3.eth.wait_for_transaction_receipt(approve_tx)
transfer_receipt = w3.eth.wait_for_transaction_receipt(transfer_tx)

print("approve status", approve_receipt["status"])
print("transfer status", transfer_receipt["status"])
```
