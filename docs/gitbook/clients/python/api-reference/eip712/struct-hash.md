---
description: Compute TxSeismic EIP-712 struct hash
icon: hashtag
---

# struct_hash

Compute EIP-712 struct hash for an `UnsignedSeismicTx`.

## Signature

```python
def struct_hash(tx: UnsignedSeismicTx) -> bytes
```

## Encoding details

- Static fields are padded to 32 bytes.
- Dynamic `bytes` fields are hashed first:
  - `tx.data`
  - `tx.seismic.encryption_pubkey`
- `encryption_nonce` is encoded as `uint96` integer.
