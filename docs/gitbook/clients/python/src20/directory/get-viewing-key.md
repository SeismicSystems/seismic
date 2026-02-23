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

- Uses signed read (`getKey`) so `msg.sender` is authenticated.
- Returns `Bytes32`.
- Raises `ValueError` if no key is registered.

## Example

```python
viewing_key = get_viewing_key(w3, w3.seismic.encryption, pk)
print(viewing_key.hex())
```
