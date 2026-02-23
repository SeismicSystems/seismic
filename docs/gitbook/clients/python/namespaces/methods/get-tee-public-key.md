---
description: Fetch the node TEE compressed public key
icon: key
---

# get_tee_public_key

Fetch the TEE compressed secp256k1 public key from the connected Seismic node.

## Signatures

```python
# sync
w3.seismic.get_tee_public_key() -> CompressedPublicKey

# async
await w3.seismic.get_tee_public_key() -> CompressedPublicKey
```

## Behavior

This delegates to the `seismic_getTeePublicKey` RPC method and validates the returned key as `CompressedPublicKey` (33 bytes, `0x02`/`0x03` prefix).
