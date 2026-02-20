---
description: Manage viewing keys in the Directory genesis contract
icon: book
---

# Directory

Manage viewing keys stored in the Directory genesis contract at `0x1000...0004`.

## Overview

The Directory is a genesis contract that stores per-address AES-256 viewing keys used to decrypt SRC20 Transfer and Approval events. It provides:

- **Key registration** - Store your viewing key on-chain via shielded write
- **Key retrieval** - Fetch your key via authenticated signed read
- **Public queries** - Check if addresses have registered keys (no authentication)

## Quick Start

### Register a Viewing Key

```python
from seismic_web3 import create_wallet_client, PrivateKey, Bytes32
from seismic_web3.src20 import register_viewing_key
import os

private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

viewing_key = Bytes32(os.urandom(32))
tx_hash = register_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    key=viewing_key,
)

w3.eth.wait_for_transaction_receipt(tx_hash)
print("Key registered")
```

### Fetch Your Viewing Key

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import get_viewing_key

private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

viewing_key = get_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
)

print(f"Your key: {viewing_key.hex()}")
```

### Check if Address Has Key

```python
from web3 import Web3
from seismic_web3.src20 import check_has_key

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))
address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"

if check_has_key(w3, address):
    print("Address has a registered key")
```

## Functions

### Key Management

| Function | Description |
|----------|-------------|
| [register_viewing_key](register-viewing-key.md) | Register a viewing key (shielded write) |
| [get_viewing_key](get-viewing-key.md) | Fetch your viewing key (signed read) |

### Public Queries

| Function | Description |
|----------|-------------|
| [check_has_key](check-has-key.md) | Check if address has a key (public) |
| [get_key_hash](check-has-key.md) | Get keccak256 hash of address's key (public) |

### Pure Helper

| Function | Description |
|----------|-------------|
| `compute_key_hash` | Compute `keccak256(viewing_key)` locally |

## Directory Contract

- **Address:** `0x1000000000000000000000000000000000000004`
- **Type:** Genesis contract (pre-deployed)
- **Methods:**
  - `setKey(suint256)` - Register viewing key (shielded write)
  - `getKey()` - Fetch your key (signed read)
  - `checkHasKey(address)` - Check if address has key (public read)
  - `keyHash(address)` - Get key hash (public read)

## Key Properties

### Storage

- Keys are stored per address (sender's address)
- Keys are encrypted on-chain
- Only the owner can retrieve their key via signed read

### Key Hash

- Event topics use `keccak256(viewing_key)` for filtering
- The hash is publicly visible but does not expose the key
- Used by event watchers to filter for relevant events

### Security

- Viewing keys allow decrypting all SRC20 events for that address
- Keep viewing keys secure (similar to private keys)
- Losing the viewing key means you cannot decrypt past events
- Keys can be overwritten by registering a new one

## Common Workflows

### First-Time Setup

```python
from seismic_web3 import create_wallet_client, PrivateKey, Bytes32
from seismic_web3.src20 import check_has_key, register_viewing_key
from eth_account import Account
import os

private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

address = Account.from_key(private_key).address

if not check_has_key(w3, address):
    viewing_key = Bytes32(os.urandom(32))
    tx_hash = register_viewing_key(
        w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
        key=viewing_key,
    )
    w3.eth.wait_for_transaction_receipt(tx_hash)

    # Save key securely
    save_to_secure_storage(viewing_key)
```

### Key Rotation

```python
from seismic_web3 import create_wallet_client, PrivateKey, Bytes32
from seismic_web3.src20 import get_viewing_key, register_viewing_key
import os

private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Get old key
old_key = get_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
)

# Register new key
new_key = Bytes32(os.urandom(32))
tx_hash = register_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    key=new_key,
)

w3.eth.wait_for_transaction_receipt(tx_hash)
print(f"Old key: {old_key.hex()}")
print(f"New key: {new_key.hex()}")
```

### Async Operations

All functions have async variants:

```python
from seismic_web3 import create_async_wallet_client, PrivateKey, Bytes32
from seismic_web3.src20 import async_register_viewing_key, async_get_viewing_key
import asyncio
import os

async def main():
    private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
    w3 = await create_async_wallet_client(
        "wss://gcp-1.seismictest.net/ws",
        private_key=private_key,
    )

    # Register
    viewing_key = Bytes32(os.urandom(32))
    tx_hash = await async_register_viewing_key(
        w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
        key=viewing_key,
    )

    # Fetch
    fetched_key = await async_get_viewing_key(
        w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
    )

asyncio.run(main())
```

## Gas Costs

| Operation | Gas Cost (approx) |
|-----------|------------------|
| Register key (first time) | ~41,000 gas |
| Register key (update) | ~41,000 gas |
| Fetch key (signed read) | Free (off-chain) |
| Check has key | Free (off-chain) |
| Get key hash | Free (off-chain) |

## Notes

- Viewing keys are separate from wallet private keys
- Keys can be derived from seed phrases or generated randomly
- Consider backing up viewing keys to secure storage
- Share viewing keys cautiously (they allow decrypting all events)

## See Also

- [Event Watching](../event-watching/README.md) - Watch and decrypt SRC20 events
- [Types](../types/README.md) - Event data structures
- [SRC20 Overview](../README.md) - Full SRC20 documentation
