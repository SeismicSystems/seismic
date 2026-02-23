---
description: Sync SRC20 polling watcher class
icon: activity
---

# SRC20EventWatcher

Background-thread polling watcher for decrypted SRC20 events.

## Constructor

```python
SRC20EventWatcher(
    w3: Web3,
    aes_key: Bytes32,
    *,
    token_address: ChecksumAddress | None = None,
    on_transfer: TransferCallback | None = None,
    on_approval: ApprovalCallback | None = None,
    on_error: ErrorCallback | None = None,
    poll_interval: float = 2.0,
    from_block: int | str = "latest",
)
```

## Lifecycle API

```python
watcher.start()
watcher.stop()
watcher.is_running
```

Also supports context manager usage:

```python
with watcher:
    ...
```

## Example

```python
watcher = SRC20EventWatcher(
    w3,
    viewing_key,
    token_address="0xTokenAddress",
    on_transfer=lambda log: print(log.decrypted_amount),
)
watcher.start()
# ...
watcher.stop()
```
