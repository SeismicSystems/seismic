---
description: Signed read using contract read namespace
icon: search
---

# Signed Read Pattern

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

TOKEN = "0xYourTokenAddress"
contract = w3.seismic.contract(TOKEN, SRC20_ABI)

# Signed encrypted read
raw = contract.read.balanceOf()
print(raw.hex())
```

Lower-level form with pre-encoded calldata:

```python
from seismic_web3.contract.abi import encode_shielded_calldata

data = encode_shielded_calldata(SRC20_ABI, "balanceOf", [])
raw = w3.seismic.signed_call(to=TOKEN, data=data)
```
