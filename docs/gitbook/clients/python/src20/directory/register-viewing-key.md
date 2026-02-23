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

- Uses a shielded write to call `setKey(suint256)`.
- Encodes `key` as big-endian integer for calldata.
- Returns transaction hash.
- `compute_key_hash` returns `keccak256(aes_key)` for topic filtering.
