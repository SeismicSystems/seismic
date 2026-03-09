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

[`DebugWriteResult`](../../api-reference/transaction-types/debug-write-result.md) with:

| Field | Type | Description |
|-------|------|-------------|
| `tx_hash` | `HexBytes` | Transaction hash from the network |
| `plaintext_tx` | [`PlaintextTx`](../../api-reference/transaction-types/plaintext-tx.md) | Transaction with **unencrypted** calldata |
| `shielded_tx` | [`UnsignedSeismicTx`](../../api-reference/transaction-types/unsigned-seismic-tx.md) | Full `TxSeismic` with **encrypted** calldata |

The transaction **is** broadcast — this is not a dry run.

## Example

```python
result = w3.seismic.debug_send_shielded_transaction(to="0xTarget", data=calldata)
print(result.tx_hash.hex())
print(result.plaintext_tx.data.hex())
print(result.shielded_tx.data.hex())
```

## Notes

- Parameters are identical to [`send_shielded_transaction`](send-shielded-transaction.md)
- For contract interactions, prefer `contract.dwrite.functionName(...)` which handles ABI encoding automatically
- Be careful logging `plaintext_tx` in production — it contains unencrypted calldata

## See Also

- [send_shielded_transaction](send-shielded-transaction.md) — Same pipeline without debug info
- [contract.dwrite](../../contract/namespaces/dwrite.md) — High-level debug write API
- [DebugWriteResult](../../api-reference/transaction-types/debug-write-result.md) — Return type reference
