---
description: Perform encrypted signed eth_call reads
icon: search
---

# Signed Reads

Signed reads encrypt calldata and execute `eth_call` with a signed raw transaction payload.

## Recommended path: contract read namespace

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)
raw = token.read.balanceOf()   # returns HexBytes
```

Decode `raw` with ABI tools for your function return type when needed.

## Direct namespace method

If you already have encoded calldata:

```python
from seismic_web3.contract.abi import encode_shielded_calldata

call_data = encode_shielded_calldata(SRC20_ABI, "balanceOf", [])
raw = w3.seismic.signed_call(
    to="0xYourTokenAddress",
    data=call_data,
)
```

## EIP-712 mode

Both contract `.read` and `w3.seismic.signed_call(...)` support `eip712=True`.
