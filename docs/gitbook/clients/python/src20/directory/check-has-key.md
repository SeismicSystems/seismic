---
description: Query whether an address has a Directory viewing key
icon: search
---

# check_has_key and get_key_hash

Public read helpers for Directory key registration status.

## Signatures

```python
def check_has_key(w3: Web3, address: ChecksumAddress) -> bool
async def async_check_has_key(w3: AsyncWeb3, address: ChecksumAddress) -> bool

def get_key_hash(w3: Web3, address: ChecksumAddress) -> bytes
async def async_get_key_hash(w3: AsyncWeb3, address: ChecksumAddress) -> bytes
```

## Behavior

- Both helpers use plain `eth_call` (no private key/encryption needed).
- `check_has_key` returns a decoded boolean.
- `get_key_hash` returns the on-chain `bytes32` key hash for that address.
