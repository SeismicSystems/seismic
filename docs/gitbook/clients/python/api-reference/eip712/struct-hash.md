---
description: Compute TxSeismic EIP-712 struct hash
icon: shield
---

# struct_hash

Compute the EIP-712 struct hash for an `UnsignedSeismicTx`.

## Signature

```python
def struct_hash(tx: UnsignedSeismicTx) -> bytes
```

## Encoding Notes

- Static fields are 32-byte padded per EIP-712 encoding.
- Dynamic `bytes` fields are hashed first:
  - `tx.data`
  - `tx.seismic.encryption_pubkey`
- `encryption_nonce` is encoded as `uint96` (via int conversion).

## Returns

32-byte `keccak256` struct hash.
