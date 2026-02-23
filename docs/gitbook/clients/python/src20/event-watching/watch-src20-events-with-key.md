---
description: Watch SRC20 events with explicit viewing key
icon: broadcast
---

# watch_src20_events_with_key

Create a watcher with a provided viewing key (no Directory lookup).

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

async def async_watch_src20_events_with_key(...same args...) -> AsyncSRC20EventWatcher
```

## Example

```python
watcher = watch_src20_events_with_key(
    w3,
    viewing_key=viewing_key,
    token_address="0xTokenAddress",
    on_approval=lambda log: print("approval", log.decrypted_amount),
)
```
