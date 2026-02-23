---
description: Async SRC20 polling watcher class
icon: activity
---

# AsyncSRC20EventWatcher

`asyncio` task-based watcher for decrypted SRC20 events.

## Constructor

```python
AsyncSRC20EventWatcher(
    w3: AsyncWeb3,
    aes_key: Bytes32,
    *,
    token_address: ChecksumAddress | None = None,
    on_transfer: AsyncTransferCallback | TransferCallback | None = None,
    on_approval: AsyncApprovalCallback | ApprovalCallback | None = None,
    on_error: AsyncErrorCallback | ErrorCallback | None = None,
    poll_interval: float = 2.0,
    from_block: int | str = "latest",
)
```

## Lifecycle API

```python
await watcher.start()
await watcher.stop()
watcher.is_running
```

Also supports async context manager usage:

```python
async with watcher:
    ...
```

## Example

```python
watcher = AsyncSRC20EventWatcher(
    w3,
    viewing_key,
    on_transfer=lambda log: print(log.decrypted_amount),
)
await watcher.start()
await asyncio.sleep(10)
await watcher.stop()
```
