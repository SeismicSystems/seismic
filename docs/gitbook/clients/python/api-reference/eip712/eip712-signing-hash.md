---
description: Compute final EIP-712 signing digest for TxSeismic
icon: fingerprint
---

# eip712_signing_hash

Compute the digest signed in EIP-712 mode.

## Signature

```python
def eip712_signing_hash(tx: UnsignedSeismicTx) -> bytes
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tx` | [`UnsignedSeismicTx`](../transaction-types/unsigned-seismic-tx.md) | Yes | The unsigned Seismic transaction |

## Returns

| Type | Description |
|------|-------------|
| `bytes` | 32-byte keccak256 digest to be ECDSA-signed |

## How it works

```text
keccak256("\x19\x01" ‖ domainSeparator ‖ structHash)
```

Where:
- `\x19\x01` — EIP-712 magic bytes
- `domainSeparator` — from [`domain_separator(chain_id)`](domain-separator.md)
- `structHash` — from [`struct_hash(tx)`](struct-hash.md)

## Example

```python
digest = eip712_signing_hash(unsigned_tx)
print(len(digest))  # 32
```

## See Also

- [sign_seismic_tx_eip712](sign-seismic-tx-eip712.md) — uses this hash to sign
- [domain_separator](domain-separator.md) — computes domain separator
- [struct_hash](struct-hash.md) — computes struct hash
- [build_seismic_typed_data](build-seismic-typed-data.md) — build full typed data
