---
description: Sign and serialize TxSeismic using EIP-712 hashing
icon: shield
---

# sign_seismic_tx_eip712

Sign and serialize a Seismic transaction using EIP-712 hashing.

## Signature

```python
def sign_seismic_tx_eip712(tx: UnsignedSeismicTx, private_key: PrivateKey) -> HexBytes
```

## Steps

1. Compute `eip712_signing_hash(tx)`
2. Sign with `eth_keys.PrivateKey.sign_msg_hash()`
3. Serialize with `serialize_signed(tx, sig)`

## Returns

Signed transaction bytes ready for `eth_sendRawTransaction`.

## Important

- The function does not enforce `tx.seismic.message_version == 2`.
- Callers should set `message_version=2` when using EIP-712 verification path.
