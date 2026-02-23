---
description: Signed read with ABI decoding
icon: search
---

# Signed Read Pattern

Use signed reads when the contract uses caller identity (`msg.sender`) in read logic.

```python
import os
from eth_abi import decode
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

TOKEN = "0xYourTokenAddress"
token = w3.seismic.contract(TOKEN, SRC20_ABI)

# Signed read (encrypted eth_call)
raw_signed = token.read.balanceOf()
balance_signed = decode(["uint256"], bytes(raw_signed))[0]
print("signed balance", balance_signed)
```

## Using the low-level namespace method

```python
from seismic_web3.contract.abi import encode_shielded_calldata

call_data = encode_shielded_calldata(SRC20_ABI, "balanceOf", [])
raw = w3.seismic.signed_call(to=TOKEN, data=call_data)
balance = decode(["uint256"], bytes(raw))[0]
```

## Transparent comparison

```python
raw_tread = token.tread.balanceOf()
balance_tread = decode(["uint256"], bytes(raw_tread))[0]
print("transparent balance", balance_tread)
```
