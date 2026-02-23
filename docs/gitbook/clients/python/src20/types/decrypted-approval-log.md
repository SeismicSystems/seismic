---
description: Decoded SRC20 Approval event
icon: file-code
---

# DecryptedApprovalLog

Dataclass emitted for decrypted SRC20 `Approval` logs.

## Definition

```python
@dataclass(frozen=True)
class DecryptedApprovalLog:
    owner: ChecksumAddress
    spender: ChecksumAddress
    encrypt_key_hash: bytes
    encrypted_amount: bytes
    decrypted_amount: int
    transaction_hash: HexBytes
    block_number: int
```
