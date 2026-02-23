---
description: Full metadata used as AAD context for encryption
icon: file-code
---

# TxSeismicMetadata

Binds sender + legacy fields + seismic fields for AAD.

## Definition

```python
@dataclass(frozen=True)
class TxSeismicMetadata:
    sender: ChecksumAddress
    legacy_fields: LegacyFields
    seismic_elements: SeismicElements
```

## Use

This structure is passed to encryption/decryption routines to bind ciphertext to transaction context.
