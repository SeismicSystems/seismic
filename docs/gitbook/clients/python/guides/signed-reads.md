---
description: Perform encrypted signed eth_call reads
icon: search
---

# Signed Reads

Signed reads encrypt calldata and execute `eth_call` using a signed raw transaction payload.

## Why use `.read` instead of `.tread`

Use `.read` when contract logic depends on `msg.sender` (for example SRC20 `balanceOf()` in this SDK ABI). Transparent `.tread` does a plain `eth_call` and does not provide the same identity semantics.

## Preferred flow: `contract.read`

```python
import os
from eth_abi import decode
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)

raw = token.read.balanceOf()
balance = decode(["uint256"], bytes(raw))[0]
print(balance)
```

## Low-level flow: `w3.seismic.signed_call`

```python
from eth_abi import decode
from seismic_web3.contract.abi import encode_shielded_calldata

data = encode_shielded_calldata(SRC20_ABI, "balanceOf", [])
raw = w3.seismic.signed_call(to="0xYourTokenAddress", data=data)

balance = decode(["uint256"], bytes(raw))[0]
```

## Explicit gas/security controls

```python
from seismic_web3 import SeismicSecurityParams

params = SeismicSecurityParams(blocks_window=200)
raw = w3.seismic.signed_call(
    to="0xYourTokenAddress",
    data=data,
    gas=30_000_000,
    security=params,
)
```

## Async variant

```python
w3 = await SEISMIC_TESTNET.async_wallet_client(pk)
token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)

raw = await token.read.balanceOf()
balance = decode(["uint256"], bytes(raw))[0]
```
