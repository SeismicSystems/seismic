---
description: watch_src20_events function signatures
icon: broadcast
---

# watch_src20_events

| Function | Signature | Returns |
| --- | --- | --- |
| `watch_src20_events` | `watch_src20_events(w3: Web3, *, encryption: EncryptionState, private_key: PrivateKey, token_address: ChecksumAddress | None = None, on_transfer: TransferCallback | None = None, on_approval: ApprovalCallback | None = None, on_error: ErrorCallback | None = None, poll_interval: float = 2.0, from_block: int | str = "latest")` | `SRC20EventWatcher` |
| `async_watch_src20_events` | `async_watch_src20_events(w3: AsyncWeb3, *, encryption: EncryptionState, private_key: PrivateKey, token_address: ChecksumAddress | None = None, on_transfer: AsyncTransferCallback | TransferCallback | None = None, on_approval: AsyncApprovalCallback | ApprovalCallback | None = None, on_error: AsyncErrorCallback | ErrorCallback | None = None, poll_interval: float = 2.0, from_block: int | str = "latest")` | `AsyncSRC20EventWatcher` |
