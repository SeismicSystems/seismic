---
description: End-to-end shielded write with verification
icon: lock
---

# Shielded Write Complete

This example writes encrypted calldata, waits for confirmation, and verifies with a signed read.

```python
import os
from eth_abi import decode
from seismic_web3 import PrivateKey, SEISMIC_TESTNET

ABI = [
    {
        "name": "setNumber",
        "type": "function",
        "inputs": [{"name": "value", "type": "uint256"}],
        "outputs": [],
    },
    {
        "name": "getNumber",
        "type": "function",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
    },
]

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

contract = w3.seismic.contract("0xYourContractAddress", ABI)

# 1. write
next_value = 42
tx_hash = contract.write.setNumber(next_value)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
assert receipt.status == 1

# 2. verify with signed read
raw = contract.read.getNumber()
value = decode(["uint256"], bytes(raw))[0]
assert value == next_value
print("ok", value)
```

## Debug variant

```python
result = contract.dwrite.setNumber(43)
print(result.tx_hash.hex())
print(result.plaintext_tx.data.hex())
print(result.shielded_tx.data.hex())
```

## Low-level variant

```python
from seismic_web3.contract.abi import encode_shielded_calldata

data = encode_shielded_calldata(ABI, "setNumber", [44])
tx_hash = w3.seismic.send_shielded_transaction(to="0xYourContractAddress", data=data)
```
