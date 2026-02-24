---
description: SRC20 event watcher function signatures
icon: radar
---

# Event Watching

| Function | Signature | Returns |
| --- | --- | --- |
| `watch_src20_events` | `watch_src20_events(w3: Web3, *, encryption: EncryptionState, private_key: PrivateKey, token_address: ChecksumAddress | None = None, on_transfer: TransferCallback | None = None, on_approval: ApprovalCallback | None = None, on_error: ErrorCallback | None = None, poll_interval: float = 2.0, from_block: int | str = "latest")` | `SRC20EventWatcher` |
| `watch_src20_events_with_key` | `watch_src20_events_with_key(w3: Web3, *, viewing_key: Bytes32, token_address: ChecksumAddress | None = None, on_transfer: TransferCallback | None = None, on_approval: ApprovalCallback | None = None, on_error: ErrorCallback | None = None, poll_interval: float = 2.0, from_block: int | str = "latest")` | `SRC20EventWatcher` |
| `async_watch_src20_events` | `async_watch_src20_events(w3: AsyncWeb3, *, encryption: EncryptionState, private_key: PrivateKey, token_address: ChecksumAddress | None = None, on_transfer: AsyncTransferCallback | TransferCallback | None = None, on_approval: AsyncApprovalCallback | ApprovalCallback | None = None, on_error: AsyncErrorCallback | ErrorCallback | None = None, poll_interval: float = 2.0, from_block: int | str = "latest")` | `AsyncSRC20EventWatcher` |
| `async_watch_src20_events_with_key` | `async_watch_src20_events_with_key(w3: AsyncWeb3, *, viewing_key: Bytes32, token_address: ChecksumAddress | None = None, on_transfer: AsyncTransferCallback | TransferCallback | None = None, on_approval: AsyncApprovalCallback | ApprovalCallback | None = None, on_error: AsyncErrorCallback | ErrorCallback | None = None, poll_interval: float = 2.0, from_block: int | str = "latest")` | `AsyncSRC20EventWatcher` |
