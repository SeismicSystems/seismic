---
description: ECDSA signature dataclass used for serialized TxSeismic
icon: file-code
---

# Signature

`Signature` stores ECDSA components.

```python
@dataclass(frozen=True)
class Signature:
    v: int
    r: int
    s: int
```

## Example

```python
from seismic_web3 import Signature

sig = Signature(v=1, r=123, s=456)
```
