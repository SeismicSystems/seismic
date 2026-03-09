---
description: Sign and serialize TxSeismic using EIP-712 hashing
icon: pen-to-square
---

# sign_seismic_tx_eip712

Sign and serialize a Seismic transaction using EIP-712 typed data hashing. This is the primary signing function for `message_version == 2` transactions.

## Signature

```python
def sign_seismic_tx_eip712(
    tx: UnsignedSeismicTx,
    private_key: PrivateKey,
) -> HexBytes
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tx` | [`UnsignedSeismicTx`](../transaction-types/unsigned-seismic-tx.md) | Yes | The unsigned Seismic transaction (should have `message_version == 2`) |
| `private_key` | [`PrivateKey`](../types/private-key.md) | Yes | 32-byte secp256k1 private key |

## Returns

| Type | Description |
|------|-------------|
| `HexBytes` | Full signed transaction bytes (`0x4a` prefix + RLP) ready for `eth_sendRawTransaction` |

## Steps

1. Compute [`eip712_signing_hash(tx)`](eip712-signing-hash.md)
2. Sign with `eth_keys.PrivateKey.sign_msg_hash()`
3. Serialize with `serialize_signed(tx, sig)` — same RLP as raw signing

The RLP serialization is identical to raw signing mode; only the ECDSA message hash differs. The Seismic node checks `message_version` to determine which verification path to use.

## EIP-712 vs raw signing

| Aspect | EIP-712 (`message_version=2`) | Raw (`message_version=0`) |
|--------|-------------------------------|---------------------------|
| **Signing hash** | Structured EIP-712 hash | RLP hash of unsigned tx |
| **Wallet support** | Better UX (structured display) | Generic message signing |
| **RLP output** | Identical | Identical |
| **Verification** | Node uses EIP-712 path | Node uses raw path |

## Example

```python
from seismic_web3 import sign_seismic_tx_eip712

signed_tx = sign_seismic_tx_eip712(unsigned_tx, private_key)
tx_hash = w3.eth.send_raw_transaction(signed_tx)
```

## Warnings

- The function does not enforce `message_version == 2`; callers should set it appropriately
- Ensure `expires_at_block` hasn't passed before broadcasting

## See Also

- [eip712_signing_hash](eip712-signing-hash.md) — computes the signing hash
- [build_seismic_typed_data](build-seismic-typed-data.md) — build typed data dict for external signers
- [UnsignedSeismicTx](../transaction-types/unsigned-seismic-tx.md) — input transaction type
- [PrivateKey](../types/private-key.md) — signing key type
