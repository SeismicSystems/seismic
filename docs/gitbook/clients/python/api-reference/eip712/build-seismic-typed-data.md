---
description: Build eth_signTypedData_v4 payload for TxSeismic
icon: shield
---

# build_seismic_typed_data

Build a JSON-serializable EIP-712 typed-data payload.

## Signature

```python
def build_seismic_typed_data(tx: UnsignedSeismicTx) -> dict[str, Any]
```

## Returns

Dictionary with `types`, `primaryType`, `domain`, and `message`.

## Behavior notes

- `message.to` becomes zero address when `tx.to is None`
- `input`, `encryptionPubkey`, and `recentBlockHash` are `0x` hex strings
- `encryptionNonce` is emitted as integer

## Example

```python
typed_data = build_seismic_typed_data(unsigned_tx)
print(typed_data["primaryType"])
```
