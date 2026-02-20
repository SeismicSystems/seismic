---
description: Register a viewing key in the Directory contract
icon: key
---

# register_viewing_key

Register a viewing key in the Directory genesis contract for SRC20 event decryption.

## Overview

`register_viewing_key()` and `async_register_viewing_key()` register a 32-byte AES-256 viewing key in the Directory contract at `0x1000...0004`. This key is used to decrypt SRC20 Transfer and Approval events.

The function uses a **shielded write** (`setKey(suint256)`) to securely store the key on-chain. Once registered, the key can be retrieved via [`get_viewing_key()`](get-viewing-key.md) or used directly with event watchers.

## Signature

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

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `w3` | `Web3` or `AsyncWeb3` | Yes | Web3 instance with Seismic support |
| `encryption` | [`EncryptionState`](../../client/encryption-state.md) | Yes | Encryption state from wallet client |
| `private_key` | [`PrivateKey`](../../api-reference/types/private-key.md) | Yes | 32-byte signing key for the transaction |
| `key` | [`Bytes32`](../../api-reference/types/bytes32.md) | Yes | 32-byte AES-256 viewing key to register |

## Returns

| Type | Description |
|------|-------------|
| `HexBytes` | Transaction hash |

## Examples

### Basic Usage (Sync)

```python
from seismic_web3 import create_wallet_client, PrivateKey, Bytes32
from seismic_web3.src20 import register_viewing_key
import os

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Generate a random viewing key
viewing_key = Bytes32(os.urandom(32))

# Register the key
tx_hash = register_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    key=viewing_key,
)

print(f"Transaction hash: {tx_hash.hex()}")

# Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
print(f"Registered in block {receipt['blockNumber']}")
```

### With Fixed Key

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, Bytes32
from seismic_web3.src20 import register_viewing_key

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Use a specific viewing key (e.g., derived from seed)
viewing_key = Bytes32(bytes.fromhex(os.environ["VIEWING_KEY"].removeprefix("0x")))

tx_hash = register_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    key=viewing_key,
)

print(f"Registered key: {viewing_key.hex()}")
```

### Store Key Securely

```python
from seismic_web3 import create_wallet_client, PrivateKey, Bytes32
from seismic_web3.src20 import register_viewing_key
import os

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Generate and save the viewing key
viewing_key = Bytes32(os.urandom(32))

# Save to secure storage (e.g., encrypted file, key vault)
save_to_secure_storage(viewing_key)

# Register on-chain
tx_hash = register_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    key=viewing_key,
)

# Wait for confirmation
w3.eth.wait_for_transaction_receipt(tx_hash)
print("Key registered successfully")
```

### Async Usage

```python
from seismic_web3 import create_async_wallet_client, PrivateKey, Bytes32
from seismic_web3.src20 import async_register_viewing_key
import asyncio
import os

async def main():
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
    w3 = await create_async_wallet_client(
        "wss://gcp-1.seismictest.net/ws",
        private_key=private_key,
    )

    # Generate a random viewing key
    viewing_key = Bytes32(os.urandom(32))

    # Register the key
    tx_hash = await async_register_viewing_key(
        w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
        key=viewing_key,
    )

    print(f"Transaction hash: {tx_hash.hex()}")

    # Wait for confirmation
    receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
    print(f"Registered in block {receipt['blockNumber']}")

asyncio.run(main())
```

### Check Before Registering

```python
from seismic_web3 import create_wallet_client, PrivateKey, Bytes32
from seismic_web3.src20 import register_viewing_key, check_has_key
import os

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Get wallet address
from eth_account import Account
address = Account.from_key(private_key).address

# Check if key already exists
if check_has_key(w3, address):
    print("Key already registered")
else:
    # Generate and register new key
    viewing_key = Bytes32(os.urandom(32))
    tx_hash = register_viewing_key(
        w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
        key=viewing_key,
    )
    print(f"Registered new key: {tx_hash.hex()}")
```

### Derive from Seed

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, Bytes32
from seismic_web3.src20 import register_viewing_key
import hashlib

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Derive viewing key from seed phrase
seed = "my secret seed phrase for viewing key"
viewing_key = Bytes32(hashlib.sha256(seed.encode()).digest())

tx_hash = register_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    key=viewing_key,
)

print(f"Registered derived key: {tx_hash.hex()}")
```

## How It Works

1. **Encode calldata** - Encodes `setKey(suint256)` with the viewing key as an unsigned 256-bit integer
2. **Shielded write** - Calls [`send_shielded_transaction()`](../../api-reference/transaction-types/seismic-elements.md) to the Directory contract at `0x1000...0004`
3. **Transaction** - The transaction is signed, encrypted, and sent to the node
4. **Storage** - The key is stored on-chain and mapped to the sender's address

## Gas Cost

- Base cost: ~21,000 gas (standard transaction)
- Storage cost: ~20,000 gas (first-time registration)
- Total: ~41,000 gas for first registration
- Update cost is similar to initial registration

## Notes

- The key is stored per address (sender's address)
- Registering a new key overwrites the previous one
- The key is stored encrypted on-chain
- Only the key owner can retrieve it via signed read
- Anyone can check if an address has a key via [`check_has_key()`](check-has-key.md)
- The key hash can be queried publicly via [`get_key_hash()`](check-has-key.md)

## Warnings

- **Key security** - Keep viewing keys secure; they allow decrypting all SRC20 events
- **No recovery** - If you lose the viewing key, you cannot decrypt past events
- **Overwriting** - Registering a new key overwrites the previous one permanently
- **Gas costs** - Requires a transaction with gas fees
- **Confirmation** - Wait for transaction confirmation before using the key

## See Also

- [get_viewing_key](get-viewing-key.md) - Fetch your viewing key from Directory
- [check_has_key](check-has-key.md) - Check if an address has a registered key
- [watch_src20_events](../event-watching/watch-src20-events.md) - Watch events with your key
- [Bytes32](../../api-reference/types/bytes32.md) - 32-byte value type
- [EncryptionState](../../client/encryption-state.md) - Encryption state
