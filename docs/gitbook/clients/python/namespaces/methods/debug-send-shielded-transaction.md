---
description: Send shielded transaction and return debug artifacts
icon: bug
---

# debug_send_shielded_transaction

Same send pipeline as `send_shielded_transaction`, plus plaintext/encrypted transaction views.

## Signatures

```python
# sync
w3.seismic.debug_send_shielded_transaction(...same args as send_shielded_transaction...) -> DebugWriteResult

# async
await w3.seismic.debug_send_shielded_transaction(...same args...) -> DebugWriteResult
```

## Returns

`DebugWriteResult` with:

- `plaintext_tx`
- `shielded_tx`
- `tx_hash`

The transaction is still broadcast.

## Example

```python
result = w3.seismic.debug_send_shielded_transaction(to="0xTarget", data=calldata)
print(result.tx_hash.hex())
print(result.plaintext_tx.data.hex())
print(result.shielded_tx.data.hex())
```
