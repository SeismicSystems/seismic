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

## ABI contents

`DIRECTORY_ABI` includes:

- `checkHasKey(address) -> bool` (`view`)
- `keyHash(address) -> bytes32` (`view`)
- `getKey() -> uint256` (`view`)
- `setKey(suint256)` (`nonpayable`)

This ABI is used by SRC20 viewing-key helper functions.
