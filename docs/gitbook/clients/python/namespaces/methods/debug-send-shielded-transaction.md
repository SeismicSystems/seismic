---
description: Send shielded transaction and return debug artifacts
icon: bug
---

# debug_send_shielded_transaction

Same send pipeline as `send_shielded_transaction`, but also returns plaintext/encrypted transaction views.

## Signatures

```python
# sync
w3.seismic.debug_send_shielded_transaction(
    *,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
    eip712: bool = False,
) -> DebugWriteResult

# async
await w3.seismic.debug_send_shielded_transaction(...same args...) -> DebugWriteResult
```

## Returns

`DebugWriteResult` with:

- `plaintext_tx`
- `shielded_tx`
- `tx_hash`

The transaction is still broadcast.
