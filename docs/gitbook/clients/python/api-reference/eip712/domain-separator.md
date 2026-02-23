---
description: Compute Seismic EIP-712 domain separator
icon: shield
---

# domain_separator

Compute the EIP-712 domain separator used for Seismic transaction signing.

## Signature

```python
def domain_separator(chain_id: int) -> bytes
```

## Domain Fields

The SDK uses:

- `name = "Seismic Transaction"`
- `version = str(TYPED_DATA_MESSAGE_VERSION)` (currently `"2"`)
- `verifyingContract = 0x0000000000000000000000000000000000000000`
- `chainId = chain_id` argument

## Returns

32-byte `keccak256` domain separator.
