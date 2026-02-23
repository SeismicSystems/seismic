---
description: Register key and watch decrypted SRC20 events
icon: coins
---

# SRC20 Workflow

```python
import os
from seismic_web3 import Bytes32, PrivateKey, SEISMIC_TESTNET
from seismic_web3.src20 import (
    check_has_key,
    get_viewing_key,
    register_viewing_key,
    watch_src20_events_with_key,
)

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

address = w3.eth.account.from_key(bytes(pk)).address

if not check_has_key(w3, address):
    key = Bytes32(os.urandom(32))
    tx_hash = register_viewing_key(
        w3,
        encryption=w3.seismic.encryption,
        private_key=pk,
        key=key,
    )
    w3.eth.wait_for_transaction_receipt(tx_hash)

viewing_key = get_viewing_key(w3, w3.seismic.encryption, pk)

watcher = watch_src20_events_with_key(
    w3,
    viewing_key=viewing_key,
    token_address="0xYourTokenAddress",
    on_transfer=lambda log: print("transfer", log.decrypted_amount),
    on_approval=lambda log: print("approval", log.decrypted_amount),
)

# ... later
# watcher.stop()
```
