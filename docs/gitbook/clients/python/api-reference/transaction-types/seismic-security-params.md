---
description: Optional transaction security overrides
icon: slider
---

# SeismicSecurityParams

Optional overrides for metadata/security fields when sending shielded transactions or signed calls.

## Definition

```python
@dataclass(frozen=True)
class SeismicSecurityParams:
    blocks_window: int | None = None
    encryption_nonce: EncryptionNonce | None = None
    recent_block_hash: Bytes32 | None = None
    expires_at_block: int | None = None
```

## Default Behavior

When a field is `None`, SDK defaults are used:

- `blocks_window`: defaults to 100 blocks
- `encryption_nonce`: random nonce
- `recent_block_hash`: fetched from latest block
- `expires_at_block`: computed from latest block and window
