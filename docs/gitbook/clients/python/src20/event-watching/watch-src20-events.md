---
description: Watch SRC20 events with automatic key fetching
icon: eye
---

# watch_src20_events

Watch SRC20 Transfer and Approval events for your wallet, automatically fetching your viewing key from the Directory contract via signed read.

## Signatures

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
    ...same keyword args...
) -> AsyncSRC20EventWatcher
```

## Parameters

| Parameter | Type | Default | Description |
| --- | --- | --- | --- |
| `w3` | `Web3` / `AsyncWeb3` | | Web3 instance with Seismic support |
| `encryption` | [`EncryptionState`](../../client/encryption-state.md) | | Encryption state from wallet client |
| `private_key` | [`PrivateKey`](../../api-reference/types/private-key.md) | | Signing key for Directory signed read |
| `token_address` | `ChecksumAddress \| None` | `None` | SRC20 contract to filter (`None` = all tokens) |
| `on_transfer` | Callback | `None` | Invoked for Transfer events |
| `on_approval` | Callback | `None` | Invoked for Approval events |
| `on_error` | Callback | `None` | Invoked on errors (decryption, RPC) |
| `poll_interval` | `float` | `2.0` | Seconds between polls |
| `from_block` | `int \| "latest"` | `"latest"` | Starting block |

## Returns

[`SRC20EventWatcher`](src20-event-watcher.md) (sync) or [`AsyncSRC20EventWatcher`](async-src20-event-watcher.md) (async) — already started.

## Example

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET
from seismic_web3.src20 import watch_src20_events

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

with watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=pk,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
) as watcher:
    input("Press Enter to stop...\n")
```

## Notes

- Fetches your viewing key from Directory via `get_viewing_key()` on creation — raises `ValueError` if no key registered
- The watcher starts automatically; call `.stop()` or use context manager to clean up
- If you already have the viewing key, use [`watch_src20_events_with_key()`](../intelligence-providers/watch-src20-events-with-key.md) to skip the signed read

## See Also

- [watch_src20_events_with_key](../intelligence-providers/watch-src20-events-with-key.md) — Watch with explicit key (no wallet needed)
- [SRC20EventWatcher](src20-event-watcher.md) — Underlying sync watcher class
- [register_viewing_key](../intelligence-providers/directory/register-viewing-key.md) — Register a key first
