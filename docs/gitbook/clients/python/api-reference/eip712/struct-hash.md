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

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tx` | [`UnsignedSeismicTx`](../transaction-types/unsigned-seismic-tx.md) | Yes | The unsigned Seismic transaction |

## Returns

| Type | Description |
|------|-------------|
| `bytes` | 32-byte keccak256 hash of the encoded struct |

## How it works

```text
keccak256(
    TX_SEISMIC_TYPE_HASH
    ‖ encode(chain_id)
    ‖ encode(nonce)
    ‖ encode(gas_price)
    ‖ encode(gas)
    ‖ encode(to)
    ‖ encode(value)
    ‖ keccak256(data)              // dynamic type
    ‖ keccak256(encryption_pubkey) // dynamic type
    ‖ encode(encryption_nonce)
    ‖ encode(message_version)
    ‖ recent_block_hash            // already 32 bytes
    ‖ encode(expires_at_block)
    ‖ encode(signed_read)
)
```

## Encoding details

- Static fields are left-padded to 32 bytes
- Dynamic `bytes` fields (`tx.data`, `tx.seismic.encryption_pubkey`) are hashed with `keccak256` before inclusion
- `encryption_nonce` is converted from 12 bytes to a `uint96` integer via `int.from_bytes(…, "big")`
- `to` becomes 32 zero bytes when `tx.to is None`

## See Also

- [eip712_signing_hash](eip712-signing-hash.md) — uses struct hash
- [domain_separator](domain-separator.md) — the other component of the signing hash
- [build_seismic_typed_data](build-seismic-typed-data.md) — builds full typed data
