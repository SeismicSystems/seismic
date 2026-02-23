---
description: Full metadata used as AAD context for encryption
icon: file-code
---

# TxSeismicMetadata

`TxSeismicMetadata` binds sender + legacy fields + Seismic fields into one structure.

## Definition

```python
@dataclass(frozen=True)
class TxSeismicMetadata:
    sender: ChecksumAddress
    legacy_fields: LegacyFields
    seismic_elements: SeismicElements
```

## Why It Exists

This metadata is encoded as AAD for AES-GCM encryption/decryption, so ciphertext is authenticated against full transaction context.
