---
description: Check if an address has a registered viewing key
icon: check
---

# check_has_key

Check whether an address has a registered viewing key in the Directory contract.

## Overview

`check_has_key()` and `async_check_has_key()` query the Directory contract to check if a specific address has registered a viewing key. This is a public, read-only operation that requires no authentication.

Also documented here: `get_key_hash()` and `async_get_key_hash()` retrieve the keccak256 hash of an address's viewing key.

## Signatures

```python
def check_has_key(
    w3: Web3,
    address: ChecksumAddress,
) -> bool

async def async_check_has_key(
    w3: AsyncWeb3,
    address: ChecksumAddress,
) -> bool

def get_key_hash(
    w3: Web3,
    address: ChecksumAddress,
) -> bytes

async def async_get_key_hash(
    w3: AsyncWeb3,
    address: ChecksumAddress,
) -> bytes
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `w3` | `Web3` or `AsyncWeb3` | Yes | Standard Web3 instance (no Seismic namespace required) |
| `address` | `ChecksumAddress` | Yes | Ethereum address to check |

## Returns

### check_has_key / async_check_has_key

| Type | Description |
|------|-------------|
| `bool` | `True` if the address has a registered key, `False` otherwise |

### get_key_hash / async_get_key_hash

| Type | Description |
|------|-------------|
| `bytes` | 32-byte `keccak256` hash of the viewing key |

## Examples

### Check Has Key (Sync)

```python
from web3 import Web3
from seismic_web3.src20 import check_has_key

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"

if check_has_key(w3, address):
    print(f"Address {address} has a registered viewing key")
else:
    print(f"Address {address} has no viewing key")
```

### Check Before Registering

```python
from seismic_web3 import create_wallet_client, PrivateKey, Bytes32
from seismic_web3.src20 import check_has_key, register_viewing_key
from eth_account import Account
import os

private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Get your address
address = Account.from_key(private_key).address

# Check if key already exists
if check_has_key(w3, address):
    print("Key already registered")
else:
    print("Registering new key...")
    viewing_key = Bytes32(os.urandom(32))
    tx_hash = register_viewing_key(
        w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
        key=viewing_key,
    )
    w3.eth.wait_for_transaction_receipt(tx_hash)
    print("Key registered successfully")
```

### Get Key Hash (Sync)

```python
from web3 import Web3
from seismic_web3.src20 import get_key_hash

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"

key_hash = get_key_hash(w3, address)
print(f"Key hash: {key_hash.hex()}")
```

### Compare Key Hash

```python
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20 import get_key_hash
from seismic_web3.src20.directory import compute_key_hash

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"

# Get hash from chain
onchain_hash = get_key_hash(w3, address)

# Compare with local key hash
local_key = Bytes32(bytes.fromhex("YOUR_VIEWING_KEY_HEX"))
local_hash = compute_key_hash(local_key)

if onchain_hash == local_hash:
    print("Key matches!")
else:
    print("Key does not match")
```

### Async Usage

```python
from web3 import AsyncWeb3
from seismic_web3.src20 import async_check_has_key, async_get_key_hash
import asyncio

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))
    address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"

    # Check if key exists
    has_key = await async_check_has_key(w3, address)
    print(f"Has key: {has_key}")

    if has_key:
        # Get the key hash
        key_hash = await async_get_key_hash(w3, address)
        print(f"Key hash: {key_hash.hex()}")

asyncio.run(main())
```

### Batch Check Multiple Addresses

```python
from web3 import Web3
from seismic_web3.src20 import check_has_key

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

addresses = [
    "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B",
    "0x5FbDB2315678afecb367f032d93F642f64180aa3",
]

for addr in addresses:
    has_key = check_has_key(w3, addr)
    status = "registered" if has_key else "not registered"
    print(f"{addr}: {status}")
```

### Monitor Registration

```python
from web3 import Web3
from seismic_web3.src20 import check_has_key
import time

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"

print(f"Waiting for {address} to register a key...")
while not check_has_key(w3, address):
    time.sleep(2)

print("Key registered!")
```

### Validate Before Event Watching

```python
from web3 import Web3
from seismic_web3 import Bytes32
from seismic_web3.src20 import check_has_key, watch_src20_events_with_key
from eth_account import Account

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
viewing_key = Bytes32(bytes.fromhex("YOUR_VIEWING_KEY_HEX"))

# Compute expected address from key (if known)
# For this example, we'll check a known address
address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"

if not check_has_key(w3, address):
    print("ERROR: No viewing key registered for this address")
else:
    # Start watching
    watcher = watch_src20_events_with_key(
        w3,
        viewing_key=viewing_key,
        on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
    )
```

### Get Key Hash for Event Filtering

```python
from web3 import Web3
from seismic_web3.src20 import get_key_hash

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"

# Get the key hash (used as event topic)
key_hash = get_key_hash(w3, address)

# Use for manual event filtering
filter_params = {
    "topics": [
        "0x...",  # Transfer event signature
        None,
        None,
        key_hash.hex(),  # Filter by key hash
    ]
}
logs = w3.eth.get_logs(filter_params)
```

## How It Works

### check_has_key

1. **Encode calldata** - Encodes `checkHasKey(address)` with the target address
2. **Call contract** - Makes a plain `eth_call` to Directory at `0x1000...0004`
3. **Decode result** - Decodes the boolean result from ABI

### get_key_hash

1. **Encode calldata** - Encodes `keyHash(address)` with the target address
2. **Call contract** - Makes a plain `eth_call` to Directory at `0x1000...0004`
3. **Decode result** - Decodes the 32-byte hash from ABI

## Notes

- Both functions are read-only (no gas costs)
- No authentication required (publicly accessible)
- Works with standard `Web3` instances (no Seismic namespace needed)
- The key hash is `keccak256(viewing_key)`
- Used as the 4th topic in SRC20 Transfer and Approval events
- The hash allows event filtering without exposing the key

## Use Cases

### check_has_key

- Verify an address has registered a viewing key before attempting to fetch it
- Check if registration is complete after sending registration transaction
- Validate addresses in batch operations
- Monitor for new registrations

### get_key_hash

- Manual event filtering with `eth_getLogs`
- Compare on-chain key with locally stored key (via hash)
- Verify event watcher is filtering for the correct key
- Debug event watching issues

## Warnings

- **Public information** - Anyone can check if an address has a key
- **No key retrieval** - These functions do not return the viewing key itself
- **Hash only** - `get_key_hash` returns the hash, not the key
- **Zero hash** - If no key is registered, `get_key_hash` returns 32 zero bytes

## See Also

- [register_viewing_key](register-viewing-key.md) - Register a viewing key
- [get_viewing_key](get-viewing-key.md) - Fetch your viewing key (authenticated)
- [watch_src20_events](../event-watching/watch-src20-events.md) - Watch events with automatic key fetching
- [compute_key_hash](register-viewing-key.md) - Compute hash from viewing key
