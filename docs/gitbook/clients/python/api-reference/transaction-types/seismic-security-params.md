---
description: Optional transaction security overrides
icon: slider
---

# SeismicSecurityParams

Optional overrides for freshness/nonce/expiry fields.

## Definition

```python
@dataclass(frozen=True)
class SeismicSecurityParams:
    blocks_window: int | None = None
    encryption_nonce: EncryptionNonce | None = None
    recent_block_hash: Bytes32 | None = None
    expires_at_block: int | None = None
```

## Defaults when `None`

- `blocks_window`: 100
- `encryption_nonce`: random
- `recent_block_hash`: latest block hash
- `expires_at_block`: computed from latest block + window
