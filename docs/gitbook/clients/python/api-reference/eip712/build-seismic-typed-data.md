---
description: Build eth_signTypedData_v4 payload for TxSeismic
icon: shield
---

# build_seismic_typed_data

Build a JSON-serializable EIP-712 typed-data payload from an `UnsignedSeismicTx`.

## Signature

```python
def build_seismic_typed_data(tx: UnsignedSeismicTx) -> dict[str, Any]
```

## Returns

Dictionary with keys:

- `types`
- `primaryType` (`"TxSeismic"`)
- `domain`
- `message`

The format matches `eth_signTypedData_v4` expectations.

## Behavior Details

- `message.to` is set to zero address when `tx.to is None`.
- `input`, `encryptionPubkey`, and `recentBlockHash` are emitted as `0x` hex strings.
- `encryptionNonce` is emitted as an integer (`uint96` value).
