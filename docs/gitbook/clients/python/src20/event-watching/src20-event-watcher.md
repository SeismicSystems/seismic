---
description: Synchronous SRC20 event watcher class
icon: clock
---

# SRC20EventWatcher

Polling-based SRC20 event watcher that runs in a background daemon thread.

Typically created via [`watch_src20_events()`](watch-src20-events.md) or [`watch_src20_events_with_key()`](../intelligence-providers/watch-src20-events-with-key.md) rather than instantiated directly.

## Constructor

```python
class SRC20EventWatcher:
    def __init__(
        self,
        w3: Web3,
        aes_key: Bytes32,
        *,
        token_address: ChecksumAddress | None = None,
        on_transfer: TransferCallback | None = None,
        on_approval: ApprovalCallback | None = None,
        on_error: ErrorCallback | None = None,
        poll_interval: float = 2.0,
        from_block: int | str = "latest",
    ) -> None
```

## Parameters

| Parameter | Type | Default | Description |
| --- | --- | --- | --- |
| `w3` | `Web3` | | Web3 instance |
| `aes_key` | [`Bytes32`](../../api-reference/types/bytes32.md) | | 32-byte AES-256 viewing key |
| `token_address` | `ChecksumAddress \| None` | `None` | SRC20 contract to filter (`None` = all tokens) |
| `on_transfer` | `TransferCallback \| None` | `None` | Callback for Transfer events |
| `on_approval` | `ApprovalCallback \| None` | `None` | Callback for Approval events |
| `on_error` | `ErrorCallback \| None` | `None` | Callback for errors |
| `poll_interval` | `float` | `2.0` | Seconds between polls |
| `from_block` | `int \| "latest"` | `"latest"` | Starting block |

## Methods

| Method | Description |
| --- | --- |
| `start()` | Start the background polling thread |
| `stop()` | Signal stop and wait for thread exit |
| `is_running` | Property — `True` if thread is alive |

## Context Manager

```python
with watch_src20_events_with_key(w3, viewing_key=key, on_transfer=cb) as watcher:
    time.sleep(60)
# Automatically stopped
```

## Notes

- Runs as a daemon thread named `"src20-watcher"` — exits when main thread exits
- Block tracking is automatic; the watcher remembers the last processed block
- If `on_error` is not provided, errors are logged at DEBUG level
- Computes `keccak256(aes_key)` once at initialization for event filtering

## See Also

- [AsyncSRC20EventWatcher](async-src20-event-watcher.md) — Async variant
- [watch_src20_events](watch-src20-events.md) — Factory with auto key fetch
- [watch_src20_events_with_key](../intelligence-providers/watch-src20-events-with-key.md) — Factory with explicit key
- [DecryptedTransferLog](../types/decrypted-transfer-log.md) — Transfer callback data
- [DecryptedApprovalLog](../types/decrypted-approval-log.md) — Approval callback data
