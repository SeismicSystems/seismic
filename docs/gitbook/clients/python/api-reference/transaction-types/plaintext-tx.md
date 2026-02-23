---
description: Plaintext transaction view returned by debug writes
icon: file-code
---

# PlaintextTx

`PlaintextTx` is the unencrypted transaction view returned by debug shielded write methods.

## Definition

```python
@dataclass(frozen=True)
class PlaintextTx:
    to: ChecksumAddress | None
    data: HexBytes
    nonce: int
    gas: int
    gas_price: int
    value: int
```

## Usage

You get this from `DebugWriteResult.plaintext_tx` when using debug write APIs.
