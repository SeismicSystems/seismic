---
description: Method-level reference for `w3.seismic`
icon: list
---

# Namespace Methods

## Public methods

- [get_tee_public_key](get-tee-public-key.md)
- [get_deposit_root](get-deposit-root.md)
- [get_deposit_count](get-deposit-count.md)

Available on both public and wallet clients.

## Wallet methods

- [send_shielded_transaction](send-shielded-transaction.md)
- [debug_send_shielded_transaction](debug-send-shielded-transaction.md)
- [signed_call](signed-call.md)
- [deposit](deposit.md)

Available only on wallet clients.

## Utility

- [encode_shielded_calldata](encode-shielded-calldata.md)

Used internally by contract/namespace methods, and available directly when needed.

All methods have both sync and async variants. Async variants use `await`.
