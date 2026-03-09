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

- Calls custom RPC method `seismic_getTeePublicKey`
- Validates return value as `CompressedPublicKey` (33 bytes, prefix `0x02` or `0x03`)

## Notes

- Wallet clients call this automatically during construction to derive the ECDH shared key for calldata encryption — you don't need to call it manually unless implementing custom encryption
- The TEE's key is ephemeral and regenerated on node restart

## See Also

- [CompressedPublicKey](../../api-reference/types/compressed-public-key.md) — Return type
- [send_shielded_transaction](send-shielded-transaction.md) — Uses the TEE key for encryption
