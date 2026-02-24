---
description: Directory ABI function signatures
icon: file-code
---

# Directory

## Constants

```python
DIRECTORY_ADDRESS: str = "0x1000000000000000000000000000000000000004"
DIRECTORY_ABI: list[dict[str, Any]]
```

## Functions

| Function | ABI Signature | Returns |
| --- | --- | --- |
| `checkHasKey` | `checkHasKey(_addr: address)` | `bool` |
| `keyHash` | `keyHash(to: address)` | `bytes32` |
| `getKey` | `getKey()` | `uint256` |
| `setKey` | `setKey(_key: suint256)` | `None` |
