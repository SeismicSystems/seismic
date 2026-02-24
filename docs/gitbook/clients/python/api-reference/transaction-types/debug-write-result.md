---
description: Debug result returned by debug shielded writes
icon: file-code
---

# DebugWriteResult

Debug artifacts returned by debug shielded transaction methods.

## Definition

```python
@dataclass(frozen=True)
class DebugWriteResult:
    plaintext_tx: PlaintextTx
    shielded_tx: UnsignedSeismicTx
    tx_hash: HexBytes
```

## Example

```python
result = contract.dwrite.transfer("0xRecipient", 100)
print(result.tx_hash.hex())
```
