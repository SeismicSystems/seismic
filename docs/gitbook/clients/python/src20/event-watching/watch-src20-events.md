---
description: Watch SRC20 events using caller key from Directory
icon: broadcast
---

# watch_src20_events

Create a watcher that fetches caller viewing key from Directory, then starts polling.

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

async def async_watch_src20_events(...same args...) -> AsyncSRC20EventWatcher
```

## Example

```python
watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=pk,
    token_address="0xTokenAddress",
    on_transfer=lambda log: print("transfer", log.decrypted_amount),
)
```

If no viewing key is registered, key fetch raises `ValueError`.
