---
description: Query whether an address has a Directory viewing key
icon: search
---

# check_has_key and get_key_hash

Public read helpers for key registration state.

## Signatures

```python
def check_has_key(w3: Web3, address: ChecksumAddress) -> bool
async def async_check_has_key(w3: AsyncWeb3, address: ChecksumAddress) -> bool

def get_key_hash(w3: Web3, address: ChecksumAddress) -> bytes
async def async_get_key_hash(w3: AsyncWeb3, address: ChecksumAddress) -> bytes
```

## Example

```python
from seismic_web3.src20 import check_has_key, get_key_hash

address = "0xYourAddress"
print(check_has_key(w3, address))
print(get_key_hash(w3, address).hex())
```

## Notes

- Uses plain `eth_call` (no private key required).
- `get_key_hash` returns on-chain `bytes32` key hash.
