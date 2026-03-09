---
description: Watch SRC20 events with an explicit viewing key
icon: satellite-dish
---

# watch_src20_events_with_key

Watch SRC20 Transfer and Approval events using an explicit viewing key, without requiring wallet authentication.

## Signatures

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
    ...same keyword args...
) -> AsyncSRC20EventWatcher
```

## Parameters

| Parameter | Type | Default | Description |
| --- | --- | --- | --- |
| `w3` | `Web3` / `AsyncWeb3` | | Standard Web3 instance (no Seismic namespace needed) |
| `viewing_key` | [`Bytes32`](../../api-reference/types/bytes32.md) | | 32-byte AES-256 viewing key |
| `token_address` | `ChecksumAddress \| None` | `None` | SRC20 contract to filter (`None` = all tokens) |
| `on_transfer` | Callback | `None` | Invoked for Transfer events |
| `on_approval` | Callback | `None` | Invoked for Approval events |
| `on_error` | Callback | `None` | Invoked on errors |
| `poll_interval` | `float` | `2.0` | Seconds between polls |
| `from_block` | `int \| "latest"` | `"latest"` | Starting block |

## Returns

[`SRC20EventWatcher`](../event-watching/src20-event-watcher.md) (sync) or [`AsyncSRC20EventWatcher`](../event-watching/async-src20-event-watcher.md) (async) — already started.

## Example

```python
import os
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20 import watch_src20_events_with_key

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex(os.environ["VIEWING_KEY"].removeprefix("0x")))

watcher = watch_src20_events_with_key(
    w3,
    viewing_key=viewing_key,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
)

# Later...
watcher.stop()
```

## Notes

- Does **not** require `EncryptionState` or a wallet private key — only a plain `Web3` instance and the viewing key
- No RPC call to Directory (unlike `watch_src20_events` which fetches the key)
- Useful when you have a shared viewing key or want to avoid the signed read

## See Also

- [watch_src20_events](../event-watching/watch-src20-events.md) — Watch with automatic key fetch from Directory
- [SRC20EventWatcher](../event-watching/src20-event-watcher.md) — Underlying sync watcher class
- [get_viewing_key](get-viewing-key.md) — Fetch a key to pass here
