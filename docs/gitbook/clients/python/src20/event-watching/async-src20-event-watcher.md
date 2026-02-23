---
description: Async SRC20 polling watcher class
icon: activity
---

# AsyncSRC20EventWatcher

`asyncio` task-based watcher for decrypted SRC20 events.

## Constructor

```python
class AsyncSRC20EventWatcher:
    def __init__(
        self,
        w3: AsyncWeb3,
        aes_key: Bytes32,
        *,
        token_address: ChecksumAddress | None = None,
        on_transfer: AsyncTransferCallback | TransferCallback | None = None,
        on_approval: AsyncApprovalCallback | ApprovalCallback | None = None,
        on_error: AsyncErrorCallback | ErrorCallback | None = None,
        poll_interval: float = 2.0,
        from_block: int | str = "latest",
    ) -> None
```

## Lifecycle

```python
async def start(self) -> None
async def stop(self) -> None
@property
def is_running(self) -> bool

async def __aenter__(self) -> AsyncSRC20EventWatcher
async def __aexit__(self, *exc: object) -> None
```

## Notes

- Accepts both sync and async callbacks.
- Uses `asyncio.create_task` internally for polling.
