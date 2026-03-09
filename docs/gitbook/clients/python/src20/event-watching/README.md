---
description: Watch and decrypt SRC20 Transfer and Approval events
icon: radar
---

# Event Watching

SRC20 tokens emit Transfer and Approval events with encrypted amounts. The event watching system polls for these events, decrypts them using your viewing key, and invokes callbacks with decoded data.

## Factory Functions

| Function | Description |
| --- | --- |
| [watch_src20_events](watch-src20-events.md) | Watch with automatic key fetching from Directory |
| [watch_src20_events_with_key](../intelligence-providers/watch-src20-events-with-key.md) | Watch with an explicit viewing key |

Both have async variants (`async_watch_src20_events`, `async_watch_src20_events_with_key`).

## Watcher Classes

| Class | Description |
| --- | --- |
| [SRC20EventWatcher](src20-event-watcher.md) | Thread-based polling watcher (sync) |
| [AsyncSRC20EventWatcher](async-src20-event-watcher.md) | Task-based polling watcher (async) |

## Example

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET
from seismic_web3.src20 import watch_src20_events

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=pk,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
    on_approval=lambda log: print(f"Approval: {log.decrypted_amount}"),
)

# Later...
watcher.stop()
```

## How It Works

1. **Key hash** — `keccak256(viewing_key)` is used to filter events by the 4th topic
2. **Polling** — `eth_getLogs` at a configurable interval (default 2s)
3. **Decryption** — AES-256-GCM with the viewing key (no AAD)
4. **Callbacks** — invoked with [`DecryptedTransferLog`](../types/decrypted-transfer-log.md) or [`DecryptedApprovalLog`](../types/decrypted-approval-log.md)

## See Also

- [Types](../types/) — Decrypted event data structures
- [Intelligence Providers](../intelligence-providers/) — Viewing key management
