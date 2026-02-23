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

## Example

```python
tee_key = w3.seismic.get_tee_public_key()
print(tee_key.hex())
```

## Behavior

- Calls custom RPC method `seismic_getTeePublicKey`.
- Validates return value as `CompressedPublicKey` (33 bytes, prefix `0x02` or `0x03`).
