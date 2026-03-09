---
description: Transaction metadata used for AAD context
icon: tags
---

# TxSeismicMetadata

Complete metadata for a Seismic transaction, used as Additional Authenticated Data (AAD) for AES-GCM encryption.

## Definition

```python
@dataclass(frozen=True)
class TxSeismicMetadata:
    sender: ChecksumAddress
    legacy_fields: LegacyFields
    seismic_elements: SeismicElements
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `sender` | `ChecksumAddress` | Checksummed sender address |
| `legacy_fields` | [`LegacyFields`](legacy-fields.md) | Standard EVM transaction fields |
| `seismic_elements` | [`SeismicElements`](seismic-elements.md) | Seismic-specific encryption and expiry fields |

## AAD binding

The metadata is RLP-encoded and used as AAD in AES-GCM encryption, ensuring the ciphertext is cryptographically bound to the full transaction context. Any modification to transaction parameters (sender, recipient, value, etc.) invalidates the ciphertext.

## Example

```python
from seismic_web3 import TxSeismicMetadata, LegacyFields, SeismicElements

metadata = TxSeismicMetadata(
    sender="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    legacy_fields=LegacyFields(
        chain_id=5124,
        nonce=42,
        to="0x00000000219ab540356cBB839Cbe05303d7705Fa",
        value=0,
    ),
    seismic_elements=SeismicElements(...),
)
```

## Notes

- Automatically constructed by the SDK — most users won't interact with this type directly
- Combines [`LegacyFields`](legacy-fields.md) and [`SeismicElements`](seismic-elements.md) into one structure

## See Also

- [LegacyFields](legacy-fields.md) — standard EVM fields
- [SeismicElements](seismic-elements.md) — Seismic-specific fields
- [UnsignedSeismicTx](unsigned-seismic-tx.md) — transaction that uses this metadata
