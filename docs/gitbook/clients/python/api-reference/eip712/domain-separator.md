---
description: Compute Seismic EIP-712 domain separator
icon: shield
---

# domain_separator

Compute the EIP-712 domain separator for Seismic transactions.

## Signature

```python
def domain_separator(chain_id: int) -> bytes
```

## Domain constants

- `name = "Seismic Transaction"`
- `version = str(TYPED_DATA_MESSAGE_VERSION)` (currently `"2"`)
- `verifyingContract = 0x0000000000000000000000000000000000000000`

## Example

```python
from seismic_web3 import domain_separator

d = domain_separator(5124)
print(d.hex())
```
