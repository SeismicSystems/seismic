---
description: SRC20 interface ABI constant
icon: file-code
---

# SRC20_ABI

The SDK exports the SRC20 ABI constant:

```python
SRC20_ABI: list[dict[str, Any]]
```

## Included interface entries

Functions:

- `name()`
- `symbol()`
- `decimals()`
- `balanceOf()`
- `approve(address,suint256)`
- `transfer(address,suint256)`
- `transferFrom(address,address,suint256)`

Events:

- `Transfer(address,address,bytes32,bytes)`
- `Approval(address,address,bytes32,bytes)`

Notable difference vs ERC20: `balanceOf()` takes no address parameter.
