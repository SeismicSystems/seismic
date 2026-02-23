---
description: Standard EVM fields included in Seismic metadata
icon: file-code
---

# LegacyFields

`LegacyFields` stores standard EVM fields that are embedded in transaction metadata.

## Definition

```python
@dataclass(frozen=True)
class LegacyFields:
    chain_id: int
    nonce: int
    to: ChecksumAddress | None
    value: int
```

## Notes

- `to` is `None` for contract creation.
- These fields are used when building metadata/AAD for encryption.
