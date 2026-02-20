---
description: Watch SRC20 events with an explicit viewing key
icon: key
---

# watch\_src20\_events\_with\_key

Watch SRC20 Transfer and Approval events using an explicit viewing key, without requiring wallet authentication.

## Overview

`watch_src20_events_with_key()` and `async_watch_src20_events_with_key()` create event watchers that poll for SRC20 events using a provided viewing key. Unlike [`watch_src20_events`](watch-src20-events.md), these functions do not require encryption state or private key — only a plain `Web3` instance and the 32-byte viewing key.

This is useful when:

* You have a viewing key but don't have the wallet's private key
* You want to watch someone else's events (if they shared their viewing key)
* You want to avoid the RPC call to fetch the key from Directory

## Signature

```python
def watch_src20_events_with_key(
    w3: Web3,
    *,
    viewing_key: Bytes32,
    token_address: ChecksumAddress | None = None,
    on_transfer: TransferCallback | None = None,
    on_approval: ApprovalCallback | None = None,
    on_error: ErrorCallback | None = None,
    poll_interval: float = 2.0,
    from_block: int | str = "latest",
) -> SRC20EventWatcher

async def async_watch_src20_events_with_key(
    w3: AsyncWeb3,
    *,
    viewing_key: Bytes32,
    token_address: ChecksumAddress | None = None,
    on_transfer: AsyncTransferCallback | TransferCallback | None = None,
    on_approval: AsyncApprovalCallback | ApprovalCallback | None = None,
    on_error: AsyncErrorCallback | ErrorCallback | None = None,
    poll_interval: float = 2.0,
    from_block: int | str = "latest",
) -> AsyncSRC20EventWatcher
```

## Parameters

| Parameter       | Type                                      | Required | Description                                                   |
| --------------- | ----------------------------------------- | -------- | ------------------------------------------------------------- |
| `w3`            | `Web3` or `AsyncWeb3`                     | Yes      | Standard Web3 instance (no Seismic namespace required)        |
| `viewing_key`   | [`Bytes32`](../../api-reference/bytes32/) | Yes      | 32-byte AES-256 viewing key                                   |
| `token_address` | `ChecksumAddress`                         | No       | SRC20 contract to filter. If `None`, watches all SRC20 tokens |
| `on_transfer`   | Callback                                  | No       | Callback invoked for Transfer events                          |
| `on_approval`   | Callback                                  | No       | Callback invoked for Approval events                          |
| `on_error`      | Callback                                  | No       | Callback for errors (decryption failures, RPC errors)         |
| `poll_interval` | `float`                                   | No       | Seconds between polls (default `2.0`)                         |
| `from_block`    | `int` or `"latest"`                       | No       | Starting block number or `"latest"` (default)                 |

## Returns

| Type                                                     | Description                     |
| -------------------------------------------------------- | ------------------------------- |
| [`SRC20EventWatcher`](src20-event-watcher.md)            | Sync watcher (already started)  |
| [`AsyncSRC20EventWatcher`](async-src20-event-watcher.md) | Async watcher (already started) |

## Examples

### Basic Usage (Sync)

```python
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20 import watch_src20_events_with_key

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Provide viewing key directly
viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

watcher = watch_src20_events_with_key(
    w3,
    viewing_key=viewing_key,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
    on_approval=lambda log: print(f"Approval: {log.decrypted_amount}"),
)

# Later...
watcher.stop()
```

### Watch Specific Token

```python
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20 import watch_src20_events_with_key

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))
token_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"

watcher = watch_src20_events_with_key(
    w3,
    viewing_key=viewing_key,
    token_address=token_address,
    on_transfer=lambda log: print(
        f"From {log.from_address} to {log.to_address}: {log.decrypted_amount}"
    ),
)
```

### Read-Only Monitoring

```python
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20 import watch_src20_events_with_key

# No private key needed — just a public RPC endpoint
w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex("SHARED_VIEWING_KEY_HEX"))

def on_transfer(log):
    print(f"Block {log.block_number}: {log.decrypted_amount}")
    # Store in database, send notification, etc.

watcher = watch_src20_events_with_key(
    w3,
    viewing_key=viewing_key,
    on_transfer=on_transfer,
)
```

### With Context Manager

```python
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20 import watch_src20_events_with_key
import time

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

with watch_src20_events_with_key(
    w3,
    viewing_key=viewing_key,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
) as watcher:
    time.sleep(60)
# Automatically stopped
```

### Error Handling

```python
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20 import watch_src20_events_with_key

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

def on_error(exc: Exception):
    print(f"Error in watcher: {exc}")
    # Log to monitoring system, retry, etc.

watcher = watch_src20_events_with_key(
    w3,
    viewing_key=viewing_key,
    on_transfer=lambda log: print(f"Amount: {log.decrypted_amount}"),
    on_error=on_error,
)
```

### Historical Events

```python
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20 import watch_src20_events_with_key

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

# Start watching from block 1000000
watcher = watch_src20_events_with_key(
    w3,
    viewing_key=viewing_key,
    from_block=1000000,
    on_transfer=lambda log: print(f"Block {log.block_number}: {log.decrypted_amount}"),
)
```

### Async Usage

```python
from web3 import AsyncWeb3
from seismic_web3 import Bytes32
from seismic_web3.src20 import async_watch_src20_events_with_key
import asyncio

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

    # Async callback
    async def on_transfer(log):
        print(f"Transfer: {log.decrypted_amount}")
        # Can perform async operations here
        await some_async_operation(log)

    watcher = await async_watch_src20_events_with_key(
        w3,
        viewing_key=viewing_key,
        on_transfer=on_transfer,
    )

    # Run for 60 seconds
    await asyncio.sleep(60)
    await watcher.stop()

asyncio.run(main())
```

### Async Context Manager

```python
from web3 import AsyncWeb3
from seismic_web3 import Bytes32
from seismic_web3.src20 import async_watch_src20_events_with_key
import asyncio

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

    async with await async_watch_src20_events_with_key(
        w3,
        viewing_key=viewing_key,
        on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
    ) as watcher:
        await asyncio.sleep(60)
    # Automatically stopped

asyncio.run(main())
```

### Monitor Multiple Keys

```python
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20 import watch_src20_events_with_key

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Watch events for multiple viewing keys
keys = [
    Bytes32(bytes.fromhex("KEY1_HEX")),
    Bytes32(bytes.fromhex("KEY2_HEX")),
    Bytes32(bytes.fromhex("KEY3_HEX")),
]

watchers = []
for i, key in enumerate(keys):
    watcher = watch_src20_events_with_key(
        w3,
        viewing_key=key,
        on_transfer=lambda log, idx=i: print(f"Key {idx} transfer: {log.decrypted_amount}"),
    )
    watchers.append(watcher)

# Later... stop all
for watcher in watchers:
    watcher.stop()
```

## How It Works

1. **Compute key hash** - Calculates `keccak256(viewing_key)` for event filtering
2. **Create watcher** - Instantiates `SRC20EventWatcher` or `AsyncSRC20EventWatcher` with the viewing key
3. **Start polling** - Begins background polling (thread for sync, task for async)
4. **Process events** - For each poll:
   * Fetches logs via `eth_getLogs` with key hash filter
   * Decodes event topics and data
   * Decrypts encrypted amounts using AES-256-GCM
   * Invokes callbacks with decrypted data

## Callback Signatures

### Transfer Callback

```python
# Sync
def on_transfer(log: DecryptedTransferLog) -> None:
    pass

# Async
async def on_transfer(log: DecryptedTransferLog) -> None:
    pass
```

See [`DecryptedTransferLog`](../types/decrypted-transfer-log.md) for field details.

### Approval Callback

```python
# Sync
def on_approval(log: DecryptedApprovalLog) -> None:
    pass

# Async
async def on_approval(log: DecryptedApprovalLog) -> None:
    pass
```

See [`DecryptedApprovalLog`](../types/decrypted-approval-log.md) for field details.

### Error Callback

```python
# Sync
def on_error(exc: Exception) -> None:
    pass

# Async
async def on_error(exc: Exception) -> None:
    pass
```

## Notes

* Does not require `EncryptionState` or wallet private key
* Works with standard `Web3` or `AsyncWeb3` instances
* No RPC calls are made to fetch the viewing key (unlike [`watch_src20_events`](watch-src20-events.md))
* The watcher starts automatically upon creation
* For sync version, polls in a background daemon thread
* For async version, runs as an `asyncio.Task`
* Events are only visible if they match your key hash
* Call `.stop()` or use context manager to clean up

## Warnings

* **Key security** - Keep viewing keys secure; they allow decrypting all events
* **Shared keys** - Be cautious when sharing viewing keys; recipients can see all events
* **Network connectivity** - Polls will fail silently if RPC connection drops (check `on_error`)
* **Thread safety** - Sync version spawns a daemon thread; ensure proper cleanup
* **Memory usage** - Keep poll interval reasonable to avoid excessive log fetching

## See Also

* [watch\_src20\_events](watch-src20-events.md) - Watch with automatic key fetching from Directory
* [SRC20EventWatcher](src20-event-watcher.md) - Sync watcher class
* [AsyncSRC20EventWatcher](async-src20-event-watcher.md) - Async watcher class
* [get\_viewing\_key](../directory/get-viewing-key.md) - Fetch viewing key from Directory
* [DecryptedTransferLog](../types/decrypted-transfer-log.md) - Transfer event data
* [DecryptedApprovalLog](../types/decrypted-approval-log.md) - Approval event data
