---
description: Fetch your viewing key from the Directory contract
icon: download
---

# get_viewing_key

Fetch your viewing key from the Directory genesis contract using signed read authentication.

## Signatures

```python
def get_viewing_key(
    w3: Web3,
    encryption: EncryptionState,
    private_key: PrivateKey,
) -> Bytes32

async def async_get_viewing_key(
    w3: AsyncWeb3,
    encryption: EncryptionState,
    private_key: PrivateKey,
) -> Bytes32
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `w3` | `Web3` / `AsyncWeb3` | Web3 instance with Seismic support |
| `encryption` | [`EncryptionState`](../../client/encryption-state.md) | Encryption state from wallet client |
| `private_key` | [`PrivateKey`](../../api-reference/types/private-key.md) | Signing key for authentication |

## Returns

[`Bytes32`](../../api-reference/types/bytes32.md) — 32-byte AES-256 viewing key.

Raises `ValueError` if no viewing key is registered for the caller's address.

## Example

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET
from seismic_web3.src20 import get_viewing_key

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

viewing_key = get_viewing_key(w3, w3.seismic.encryption, pk)
print(f"Viewing key: {viewing_key.hex()}")
```

## Notes

- Uses a **signed read** (`getKey()`) so that `msg.sender` is authenticated — only the key owner can retrieve it
- Typically called internally by [`watch_src20_events()`](../event-watching/watch-src20-events.md) to fetch the key before polling

## See Also

- [register_viewing_key](register-viewing-key.md) — Register a viewing key
- [check_has_key](check-has-key.md) — Check if an address has a key (public, no auth)
- [watch_src20_events](../event-watching/watch-src20-events.md) — Calls this internally
