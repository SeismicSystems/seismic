---
description: End-to-end shielded write with contract namespace
icon: lock
---

# Shielded Write Complete

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

TOKEN = "0xYourTokenAddress"
RECIPIENT = "0xRecipientAddress"

# Create shielded contract wrapper
contract = w3.seismic.contract(TOKEN, SRC20_ABI)

# Send shielded transfer (calldata encrypted by SDK)
tx_hash = contract.write.transfer(RECIPIENT, 1_000)
print("tx:", tx_hash.hex())

receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
print("status:", receipt.status)
```

Debug variant:

```python
result = contract.dwrite.transfer(RECIPIENT, 1_000)
print(result.plaintext_tx)
print(result.shielded_tx)
```
