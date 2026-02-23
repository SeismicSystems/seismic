---
description: Fetch caller viewing key from Directory
icon: key
---

# get_viewing_key

Fetch the caller's 32-byte viewing key from Directory.

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

## Behavior

- Performs a signed read to `getKey` so `msg.sender` is authenticated.
- Raises `ValueError` if the key is missing/empty/zero.
- Returns the last 32 bytes of the signed-read result as `Bytes32`.
