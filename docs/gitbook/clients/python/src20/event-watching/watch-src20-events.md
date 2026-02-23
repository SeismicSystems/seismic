---
description: Watch SRC20 events using caller key from Directory
icon: broadcast
---

# watch_src20_events

Create a watcher that first fetches the caller viewing key from Directory, then starts polling decrypted SRC20 events.

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
    token_address: ChecksumAddress | None = None,
    on_transfer: AsyncTransferCallback | TransferCallback | None = None,
    on_approval: AsyncApprovalCallback | ApprovalCallback | None = None,
    on_error: AsyncErrorCallback | ErrorCallback | None = None,
    poll_interval: float = 2.0,
    from_block: int | str = "latest",
) -> AsyncSRC20EventWatcher
```

## Behavior

- Fetches viewing key via `get_viewing_key` / `async_get_viewing_key`.
- Starts watcher before returning it.
- If key is missing, raises `ValueError` from key fetch helper.
