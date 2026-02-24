---
description: Viewing key management and encrypted event decryption
icon: eye-slash
---

# Intelligence Providers

Manage viewing keys and decrypt SRC20 event data. Viewing keys are 32-byte AES-256 keys stored in the [Directory](../../abis/directory.md) genesis contract that allow holders to decrypt Transfer and Approval event amounts.

## Functions

| Function | Returns | Description |
| --- | --- | --- |
| [`register_viewing_key`](register-viewing-key.md) | `HexBytes` | Register a viewing key (shielded write) |
| [`get_viewing_key`](get-viewing-key.md) | `Bytes32` | Fetch your viewing key (signed read) |
| [`check_has_key`](check-has-key.md) | `bool` | Check if address has a key (public) |
| [`get_key_hash`](check-has-key.md) | `bytes` | Get `keccak256` hash of address's key (public) |
| `compute_key_hash` | `bytes` | Compute `keccak256(viewing_key)` locally (pure) |
| [`watch_src20_events_with_key`](watch-src20-events-with-key.md) | `SRC20EventWatcher` | Watch events using an explicit viewing key |

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

- [Event Watching](../event-watching/) — Watch and decrypt SRC20 events
- [Types](../types/) — Decrypted event data structures
- [Directory ABI](../../abis/directory.md) — Low-level ABI and contract address
