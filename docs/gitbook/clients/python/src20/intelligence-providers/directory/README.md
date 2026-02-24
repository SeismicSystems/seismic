---
description: Manage viewing keys in the Directory genesis contract
icon: book
---

# Directory

Manage viewing keys stored in the Directory genesis contract at `0x1000000000000000000000000000000000000004`.

Viewing keys are 32-byte AES-256 keys used to decrypt SRC20 Transfer and Approval event amounts. The Directory provides:

- **Key registration** — store your viewing key on-chain via shielded write
- **Key retrieval** — fetch your key via authenticated signed read
- **Public queries** — check if an address has a key or get its hash (no auth needed)

## Functions

| Function | Returns | Description |
| --- | --- | --- |
| [`register_viewing_key`](register-viewing-key.md) | `HexBytes` | Register a viewing key (shielded write) |
| [`get_viewing_key`](get-viewing-key.md) | `Bytes32` | Fetch your viewing key (signed read) |
| [`check_has_key`](check-has-key.md) | `bool` | Check if address has a key (public) |
| [`get_key_hash`](check-has-key.md) | `bytes` | Get `keccak256` hash of address's key (public) |
| `compute_key_hash` | `bytes` | Compute `keccak256(viewing_key)` locally (pure) |

All functions except `compute_key_hash` have async variants prefixed with `async_`.

## Example

```python
import os
from seismic_web3 import PrivateKey, Bytes32, SEISMIC_TESTNET
from seismic_web3.src20 import register_viewing_key, get_viewing_key

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Register a viewing key
viewing_key = Bytes32(os.urandom(32))
tx_hash = register_viewing_key(w3, w3.seismic.encryption, pk, key=viewing_key)
w3.eth.wait_for_transaction_receipt(tx_hash)

# Fetch it back
fetched = get_viewing_key(w3, w3.seismic.encryption, pk)
assert fetched == viewing_key
```

## See Also

- [Event Watching](../../event-watching/) — Watch and decrypt SRC20 events
- [Types](../../types/) — Decrypted event data structures
