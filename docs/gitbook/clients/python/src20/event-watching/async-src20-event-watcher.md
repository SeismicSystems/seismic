---
description: Asynchronous SRC20 event watcher class
icon: bolt
---

# AsyncSRC20EventWatcher

Polling-based SRC20 event watcher that runs as an `asyncio.Task`.

Typically created via [`async_watch_src20_events()`](watch-src20-events.md) or [`async_watch_src20_events_with_key()`](../intelligence-providers/watch-src20-events-with-key.md) rather than instantiated directly.

## Constructor

```python
class AsyncSRC20EventWatcher:
    def __init__(
        self,
        w3: AsyncWeb3,
        aes_key: Bytes32,
        *,
        token_address: ChecksumAddress | None = None,
        on_transfer: AsyncTransferCallback | TransferCallback | None = None,
        on_approval: AsyncApprovalCallback | ApprovalCallback | None = None,
        on_error: AsyncErrorCallback | ErrorCallback | None = None,
        poll_interval: float = 2.0,
        from_block: int | str = "latest",
    ) -> None
```

## Parameters

| Parameter | Type | Default | Description |
| --- | --- | --- | --- |
| `w3` | `AsyncWeb3` | | Async Web3 instance |
| `aes_key` | [`Bytes32`](../../api-reference/types/bytes32.md) | | 32-byte AES-256 viewing key |
| `token_address` | `ChecksumAddress \| None` | `None` | SRC20 contract to filter (`None` = all tokens) |
| `on_transfer` | Callback | `None` | Async or sync callback for Transfer events |
| `on_approval` | Callback | `None` | Async or sync callback for Approval events |
| `on_error` | Callback | `None` | Async or sync callback for errors |
| `poll_interval` | `float` | `2.0` | Seconds between polls |
| `from_block` | `int \| "latest"` | `"latest"` | Starting block |

## Methods

| Method | Description |
| --- | --- |
| `await start()` | Start the async polling task |
| `await stop()` | Cancel the task and wait for cleanup |
| `is_running` | Property — `True` if task is active |

## Async Context Manager

```python
async with AsyncSRC20EventWatcher(w3, key, on_transfer=cb) as watcher:
    await asyncio.sleep(60)
# Automatically stopped
```

## Notes

- Callbacks can be sync or async — the watcher detects and awaits coroutines automatically
- `CancelledError` is suppressed during shutdown
- If `on_error` is not provided, errors are logged at DEBUG level

## See Also

- [SRC20EventWatcher](src20-event-watcher.md) — Sync variant
- [watch_src20_events](watch-src20-events.md) — Factory with auto key fetch
- [watch_src20_events_with_key](../intelligence-providers/watch-src20-events-with-key.md) — Factory with explicit key
- [DecryptedTransferLog](../types/decrypted-transfer-log.md) — Transfer callback data
- [DecryptedApprovalLog](../types/decrypted-approval-log.md) — Approval callback data
