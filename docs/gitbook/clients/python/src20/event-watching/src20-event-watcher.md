---
description: Sync SRC20 polling watcher class
icon: activity
---

# SRC20EventWatcher

Background-thread polling watcher for decrypted SRC20 events.

## Constructor

```python
class SRC20EventWatcher:
    def __init__(
        self,
        w3: Web3,
        aes_key: Bytes32,
        *,
        token_address: ChecksumAddress | None = None,
        on_transfer: TransferCallback | None = None,
        on_approval: ApprovalCallback | None = None,
        on_error: ErrorCallback | None = None,
        poll_interval: float = 2.0,
        from_block: int | str = "latest",
    ) -> None
```

## Lifecycle

```python
def start(self) -> None
def stop(self) -> None
@property
def is_running(self) -> bool

def __enter__(self) -> SRC20EventWatcher
def __exit__(self, *exc: object) -> None
```

## Notes

- Polls with `eth_getLogs`.
- Filters by Transfer/Approval topic and viewing-key hash topic.
- Decryption or callback errors go to `on_error` when provided.
