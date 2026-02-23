---
description: Directory contract helpers for SRC20 viewing keys
icon: key
---

# Directory

Directory helpers manage AES viewing keys stored in the genesis Directory contract (`0x1000...0004`).

## Helpers

- [register_viewing_key](register-viewing-key.md)
- [get_viewing_key](get-viewing-key.md)
- [check_has_key / get_key_hash](check-has-key.md)

Async variants are available for each operation.

## Workflow example

```python
import os
from eth_account import Account
from seismic_web3 import Bytes32, PrivateKey, SEISMIC_TESTNET
from seismic_web3.src20 import check_has_key, get_viewing_key, register_viewing_key

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)
address = Account.from_key(bytes(pk)).address

if not check_has_key(w3, address):
    key = Bytes32(os.urandom(32))
    tx_hash = register_viewing_key(w3, w3.seismic.encryption, pk, key)
    w3.eth.wait_for_transaction_receipt(tx_hash)

viewing_key = get_viewing_key(w3, w3.seismic.encryption, pk)
print(viewing_key.hex())
```
