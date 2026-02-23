---
description: Debug result returned by debug shielded writes
icon: file-code
---

# DebugWriteResult

`DebugWriteResult` packages the debug artifacts returned by debug shielded transaction methods.

## Definition

```python
@dataclass(frozen=True)
class DebugWriteResult:
    plaintext_tx: PlaintextTx
    shielded_tx: UnsignedSeismicTx
    tx_hash: HexBytes
```

## Semantics

- The transaction is broadcast.
- `plaintext_tx` shows unencrypted calldata.
- `shielded_tx` shows encrypted `TxSeismic` fields.
- `tx_hash` is the hash returned by `eth_sendRawTransaction`.
