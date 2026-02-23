---
description: Decoded SRC20 event log dataclasses
icon: file-code
---

# Types

Watchers emit these dataclasses:

- [DecryptedTransferLog](decrypted-transfer-log.md)
- [DecryptedApprovalLog](decrypted-approval-log.md)

Both include:

- `encrypt_key_hash`
- `encrypted_amount`
- `decrypted_amount`
- `transaction_hash`
- `block_number`

## Example callback

```python
def on_transfer(log):
    print(log.from_address, log.to_address, log.decrypted_amount)
```
