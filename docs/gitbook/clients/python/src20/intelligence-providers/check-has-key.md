---
description: Check if an address has a registered viewing key
icon: search
---

# check_has_key / get_key_hash

Public, read-only queries against the Directory contract. No authentication required — works with a standard `Web3` instance.

## Signatures

```python
# check_has_key
def check_has_key(w3: Web3, address: ChecksumAddress) -> bool
async def async_check_has_key(w3: AsyncWeb3, address: ChecksumAddress) -> bool

# get_key_hash
def get_key_hash(w3: Web3, address: ChecksumAddress) -> bytes
async def async_get_key_hash(w3: AsyncWeb3, address: ChecksumAddress) -> bytes
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `w3` | `Web3` / `AsyncWeb3` | Standard Web3 instance (no Seismic namespace required) |
| `address` | `ChecksumAddress` | Address to query |

## Returns

- **`check_has_key`** — `True` if the address has a registered viewing key
- **`get_key_hash`** — 32-byte `keccak256` hash of the viewing key (zero bytes if no key registered)

## Example

```python
from web3 import Web3
from seismic_web3.src20 import check_has_key, get_key_hash

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

if check_has_key(w3, address):
    key_hash = get_key_hash(w3, address)
    print(f"Key hash: {key_hash.hex()}")
```

## Notes

- Both use plain `eth_call` — no gas cost, no encryption needed
- The key hash is used as the 4th topic in SRC20 Transfer and Approval events for filtering

## See Also

- [register_viewing_key](register-viewing-key.md) — Register a viewing key
- [get_viewing_key](get-viewing-key.md) — Fetch your viewing key (authenticated)
