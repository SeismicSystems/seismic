---
description: SRC20 interface ABI constant
icon: file-code
---

# SRC20_ABI

```python
SRC20_ABI: list[dict[str, Any]]
```

## Included functions

- `name()`
- `symbol()`
- `decimals()`
- `balanceOf()`
- `approve(address,suint256)`
- `transfer(address,suint256)`
- `transferFrom(address,address,suint256)`

## Included events

- `Transfer(address,address,bytes32,bytes)`
- `Approval(address,address,bytes32,bytes)`

## Notes

- `balanceOf()` takes no address argument in this interface.
- Amount arguments use shielded types in the ABI (`suint256`).

## Example

```python
from eth_abi import decode
from seismic_web3 import SEISMIC_TESTNET, PrivateKey, SRC20_ABI

pk = PrivateKey.from_hex_str("0x...")
w3 = SEISMIC_TESTNET.wallet_client(pk)
token = w3.seismic.contract("0xTokenAddress", SRC20_ABI)

raw = token.read.balanceOf()
balance = decode(["uint256"], bytes(raw))[0]
```
