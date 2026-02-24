---
description: Standard EVM fields included in Seismic metadata
icon: file-code
---

# LegacyFields

Standard EVM fields embedded in metadata/AAD.

## Definition

```python
@dataclass(frozen=True)
class LegacyFields:
    chain_id: int
    nonce: int
    to: ChecksumAddress | None
    value: int
```

## Example

```python
from seismic_web3.transaction_types import LegacyFields

legacy = LegacyFields(chain_id=5124, nonce=1, to="0x0000000000000000000000000000000000000000", value=0)
```
