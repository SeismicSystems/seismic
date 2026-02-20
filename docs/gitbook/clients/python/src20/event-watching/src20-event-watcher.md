---
description: Synchronous SRC20 event watcher class
icon: clock
---

# SRC20EventWatcher

Polling-based SRC20 event watcher that runs in a background thread (synchronous).

## Overview

`SRC20EventWatcher` is a thread-based event watcher that polls for SRC20 Transfer and Approval events, decrypts encrypted amounts using a viewing key, and invokes callbacks with decoded event data.

The watcher runs in a daemon background thread and polls at a configurable interval. It automatically handles block number tracking, log fetching, decryption, and callback invocation.

Typically created via [`watch_src20_events()`](watch-src20-events.md) or [`watch_src20_events_with_key()`](watch-src20-events-with-key.md) rather than instantiated directly.

## Class Definition

```python
class SRC20EventWatcher:
    """Polling-based SRC20 event watcher (sync, runs in a background thread)."""

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

## Constructor Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `w3` | `Web3` | Yes | Web3 instance |
| `aes_key` | [`Bytes32`](../../api-reference/types/bytes32.md) | Yes | 32-byte AES-256 viewing key |
| `token_address` | `ChecksumAddress` | No | SRC20 contract to filter. If `None`, watches all tokens |
| `on_transfer` | Callback | No | Callback invoked for Transfer events |
| `on_approval` | Callback | No | Callback invoked for Approval events |
| `on_error` | Callback | No | Callback for errors |
| `poll_interval` | `float` | No | Seconds between polls (default `2.0`) |
| `from_block` | `int` or `"latest"` | No | Starting block number or `"latest"` (default) |

## Methods

### start()

Start the background polling thread.

```python
def start(self) -> None
```

### stop()

Signal the polling thread to stop and wait for it to exit.

```python
def stop(self) -> None
```

## Properties

### is_running

Check if the watcher is currently running.

```python
@property
def is_running(self) -> bool
```

Returns `True` if the background thread is alive, `False` otherwise.

## Context Manager Support

The watcher can be used as a context manager:

```python
def __enter__(self) -> SRC20EventWatcher
def __exit__(self, *exc: object) -> None
```

The watcher starts automatically on `__enter__` and stops on `__exit__`.

## Examples

### Manual Start/Stop

```python
import os
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import SRC20EventWatcher

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex(os.environ["VIEWING_KEY"]))

watcher = SRC20EventWatcher(
    w3,
    viewing_key,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
)

watcher.start()
# ... do work ...
watcher.stop()
```

### Context Manager

```python
import os
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import SRC20EventWatcher
import time

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex(os.environ["VIEWING_KEY"]))

with SRC20EventWatcher(
    w3,
    viewing_key,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
) as watcher:
    time.sleep(60)
# Automatically stopped
```

### Check Running Status

```python
import os
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import SRC20EventWatcher

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex(os.environ["VIEWING_KEY"]))

watcher = SRC20EventWatcher(
    w3,
    viewing_key,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
)

print(f"Running: {watcher.is_running}")  # False

watcher.start()
print(f"Running: {watcher.is_running}")  # True

watcher.stop()
print(f"Running: {watcher.is_running}")  # False
```

### Filter by Token Address

```python
import os
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import SRC20EventWatcher

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex(os.environ["VIEWING_KEY"]))
token_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

watcher = SRC20EventWatcher(
    w3,
    viewing_key,
    token_address=token_address,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
)
watcher.start()
```

### Custom Poll Interval

```python
import os
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import SRC20EventWatcher

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex(os.environ["VIEWING_KEY"]))

# Poll every 5 seconds instead of default 2
watcher = SRC20EventWatcher(
    w3,
    viewing_key,
    poll_interval=5.0,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
)
watcher.start()
```

### Historical Events

```python
import os
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import SRC20EventWatcher

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex(os.environ["VIEWING_KEY"]))

# Start watching from block 1000000
watcher = SRC20EventWatcher(
    w3,
    viewing_key,
    from_block=1000000,
    on_transfer=lambda log: print(f"Block {log.block_number}: {log.decrypted_amount}"),
)
watcher.start()
```

### Error Handling

```python
import os
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import SRC20EventWatcher

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex(os.environ["VIEWING_KEY"]))

def on_error(exc: Exception):
    print(f"Error in watcher: {exc}")
    # Log to monitoring system, retry, etc.

watcher = SRC20EventWatcher(
    w3,
    viewing_key,
    on_transfer=lambda log: print(f"Amount: {log.decrypted_amount}"),
    on_error=on_error,
)
watcher.start()
```

### Both Transfer and Approval

```python
import os
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20.watch import SRC20EventWatcher

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex(os.environ["VIEWING_KEY"]))

def on_transfer(log):
    print(f"Transfer from {log.from_address} to {log.to_address}: {log.decrypted_amount}")

def on_approval(log):
    print(f"Approval from {log.owner} to {log.spender}: {log.decrypted_amount}")

watcher = SRC20EventWatcher(
    w3,
    viewing_key,
    on_transfer=on_transfer,
    on_approval=on_approval,
)
watcher.start()
```

## How It Works

1. **Thread initialization** - Creates a daemon thread named `"src20-watcher"`
2. **Polling loop** - In the background thread:
   - Fetches latest block number
   - Builds `eth_getLogs` filter with key hash
   - Fetches logs for the block range
   - Decodes and decrypts each log
   - Invokes callbacks with decrypted data
   - Waits for `poll_interval` seconds
3. **Cleanup** - When stopped, signals the thread to exit and waits up to `poll_interval * 3` seconds

## Callback Signatures

### Transfer Callback

```python
def on_transfer(log: DecryptedTransferLog) -> None:
    pass
```

See [`DecryptedTransferLog`](../types/decrypted-transfer-log.md) for field details.

### Approval Callback

```python
def on_approval(log: DecryptedApprovalLog) -> None:
    pass
```

See [`DecryptedApprovalLog`](../types/decrypted-approval-log.md) for field details.

### Error Callback

```python
def on_error(exc: Exception) -> None:
    pass
```

Invoked for:
- RPC errors (network issues, rate limiting)
- Decryption failures (wrong key, corrupted data)
- Any exceptions during log processing

## Notes

- The polling thread is a daemon thread (exits when main thread exits)
- Thread name is `"src20-watcher"` for debugging
- Block tracking is automatic; watcher remembers the last processed block
- Events are processed in order by block number
- If `on_error` is not provided, errors are logged at DEBUG level
- The watcher computes `keccak256(aes_key)` once at initialization

## Warnings

- **Thread safety** - Callbacks run in the background thread; ensure thread-safe operations
- **Daemon thread** - Will be forcefully terminated if main thread exits
- **Cleanup required** - Always call `.stop()` or use context manager to clean up
- **Network failures** - Errors are silently logged unless `on_error` is provided
- **Memory usage** - Keep poll interval reasonable to avoid excessive log fetching

## See Also

- [watch_src20_events](watch-src20-events.md) - Factory function with automatic key fetching
- [watch_src20_events_with_key](watch-src20-events-with-key.md) - Factory function with explicit key
- [AsyncSRC20EventWatcher](async-src20-event-watcher.md) - Async variant
- [DecryptedTransferLog](../types/decrypted-transfer-log.md) - Transfer event data
- [DecryptedApprovalLog](../types/decrypted-approval-log.md) - Approval event data
