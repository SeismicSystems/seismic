---
description: Seismic-specific transaction fields
icon: file-code
---

# SeismicElements

`SeismicElements` contains the Seismic extension fields included in every `TxSeismic` transaction.

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

## Field Meaning

- `encryption_pubkey`: compressed secp256k1 pubkey used for ECDH
- `encryption_nonce`: 12-byte AES-GCM nonce
- `message_version`: signing mode (`0` raw hash, `2` EIP-712)
- `recent_block_hash`: freshness anchor block hash
- `expires_at_block`: block height cutoff
- `signed_read`: `True` for signed `eth_call`, `False` for transaction send
