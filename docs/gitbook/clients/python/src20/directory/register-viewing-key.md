---
description: Register viewing key in Directory
icon: key
---

# register_viewing_key

Register a 32-byte AES viewing key in Directory.

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

def compute_key_hash(aes_key: Bytes32) -> bytes
```

## Behavior

- Sends shielded write to `setKey(suint256)`.
- Returns transaction hash.
- `compute_key_hash` is local `keccak256(aes_key)` helper.

## Example

```python
import os
from seismic_web3 import Bytes32
from seismic_web3.src20 import register_viewing_key

key = Bytes32(os.urandom(32))
tx_hash = register_viewing_key(w3, w3.seismic.encryption, pk, key)
w3.eth.wait_for_transaction_receipt(tx_hash)
```
