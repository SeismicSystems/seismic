---
description: Seismic-specific transaction fields
icon: puzzle-piece
---

# SeismicElements

Seismic-specific fields appended to every `TxSeismic` transaction. These carry the encryption parameters and block-expiry metadata.

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

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `encryption_pubkey` | [`CompressedPublicKey`](../types/compressed-public-key.md) | TEE's compressed secp256k1 key for ECDH-derived encryption |
| `encryption_nonce` | [`EncryptionNonce`](../types/encryption-nonce.md) | 12-byte AES-GCM nonce |
| `message_version` | `int` | Signing mode — `0` for raw, `2` for EIP-712 |
| `recent_block_hash` | [`Bytes32`](../types/bytes32.md) | Hash of a recent block (freshness proof) |
| `expires_at_block` | `int` | Block number after which the tx is invalid |
| `signed_read` | `bool` | `True` for signed `eth_call` reads, `False` for writes |

## Example

```python
from seismic_web3 import (
    SeismicElements,
    CompressedPublicKey,
    EncryptionNonce,
    Bytes32,
)
import os

elements = SeismicElements(
    encryption_pubkey=CompressedPublicKey(...),  # from get_tee_public_key()
    encryption_nonce=EncryptionNonce(os.urandom(12)),
    message_version=2,
    recent_block_hash=Bytes32(...),
    expires_at_block=12345678,
    signed_read=False,
)
```

In practice the SDK constructs `SeismicElements` automatically — override specific fields via [`SeismicSecurityParams`](seismic-security-params.md).

## See Also

- [SeismicSecurityParams](seismic-security-params.md) — user-facing overrides for defaults
- [UnsignedSeismicTx](unsigned-seismic-tx.md) — contains SeismicElements
- [TxSeismicMetadata](tx-seismic-metadata.md) — uses SeismicElements for AAD
