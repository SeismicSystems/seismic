---
description: Encrypted eth_call that proves your identity to the contract
icon: signature
---

# Signed Reads

A signed read (`.read`) builds a full `TxSeismic` just like a [shielded write](shielded-write.md), but targets the `eth_call` endpoint instead of broadcasting a transaction. The node decrypts the calldata inside the TEE, executes the call, encrypts the result, and returns it.

## Why this matters

Any contract function that depends on `msg.sender` needs a signed read. A plain `eth_call` zeros out the `from` field, so the contract wouldn't know who's asking.

```python
# This proves your identity to the contract
result = token.read.balanceOf()

# This does NOT — msg.sender will be 0x0
result = token.tread.balanceOf()
```

A common example: SRC20's `balanceOf()` takes no arguments and uses `msg.sender` internally. If you call it with `.tread`, the contract sees the zero address as the sender and returns its balance — which is almost certainly zero.

## What gets encrypted

Both the calldata you send and the result you get back are encrypted. An observer watching the network can see that you made a call to a particular contract address, but not what function you called or what was returned.

## Basic example

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

## Low-level API

For pre-encoded calldata or non-ABI interactions:

```python
from eth_abi import decode
from seismic_web3.contract.abi import encode_shielded_calldata

data = encode_shielded_calldata(SRC20_ABI, "balanceOf", [])
raw = w3.seismic.signed_call(
    to="0xYourTokenAddress",
    data=data,
    gas=30000000,
)

balance = decode(["uint256"], bytes(raw))[0]
```

## Async variant

```python
w3 = await SEISMIC_TESTNET.async_wallet_client(pk)
token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)

raw = await token.read.balanceOf()
balance = decode(["uint256"], bytes(raw))[0]
```

## See Also

- [ShieldedContract](../contract/shielded-contract.md) — `.read`, `.tread` namespace reference
- [Shielded Write](shielded-write.md) — Same encryption flow, but broadcasts a transaction
