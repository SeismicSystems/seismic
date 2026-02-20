---
description: Asynchronous SRC20 event watcher class
icon: bolt
---

# AsyncSRC20EventWatcher

Polling-based SRC20 event watcher that runs as an asyncio task (asynchronous).

## Overview

`AsyncSRC20EventWatcher` is a task-based event watcher that polls for SRC20 Transfer and Approval events, decrypts encrypted amounts using a viewing key, and invokes async callbacks with decoded event data.

The watcher runs as an `asyncio.Task` and polls at a configurable interval. It automatically handles block number tracking, log fetching, decryption, and callback invocation in an async context.

Typically created via [`async_watch_src20_events()`](watch-src20-events.md) or [`async_watch_src20_events_with_key()`](watch-src20-events-with-key.md) rather than instantiated directly.

## Class Definition

```python
class AsyncSRC20EventWatcher:
    """Polling-based SRC20 event watcher (async, runs as an asyncio.Task)."""

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

## Constructor Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `w3` | `AsyncWeb3` | Yes | Async Web3 instance |
| `aes_key` | [`Bytes32`](../../api-reference/types/bytes32.md) | Yes | 32-byte AES-256 viewing key |
| `token_address` | `ChecksumAddress` | No | SRC20 contract to filter. If `None`, watches all tokens |
| `on_transfer` | Callback | No | Async or sync callback invoked for Transfer events |
| `on_approval` | Callback | No | Async or sync callback invoked for Approval events |
| `on_error` | Callback | No | Async or sync callback for errors |
| `poll_interval` | `float` | No | Seconds between polls (default `2.0`) |
| `from_block` | `int` or `"latest"` | No | Starting block number or `"latest"` (default) |

## Methods

### start()

Start the async polling task.

```python
async def start(self) -> None
```

### stop()

Cancel the polling task and wait for it to finish.

```python
async def stop(self) -> None
```

## Properties

### is_running

Check if the watcher is currently running.

```python
@property
def is_running(self) -> bool
```

Returns `True` if the task exists and is not done, `False` otherwise.

## Context Manager Support

The watcher can be used as an async context manager:

```python
async def __aenter__(self) -> AsyncSRC20EventWatcher
async def __aexit__(self, *exc: object) -> None
```

The watcher starts automatically on `__aenter__` and stops on `__aexit__`.

## Examples

### Manual Start/Stop

```python
from web3 import AsyncWeb3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import AsyncSRC20EventWatcher
import asyncio

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

    watcher = AsyncSRC20EventWatcher(
        w3,
        viewing_key,
        on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
    )

    await watcher.start()
    await asyncio.sleep(60)
    await watcher.stop()

asyncio.run(main())
```

### Async Context Manager

```python
from web3 import AsyncWeb3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import AsyncSRC20EventWatcher
import asyncio

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

    async with AsyncSRC20EventWatcher(
        w3,
        viewing_key,
        on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
    ) as watcher:
        await asyncio.sleep(60)
    # Automatically stopped

asyncio.run(main())
```

### Check Running Status

```python
from web3 import AsyncWeb3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import AsyncSRC20EventWatcher
import asyncio

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

    watcher = AsyncSRC20EventWatcher(
        w3,
        viewing_key,
        on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
    )

    print(f"Running: {watcher.is_running}")  # False

    await watcher.start()
    print(f"Running: {watcher.is_running}")  # True

    await watcher.stop()
    print(f"Running: {watcher.is_running}")  # False

asyncio.run(main())
```

### Async Callback

```python
from web3 import AsyncWeb3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import AsyncSRC20EventWatcher
import asyncio

async def on_transfer(log):
    print(f"Transfer: {log.decrypted_amount}")
    # Perform async operations
    await save_to_database(log)
    await send_notification(log)

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

    async with AsyncSRC20EventWatcher(
        w3,
        viewing_key,
        on_transfer=on_transfer,
    ) as watcher:
        await asyncio.sleep(60)

asyncio.run(main())
```

### Sync Callback (Also Works)

```python
from web3 import AsyncWeb3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import AsyncSRC20EventWatcher
import asyncio

# Sync callback is automatically detected and works fine
def on_transfer(log):
    print(f"Transfer: {log.decrypted_amount}")

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

    async with AsyncSRC20EventWatcher(
        w3,
        viewing_key,
        on_transfer=on_transfer,
    ) as watcher:
        await asyncio.sleep(60)

asyncio.run(main())
```

### Filter by Token Address

```python
from web3 import AsyncWeb3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import AsyncSRC20EventWatcher
import asyncio

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))
    token_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

    async with AsyncSRC20EventWatcher(
        w3,
        viewing_key,
        token_address=token_address,
        on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
    ) as watcher:
        await asyncio.sleep(60)

asyncio.run(main())
```

### Custom Poll Interval

```python
from web3 import AsyncWeb3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import AsyncSRC20EventWatcher
import asyncio

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

    # Poll every 5 seconds instead of default 2
    async with AsyncSRC20EventWatcher(
        w3,
        viewing_key,
        poll_interval=5.0,
        on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
    ) as watcher:
        await asyncio.sleep(60)

asyncio.run(main())
```

### Historical Events

```python
from web3 import AsyncWeb3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import AsyncSRC20EventWatcher
import asyncio

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

    # Start watching from block 1000000
    async with AsyncSRC20EventWatcher(
        w3,
        viewing_key,
        from_block=1000000,
        on_transfer=lambda log: print(f"Block {log.block_number}: {log.decrypted_amount}"),
    ) as watcher:
        await asyncio.sleep(60)

asyncio.run(main())
```

### Error Handling

```python
from web3 import AsyncWeb3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import AsyncSRC20EventWatcher
import asyncio

async def on_error(exc: Exception):
    print(f"Error in watcher: {exc}")
    # Log to monitoring system, retry, etc.
    await log_to_monitoring(exc)

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

    async with AsyncSRC20EventWatcher(
        w3,
        viewing_key,
        on_transfer=lambda log: print(f"Amount: {log.decrypted_amount}"),
        on_error=on_error,
    ) as watcher:
        await asyncio.sleep(60)

asyncio.run(main())
```

### Both Transfer and Approval

```python
from web3 import AsyncWeb3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import AsyncSRC20EventWatcher
import asyncio

async def on_transfer(log):
    print(f"Transfer from {log.from_address} to {log.to_address}: {log.decrypted_amount}")
    await save_transfer(log)

async def on_approval(log):
    print(f"Approval from {log.owner} to {log.spender}: {log.decrypted_amount}")
    await save_approval(log)

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

    async with AsyncSRC20EventWatcher(
        w3,
        viewing_key,
        on_transfer=on_transfer,
        on_approval=on_approval,
    ) as watcher:
        await asyncio.sleep(60)

asyncio.run(main())
```

## How It Works

1. **Task creation** - Creates an `asyncio.Task` that runs the polling loop
2. **Polling loop** - In the async task:
   - Fetches latest block number
   - Builds `eth_getLogs` filter with key hash
   - Fetches logs for the block range
   - Decodes and decrypts each log
   - Invokes callbacks with decrypted data (awaits if async)
   - Waits for `poll_interval` seconds with `asyncio.sleep()`
3. **Cleanup** - When stopped, cancels the task and suppresses `CancelledError`

## Callback Signatures

### Transfer Callback

```python
# Async
async def on_transfer(log: DecryptedTransferLog) -> None:
    pass

# Sync (also works)
def on_transfer(log: DecryptedTransferLog) -> None:
    pass
```

The watcher automatically detects if the callback is async (using `asyncio.iscoroutine()`) and awaits it if needed.

See [`DecryptedTransferLog`](../types/decrypted-transfer-log.md) for field details.

### Approval Callback

```python
# Async
async def on_approval(log: DecryptedApprovalLog) -> None:
    pass

# Sync (also works)
def on_approval(log: DecryptedApprovalLog) -> None:
    pass
```

See [`DecryptedApprovalLog`](../types/decrypted-approval-log.md) for field details.

### Error Callback

```python
# Async
async def on_error(exc: Exception) -> None:
    pass

# Sync (also works)
def on_error(exc: Exception) -> None:
    pass
```

Invoked for:
- RPC errors (network issues, rate limiting)
- Decryption failures (wrong key, corrupted data)
- Any exceptions during log processing

## Notes

- The polling task runs in the current event loop
- Block tracking is automatic; watcher remembers the last processed block
- Events are processed in order by block number
- If `on_error` is not provided, errors are logged at DEBUG level
- The watcher computes `keccak256(aes_key)` once at initialization
- Callbacks can be sync or async; the watcher detects and handles both

## Warnings

- **Async context required** - Must be run within an `asyncio` event loop
- **Cleanup required** - Always call `await watcher.stop()` or use async context manager
- **Network failures** - Errors are silently logged unless `on_error` is provided
- **CancelledError** - Properly suppressed during shutdown; don't worry about it
- **Memory usage** - Keep poll interval reasonable to avoid excessive log fetching

## See Also

- [watch_src20_events](watch-src20-events.md) - Factory function with automatic key fetching
- [watch_src20_events_with_key](watch-src20-events-with-key.md) - Factory function with explicit key
- [SRC20EventWatcher](src20-event-watcher.md) - Sync variant
- [DecryptedTransferLog](../types/decrypted-transfer-log.md) - Transfer event data
- [DecryptedApprovalLog](../types/decrypted-approval-log.md) - Approval event data
