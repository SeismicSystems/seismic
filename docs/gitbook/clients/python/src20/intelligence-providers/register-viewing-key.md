---
description: Register a viewing key in the Directory contract
icon: key
---

# register_viewing_key

Register a 32-byte AES-256 viewing key in the Directory genesis contract for SRC20 event decryption.

## Signatures

```python
def register_viewing_key(
    w3: Web3,
    encryption: EncryptionState,
    private_key: PrivateKey,
    key: Bytes32,
) -> HexBytes

async def async_register_viewing_key(
    w3: AsyncWeb3,
    encryption: EncryptionState,
    private_key: PrivateKey,
    key: Bytes32,
) -> HexBytes
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `w3` | `Web3` / `AsyncWeb3` | Web3 instance with Seismic support |
| `encryption` | [`EncryptionState`](../../client/encryption-state.md) | Encryption state from wallet client |
| `private_key` | [`PrivateKey`](../../api-reference/types/private-key.md) | Signing key for the transaction |
| `key` | [`Bytes32`](../../api-reference/types/bytes32.md) | Viewing key to register |

## Returns

`HexBytes` — transaction hash.

## Example

```python
import os
from seismic_web3 import PrivateKey, Bytes32, SEISMIC_TESTNET
from seismic_web3.src20 import register_viewing_key

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

viewing_key = Bytes32(os.urandom(32))
tx_hash = register_viewing_key(w3, w3.seismic.encryption, pk, key=viewing_key)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
print(f"Registered in block {receipt['blockNumber']}")
```

## Notes

- Uses a **shielded write** (`setKey(suint256)`) — the key is encrypted in transit
- Registering a new key overwrites the previous one
- The key is stored per `msg.sender` address
- `compute_key_hash(key)` returns the `keccak256` hash used for event topic filtering

## See Also

- [get_viewing_key](get-viewing-key.md) — Fetch your viewing key back
- [check_has_key](check-has-key.md) — Check if an address has a registered key
