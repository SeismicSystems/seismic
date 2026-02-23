---
description: Seismic-specific transaction fields
icon: file-code
---

# SeismicElements

Seismic extension fields included in every `TxSeismic`.

## Definition

```python
@dataclass(frozen=True)
class SeismicElements:
    encryption_pubkey: CompressedPublicKey
    encryption_nonce: EncryptionNonce
    message_version: int
    recent_block_hash: Bytes32
    expires_at_block: int
    signed_read: bool
```

## Key fields

- `message_version`: signing mode (`0` raw, `2` EIP-712)
- `signed_read`: `True` for signed read calls
