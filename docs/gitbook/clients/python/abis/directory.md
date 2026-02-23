---
description: Directory contract address and ABI constant
icon: file-code
---

# Directory

## Constants

```python
DIRECTORY_ADDRESS: str = "0x1000000000000000000000000000000000000004"
DIRECTORY_ABI: list[dict[str, Any]]
```

## ABI entries

- `checkHasKey(address) -> bool`
- `keyHash(address) -> bytes32`
- `getKey() -> uint256`
- `setKey(suint256)`

## Helper usage example

```python
from seismic_web3.src20 import check_has_key, get_key_hash

has_key = check_has_key(w3, "0xAddress")
key_hash = get_key_hash(w3, "0xAddress")
```

`getKey` and `setKey` are typically used via:

- `get_viewing_key(...)`
- `register_viewing_key(...)`
