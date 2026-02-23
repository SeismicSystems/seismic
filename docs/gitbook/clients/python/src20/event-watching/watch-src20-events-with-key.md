---
description: Watch SRC20 events with explicit viewing key
icon: broadcast
---

# watch_src20_events_with_key

Create a watcher using a provided viewing key (no key fetch from Directory).

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
    token_address: ChecksumAddress | None = None,
    on_transfer: AsyncTransferCallback | TransferCallback | None = None,
    on_approval: AsyncApprovalCallback | ApprovalCallback | None = None,
    on_error: AsyncErrorCallback | ErrorCallback | None = None,
    poll_interval: float = 2.0,
    from_block: int | str = "latest",
) -> AsyncSRC20EventWatcher
```

## Behavior

- No Directory lookup is performed.
- Starts watcher before returning it.
