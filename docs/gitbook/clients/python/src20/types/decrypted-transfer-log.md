---
description: Decoded SRC20 Transfer event
icon: file-code
---

# DecryptedTransferLog

Dataclass emitted for decrypted SRC20 `Transfer` logs.

## Definition

```python
@dataclass(frozen=True)
class DecryptedTransferLog:
    from_address: ChecksumAddress
    to_address: ChecksumAddress
    encrypt_key_hash: bytes
    encrypted_amount: bytes
    decrypted_amount: int
    transaction_hash: HexBytes
    block_number: int
```
