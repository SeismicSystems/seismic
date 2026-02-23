---
description: SRC20 viewing keys, event watching, and decoded log types
icon: coins
---

# SRC20

The Python SDK provides SRC20 helpers for:

- Directory key management (`register_viewing_key`, `get_viewing_key`, key queries)
- Event watching with decryption (`watch_src20_events*`)
- Typed decoded logs (`DecryptedTransferLog`, `DecryptedApprovalLog`)

## Sections

- [Directory](directory/README.md)
- [Event Watching](event-watching/README.md)
- [Types](types/README.md)

## Source Of Truth

- `clients/py/src/seismic_web3/src20/directory.py`
- `clients/py/src/seismic_web3/src20/watch.py`
- `clients/py/src/seismic_web3/src20/types.py`
- `clients/py/src/seismic_web3/src20/crypto.py`
