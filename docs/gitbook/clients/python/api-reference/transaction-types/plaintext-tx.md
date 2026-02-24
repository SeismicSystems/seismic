---
description: Plaintext transaction view returned by debug writes
icon: file-code
---

# PlaintextTx

Unencrypted transaction view returned by debug shielded writes.

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

## Example

```python
result = contract.dwrite.transfer("0xRecipient", 100)
print(result.plaintext_tx.data.hex())
```
