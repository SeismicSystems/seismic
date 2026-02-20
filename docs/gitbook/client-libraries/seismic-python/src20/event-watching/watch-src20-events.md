---
description: Watch SRC20 events for your wallet with automatic key fetching
icon: eye
---

# watch_src20_events

Watch SRC20 Transfer and Approval events for your wallet, automatically fetching the viewing key from the Directory contract.

## Overview

`watch_src20_events()` and `async_watch_src20_events()` create event watchers that poll for SRC20 events, decrypt the encrypted amounts using your viewing key from the Directory contract, and invoke callbacks with the decrypted data.

The functions automatically:
- Fetch your viewing key from the Directory contract via signed read
- Compute the key hash for event filtering
- Poll for Transfer and Approval events matching your key hash
- Decrypt encrypted amounts using AES-256-GCM
- Invoke callbacks with fully decoded event data

## Signature

```python
def watch_src20_events(
    w3: Web3,
    *,
    encryption: EncryptionState,
    private_key: PrivateKey,
    token_address: ChecksumAddress | None = None,
    on_transfer: TransferCallback | None = None,
    on_approval: ApprovalCallback | None = None,
    on_error: ErrorCallback | None = None,
    poll_interval: float = 2.0,
    from_block: int | str = "latest",
) -> SRC20EventWatcher

async def async_watch_src20_events(
    w3: AsyncWeb3,
    *,
    encryption: EncryptionState,
    private_key: PrivateKey,
    token_address: ChecksumAddress | None = None,
    on_transfer: AsyncTransferCallback | TransferCallback | None = None,
    on_approval: AsyncApprovalCallback | ApprovalCallback | None = None,
    on_error: AsyncErrorCallback | ErrorCallback | None = None,
    poll_interval: float = 2.0,
    from_block: int | str = "latest",
) -> AsyncSRC20EventWatcher
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `w3` | `Web3` or `AsyncWeb3` | Yes | Web3 instance with Seismic support |
| `encryption` | [`EncryptionState`](../../client/encryption-state.md) | Yes | Encryption state from wallet client |
| `private_key` | [`PrivateKey`](../../api-reference/types/private-key.md) | Yes | 32-byte signing key for signed reads |
| `token_address` | `ChecksumAddress` | No | SRC20 contract to filter. If `None`, watches all SRC20 tokens |
| `on_transfer` | Callback | No | Callback invoked for Transfer events |
| `on_approval` | Callback | No | Callback invoked for Approval events |
| `on_error` | Callback | No | Callback for errors (decryption failures, RPC errors) |
| `poll_interval` | `float` | No | Seconds between polls (default `2.0`) |
| `from_block` | `int` or `"latest"` | No | Starting block number or `"latest"` (default) |

## Returns

| Type | Description |
|------|-------------|
| [`SRC20EventWatcher`](src20-event-watcher.md) | Sync watcher (already started) |
| [`AsyncSRC20EventWatcher`](async-src20-event-watcher.md) | Async watcher (already started) |

## Examples

### Basic Usage (Sync)

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import watch_src20_events

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Watch all SRC20 events for your wallet
watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
    on_approval=lambda log: print(f"Approval: {log.decrypted_amount}"),
)

# Later...
watcher.stop()
```

### Watch Specific Token

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import watch_src20_events

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

token_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"

watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    token_address=token_address,
    on_transfer=lambda log: print(
        f"Transfer from {log.from_address} to {log.to_address}: {log.decrypted_amount}"
    ),
)
```

### With Context Manager

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import watch_src20_events
import time

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

with watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
) as watcher:
    # Watcher runs in background
    time.sleep(60)
# Automatically stopped when exiting context
```

### Error Handling

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import watch_src20_events

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

def on_error(exc: Exception):
    print(f"Error in watcher: {exc}")
    # Log to monitoring system, retry, etc.

watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    on_transfer=lambda log: print(f"Amount: {log.decrypted_amount}"),
    on_error=on_error,
)
```

### Historical Events

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import watch_src20_events

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Start watching from block 1000000
watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    from_block=1000000,
    on_transfer=lambda log: print(f"Block {log.block_number}: {log.decrypted_amount}"),
)
```

### Async Usage

```python
from seismic_web3 import create_async_wallet_client, PrivateKey
from seismic_web3.src20 import async_watch_src20_events
import asyncio

async def main():
    private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
    w3 = await create_async_wallet_client(
        "wss://gcp-1.seismictest.net/ws",
        private_key=private_key,
    )

    # Async callback
    async def on_transfer(log):
        print(f"Transfer: {log.decrypted_amount}")
        # Can perform async operations here
        await some_async_operation(log)

    watcher = await async_watch_src20_events(
        w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
        on_transfer=on_transfer,
    )

    # Run for 60 seconds
    await asyncio.sleep(60)
    await watcher.stop()

asyncio.run(main())
```

### Async Context Manager

```python
from seismic_web3 import create_async_wallet_client, PrivateKey
from seismic_web3.src20 import async_watch_src20_events
import asyncio

async def main():
    private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
    w3 = await create_async_wallet_client(
        "wss://gcp-1.seismictest.net/ws",
        private_key=private_key,
    )

    async with await async_watch_src20_events(
        w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
        on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
    ) as watcher:
        await asyncio.sleep(60)
    # Automatically stopped

asyncio.run(main())
```

## How It Works

1. **Fetch viewing key** - Calls the Directory contract's `getKey()` via signed read to fetch your viewing key
2. **Compute key hash** - Calculates `keccak256(viewing_key)` for event filtering
3. **Create watcher** - Instantiates `SRC20EventWatcher` or `AsyncSRC20EventWatcher` with the viewing key
4. **Start polling** - Begins background polling (thread for sync, task for async)
5. **Process events** - For each poll:
   - Fetches logs via `eth_getLogs` with key hash filter
   - Decodes event topics and data
   - Decrypts encrypted amounts using AES-256-GCM
   - Invokes callbacks with decrypted data

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

- The watcher starts automatically upon creation
- For sync version, polls in a background daemon thread
- For async version, runs as an `asyncio.Task`
- Requires a registered viewing key in the Directory contract
- Events are only visible if they match your key hash
- Call `.stop()` or use context manager to clean up
- The `from_block` parameter defaults to `"latest"` (current block)

## Warnings

- **Viewing key required** - Raises `ValueError` if no viewing key is registered in Directory
- **Network connectivity** - Polls will fail silently if RPC connection drops (check `on_error`)
- **Gas costs** - This function makes an RPC call to fetch the viewing key
- **Thread safety** - Sync version spawns a daemon thread; ensure proper cleanup
- **Memory usage** - Keep poll interval reasonable to avoid excessive log fetching

## See Also

- [watch_src20_events_with_key](watch-src20-events-with-key.md) - Watch with explicit viewing key
- [SRC20EventWatcher](src20-event-watcher.md) - Sync watcher class
- [AsyncSRC20EventWatcher](async-src20-event-watcher.md) - Async watcher class
- [register_viewing_key](../directory/register-viewing-key.md) - Register a viewing key
- [get_viewing_key](../directory/get-viewing-key.md) - Fetch your viewing key
- [DecryptedTransferLog](../types/decrypted-transfer-log.md) - Transfer event data
- [DecryptedApprovalLog](../types/decrypted-approval-log.md) - Approval event data
