---
description: Data types for SRC20 event watching
icon: database
---

# Types

Data types for decoded SRC20 events and callbacks.

## Overview

The types module provides frozen dataclasses for decoded SRC20 events with decrypted amounts, as well as type aliases for callbacks.

## Event Data Types

| Type | Description |
|------|-------------|
| [DecryptedTransferLog](decrypted-transfer-log.md) | Transfer event with decrypted amount |
| [DecryptedApprovalLog](decrypted-approval-log.md) | Approval event with decrypted amount |

## Quick Reference

### DecryptedTransferLog

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

### DecryptedApprovalLog

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

## Callback Type Aliases

### Sync Callbacks

```python
TransferCallback = Callable[[DecryptedTransferLog], None]
ApprovalCallback = Callable[[DecryptedApprovalLog], None]
ErrorCallback = Callable[[Exception], None]
```

### Async Callbacks

```python
AsyncTransferCallback = Callable[[DecryptedTransferLog], Awaitable[None]]
AsyncApprovalCallback = Callable[[DecryptedApprovalLog], Awaitable[None]]
AsyncErrorCallback = Callable[[Exception], Awaitable[None]]
```

## Example Usage

### Transfer Event

```python
from seismic_web3.src20 import watch_src20_events, DecryptedTransferLog

def on_transfer(log: DecryptedTransferLog):
    print(f"Transfer from {log.from_address} to {log.to_address}")
    print(f"Amount: {log.decrypted_amount}")
    print(f"Block: {log.block_number}")

watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    on_transfer=on_transfer,
)
```

### Approval Event

```python
from seismic_web3.src20 import watch_src20_events, DecryptedApprovalLog

def on_approval(log: DecryptedApprovalLog):
    print(f"Approval from {log.owner} to {log.spender}")
    print(f"Amount: {log.decrypted_amount}")
    print(f"Block: {log.block_number}")

watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    on_approval=on_approval,
)
```

### Both Events

```python
from seismic_web3.src20 import (
    watch_src20_events,
    DecryptedTransferLog,
    DecryptedApprovalLog,
)

def on_transfer(log: DecryptedTransferLog):
    print(f"Transfer: {log.decrypted_amount}")

def on_approval(log: DecryptedApprovalLog):
    print(f"Approval: {log.decrypted_amount}")

watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    on_transfer=on_transfer,
    on_approval=on_approval,
)
```

## Common Fields

Both event types share common fields:

| Field | Type | Description |
|-------|------|-------------|
| `encrypt_key_hash` | `bytes` | 32-byte hash of viewing key (used for filtering) |
| `encrypted_amount` | `bytes` | Raw AES-GCM ciphertext with auth tag |
| `decrypted_amount` | `int` | Decrypted token amount (raw value) |
| `transaction_hash` | `HexBytes` | Transaction hash containing the event |
| `block_number` | `int` | Block number containing the event |

## Properties

### Immutability

All event types are frozen dataclasses:

```python
log.decrypted_amount = 1000  # Raises AttributeError
```

### Type Safety

All fields use proper types:
- Addresses are checksummed (`ChecksumAddress`)
- Transaction hashes are `HexBytes`
- Amounts are Python integers
- Block numbers are integers

## Notes

- All event dataclasses are immutable (frozen)
- Addresses are automatically checksummed
- Amounts are raw values (consider token decimals)
- Encrypted amounts include authentication tags
- Created automatically by event watchers

## See Also

- [Event Watching](../event-watching/README.md) - Watch and decrypt SRC20 events
- [Directory](../directory/README.md) - Manage viewing keys
- [SRC20 Overview](../README.md) - Full SRC20 documentation
