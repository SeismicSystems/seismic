---
description: Transaction metadata used for AAD context
icon: tags
---

# TxSeismicMetadata

Complete metadata for a Seismic transaction, used as Additional Authenticated Data (AAD) for AES-GCM encryption.

## Overview

`TxSeismicMetadata` contains the full transaction context used to construct the Additional Authenticated Data (AAD) for AES-GCM encryption. This ensures the ciphertext is cryptographically bound to the complete transaction parameters.

## Definition

```python
@dataclass(frozen=True)
class TxSeismicMetadata:
    """Complete metadata for a Seismic transaction.

    Used to construct the Additional Authenticated Data (AAD) for
    AES-GCM encryption, ensuring the ciphertext is bound to the
    full transaction context.
    """
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

## Examples

### Manual Construction

```python
from seismic_web3 import (
    TxSeismicMetadata,
    LegacyFields,
    SeismicElements,
)

metadata = TxSeismicMetadata(
    sender="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    legacy_fields=LegacyFields(
        chain_id=5124,
        nonce=42,
        to="0x1234567890123456789012345678901234567890",
        value=1_000_000_000_000_000_000,  # 1 ETH
    ),
    seismic_elements=SeismicElements(...),
)
```

### Use in Encryption

```python
from seismic_web3.transaction.encrypt import build_metadata

# Metadata is built automatically during encryption
metadata = build_metadata(
    sender=sender_address,
    chain_id=chain_id,
    nonce=nonce,
    to=recipient_address,
    value=value_in_wei,
    seismic_elements=elements,
)

# Used as AAD for AES-GCM
encrypted_data = aes_gcm_encrypt(
    key=encryption_key,
    nonce=nonce,
    plaintext=calldata,
    aad=serialize_metadata(metadata),  # AAD binding
)
```

### Access Individual Components

```python
metadata = TxSeismicMetadata(...)

# Access sender
print(f"From: {metadata.sender}")

# Access legacy fields
print(f"Chain ID: {metadata.legacy_fields.chain_id}")
print(f"Nonce: {metadata.legacy_fields.nonce}")
print(f"To: {metadata.legacy_fields.to}")
print(f"Value: {metadata.legacy_fields.value} wei")

# Access Seismic elements
print(f"Expires at: {metadata.seismic_elements.expires_at_block}")
print(f"Message version: {metadata.seismic_elements.message_version}")
```

## AAD (Additional Authenticated Data)

The metadata is serialized and used as AAD in AES-GCM encryption:

```
AAD = RLP([
    sender,
    chain_id,
    nonce,
    to,
    value,
    encryption_pubkey,
    encryption_nonce,
    message_version,
    recent_block_hash,
    expires_at_block,
    signed_read,
])
```

This ensures:
- **Integrity** - Any modification to transaction parameters is detected
- **Binding** - Ciphertext cannot be replayed with different parameters
- **Authenticity** - TEE verifies the entire transaction context

## Properties

- **Immutable** - Cannot be modified after construction (`frozen=True`)
- **Type-safe** - All fields are validated at construction
- **RLP-serializable** - Can be encoded for use as AAD

## Why AAD Matters

Without AAD, an attacker could:
- Replay encrypted calldata with different `to` addresses
- Change the `value` being transferred
- Modify expiry or freshness parameters

With AAD binding:
- Ciphertext is valid **only** with the exact metadata used during encryption
- Any parameter change invalidates the ciphertext
- TEE rejects tampered transactions

## Notes

- Automatically constructed by the SDK during transaction building
- Used internally — most users won't interact with this type directly
- Critical for Seismic's security model
- Combines data from [`LegacyFields`](legacy-fields.md) and [`SeismicElements`](seismic-elements.md)

## See Also

- [LegacyFields](legacy-fields.md) - Standard EVM fields in metadata
- [SeismicElements](seismic-elements.md) - Seismic-specific fields in metadata
- [UnsignedSeismicTx](unsigned-seismic-tx.md) - Transaction that uses this metadata
- [aes_gcm_encrypt](../../precompiles/aes-gcm-encrypt.md) - Uses AAD parameter
- [aes_gcm_decrypt](../../precompiles/aes-gcm-decrypt.md) - Verifies AAD parameter
