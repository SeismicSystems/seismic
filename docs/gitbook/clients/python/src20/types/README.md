---
description: Data types for SRC20 event watching
icon: database
---

# Types

Frozen dataclasses for decoded SRC20 events and callback type aliases.

## Event Data Types

| Type | Description |
| --- | --- |
| [DecryptedTransferLog](decrypted-transfer-log.md) | Transfer event with decrypted amount |
| [DecryptedApprovalLog](decrypted-approval-log.md) | Approval event with decrypted amount |

## Callback Type Aliases

```python
# Sync
TransferCallback = Callable[[DecryptedTransferLog], None]
ApprovalCallback = Callable[[DecryptedApprovalLog], None]
ErrorCallback = Callable[[Exception], None]

# Async
AsyncTransferCallback = Callable[[DecryptedTransferLog], Awaitable[None]]
AsyncApprovalCallback = Callable[[DecryptedApprovalLog], Awaitable[None]]
AsyncErrorCallback = Callable[[Exception], Awaitable[None]]
```

## Common Fields

Both event types share these fields:

| Field | Type | Description |
| --- | --- | --- |
| `encrypt_key_hash` | `bytes` | 32-byte `keccak256` of the viewing key (event topic) |
| `encrypted_amount` | `bytes` | Raw AES-GCM ciphertext with auth tag |
| `decrypted_amount` | `int` | Decrypted token amount (raw — consider decimals) |
| `transaction_hash` | `HexBytes` | Transaction containing the event |
| `block_number` | `int` | Block containing the event |

## See Also

- [Event Watching](../event-watching/) — Watch and decrypt SRC20 events
- [Directory](../intelligence-providers/directory/) — Manage viewing keys
