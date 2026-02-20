---
description: Watch and decrypt SRC20 Transfer and Approval events
icon: eye
---

# Event Watching

Watch SRC20 Transfer and Approval events with automatic decryption of encrypted amounts.

## Overview

SRC20 tokens emit Transfer and Approval events with encrypted amounts. The event watching system provides tools to:

- Poll for SRC20 events matching your viewing key
- Automatically decrypt encrypted amounts using AES-256-GCM
- Invoke callbacks with fully decoded event data
- Handle both sync (threading) and async (asyncio) execution models

## Quick Start

### Watch Your Own Events

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import watch_src20_events

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://rpc.seismic.network", private_key=private_key)

watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
)

# Later...
watcher.stop()
```

### Watch with Explicit Key

```python
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20 import watch_src20_events_with_key

w3 = Web3(Web3.HTTPProvider("https://rpc.seismic.network"))
viewing_key = Bytes32(bytes.fromhex("YOUR_32_BYTE_VIEWING_KEY_HEX"))

watcher = watch_src20_events_with_key(
    w3,
    viewing_key=viewing_key,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
)
```

## Factory Functions

High-level functions that create and start watchers:

| Function | Description |
|----------|-------------|
| [watch_src20_events](watch-src20-events.md) | Watch with automatic key fetching (sync) |
| [watch_src20_events_with_key](watch-src20-events-with-key.md) | Watch with explicit viewing key (sync) |

Both have async variants (`async_watch_src20_events`, `async_watch_src20_events_with_key`).

## Watcher Classes

Low-level classes for advanced use cases:

| Class | Description |
|-------|-------------|
| [SRC20EventWatcher](src20-event-watcher.md) | Thread-based watcher (sync) |
| [AsyncSRC20EventWatcher](async-src20-event-watcher.md) | Task-based watcher (async) |

## Event Types

Decoded event data structures:

| Type | Description |
|------|-------------|
| [DecryptedTransferLog](../types/decrypted-transfer-log.md) | Transfer event with decrypted amount |
| [DecryptedApprovalLog](../types/decrypted-approval-log.md) | Approval event with decrypted amount |

## How It Works

1. **Key hash filtering** - Events are filtered by `keccak256(viewing_key)` in the event topics
2. **Polling** - The watcher polls `eth_getLogs` at a configurable interval
3. **Decryption** - Encrypted amounts are decrypted using AES-256-GCM with the viewing key
4. **Callbacks** - User callbacks are invoked with fully decoded event data

## Sync vs Async

### Sync (Threading)

- Runs in a background daemon thread
- Uses `Web3` (synchronous)
- Callbacks are regular functions
- Good for scripts and simple applications

```python
watcher = watch_src20_events(w3, ...)
watcher.stop()
```

### Async (asyncio)

- Runs as an `asyncio.Task`
- Uses `AsyncWeb3` (asynchronous)
- Callbacks can be async or sync functions
- Good for async applications and WebSocket connections

```python
watcher = await async_watch_src20_events(w3, ...)
await watcher.stop()
```

## Common Patterns

### Context Manager

```python
with watch_src20_events(...) as watcher:
    time.sleep(60)
# Automatically stopped
```

### Error Handling

```python
def on_error(exc: Exception):
    print(f"Error: {exc}")

watcher = watch_src20_events(
    w3,
    ...,
    on_error=on_error,
)
```

### Filter by Token

```python
watcher = watch_src20_events(
    w3,
    ...,
    token_address="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
)
```

### Historical Events

```python
watcher = watch_src20_events(
    w3,
    ...,
    from_block=1000000,  # Start from specific block
)
```

## See Also

- [Directory](../directory/README.md) - Manage viewing keys
- [Types](../types/README.md) - Event data structures
- [SRC20 Overview](../README.md) - Full SRC20 documentation
