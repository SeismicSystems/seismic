---
description: Decoded SRC20 event log dataclasses
icon: file-code
---

# Types

SRC20 watchers emit these dataclasses:

- [DecryptedTransferLog](decrypted-transfer-log.md)
- [DecryptedApprovalLog](decrypted-approval-log.md)

Both include:

- original indexed addresses
- `encrypt_key_hash`
- raw `encrypted_amount`
- decrypted integer amount
- transaction hash and block number
