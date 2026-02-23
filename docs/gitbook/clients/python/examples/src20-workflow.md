---
description: Register viewing key, read balances, write transfers, watch events
icon: coins
---

# SRC20 Workflow

```python
import os
from eth_abi import decode
from eth_account import Account
from seismic_web3 import Bytes32, PrivateKey, SEISMIC_TESTNET, SRC20_ABI
from seismic_web3.src20 import (
    check_has_key,
    get_viewing_key,
    register_viewing_key,
    watch_src20_events_with_key,
)

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)
address = Account.from_key(bytes(pk)).address

# 1) ensure viewing key is registered
if not check_has_key(w3, address):
    key = Bytes32(os.urandom(32))
    tx_hash = register_viewing_key(w3, w3.seismic.encryption, pk, key)
    w3.eth.wait_for_transaction_receipt(tx_hash)

viewing_key = get_viewing_key(w3, w3.seismic.encryption, pk)

# 2) read metadata (transparent)
name = decode(["string"], bytes(token.tread.name()))[0]
symbol = decode(["string"], bytes(token.tread.symbol()))[0]
decimals = decode(["uint8"], bytes(token.tread.decimals()))[0]
print(name, symbol, decimals)

# 3) read own balance (signed read)
balance_raw = token.read.balanceOf()
balance = decode(["uint256"], bytes(balance_raw))[0]
print("balance", balance)

# 4) write operations
tx1 = token.write.approve("0xSpenderAddress", 500)
tx2 = token.write.transfer("0xRecipientAddress", 100)

w3.eth.wait_for_transaction_receipt(tx1)
w3.eth.wait_for_transaction_receipt(tx2)

# 5) watch decrypted events
watcher = watch_src20_events_with_key(
    w3,
    viewing_key=viewing_key,
    token_address="0xYourTokenAddress",
    on_transfer=lambda log: print("transfer", log.decrypted_amount),
    on_approval=lambda log: print("approval", log.decrypted_amount),
)

# ... later: watcher.stop()
```
