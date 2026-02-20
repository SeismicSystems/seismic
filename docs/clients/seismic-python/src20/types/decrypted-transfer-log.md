---
description: Decoded SRC20 Transfer event with decrypted amount
icon: arrow-right
---

# DecryptedTransferLog

Frozen dataclass representing a decoded SRC20 Transfer event with decrypted amount.

## Overview

`DecryptedTransferLog` contains all data from an SRC20 Transfer event, including the decrypted amount. It is returned by event watcher callbacks when a Transfer event is detected and successfully decrypted.

SRC20 Transfer events have the signature:
```solidity
event Transfer(
    address indexed from,
    address indexed to,
    bytes32 indexed encryptKeyHash,
    bytes encryptedAmount
)
```

## Definition

```python
@dataclass(frozen=True)
class DecryptedTransferLog:
    """Decoded SRC20 Transfer event with decrypted amount."""

    from_address: ChecksumAddress
    to_address: ChecksumAddress
    encrypt_key_hash: bytes
    encrypted_amount: bytes
    decrypted_amount: int
    transaction_hash: HexBytes
    block_number: int
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `from_address` | `ChecksumAddress` | Address sending the tokens (checksummed) |
| `to_address` | `ChecksumAddress` | Address receiving the tokens (checksummed) |
| `encrypt_key_hash` | `bytes` | 32-byte `keccak256` hash of the viewing key |
| `encrypted_amount` | `bytes` | Raw encrypted amount bytes (AES-256-GCM ciphertext) |
| `decrypted_amount` | `int` | Decrypted token amount as integer |
| `transaction_hash` | `HexBytes` | Transaction hash containing this event |
| `block_number` | `int` | Block number containing this event |

## Properties

- **Immutable** - All fields are frozen after creation
- **Type-safe** - Uses proper types (`ChecksumAddress`, `HexBytes`)
- **Complete** - Contains both encrypted and decrypted data

## Examples

### Basic Callback Usage

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import watch_src20_events, DecryptedTransferLog

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

def on_transfer(log: DecryptedTransferLog):
    print(f"Transfer from {log.from_address} to {log.to_address}")
    print(f"Amount: {log.decrypted_amount}")
    print(f"Block: {log.block_number}")
    print(f"Tx: {log.transaction_hash.hex()}")

watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    on_transfer=on_transfer,
)
```

### Access All Fields

```python
from seismic_web3.src20 import DecryptedTransferLog

def on_transfer(log: DecryptedTransferLog):
    # Address fields (checksummed)
    print(f"From: {log.from_address}")
    print(f"To: {log.to_address}")

    # Amount fields
    print(f"Encrypted: {log.encrypted_amount.hex()}")
    print(f"Decrypted: {log.decrypted_amount}")

    # Metadata
    print(f"Key hash: {log.encrypt_key_hash.hex()}")
    print(f"Tx: {log.transaction_hash.hex()}")
    print(f"Block: {log.block_number}")
```

### Filter by Address

```python
from seismic_web3.src20 import DecryptedTransferLog

MY_ADDRESS = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"

def on_transfer(log: DecryptedTransferLog):
    if log.from_address == MY_ADDRESS:
        print(f"Sent {log.decrypted_amount} to {log.to_address}")
    elif log.to_address == MY_ADDRESS:
        print(f"Received {log.decrypted_amount} from {log.from_address}")
```

### Store to Database

```python
from seismic_web3.src20 import DecryptedTransferLog
import sqlite3

def on_transfer(log: DecryptedTransferLog):
    conn = sqlite3.connect("events.db")
    conn.execute(
        """
        INSERT INTO transfers
        (from_address, to_address, amount, tx_hash, block_number)
        VALUES (?, ?, ?, ?, ?)
        """,
        (
            log.from_address,
            log.to_address,
            log.decrypted_amount,
            log.transaction_hash.hex(),
            log.block_number,
        ),
    )
    conn.commit()
    conn.close()
```

### Async Callback

```python
from seismic_web3.src20 import DecryptedTransferLog
import asyncio

async def on_transfer(log: DecryptedTransferLog):
    print(f"Transfer: {log.decrypted_amount}")

    # Perform async operations
    await save_to_database(log)
    await send_notification(log)
    await update_balance(log.to_address, log.decrypted_amount)
```

### Format Amount

```python
from seismic_web3.src20 import DecryptedTransferLog

def format_token_amount(amount: int, decimals: int = 18) -> str:
    """Format token amount with decimals."""
    return f"{amount / 10 ** decimals:.{decimals}f}"

def on_transfer(log: DecryptedTransferLog):
    # Assume 18 decimals (like most ERC20 tokens)
    formatted = format_token_amount(log.decrypted_amount, decimals=18)
    print(f"Transfer: {formatted} tokens")
```

### Track Balance Changes

```python
from seismic_web3.src20 import DecryptedTransferLog
from collections import defaultdict

balances = defaultdict(int)

def on_transfer(log: DecryptedTransferLog):
    # Update balances
    balances[log.from_address] -= log.decrypted_amount
    balances[log.to_address] += log.decrypted_amount

    print(f"Balance {log.from_address}: {balances[log.from_address]}")
    print(f"Balance {log.to_address}: {balances[log.to_address]}")
```

### Wait for Transaction Confirmation

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import DecryptedTransferLog

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

def on_transfer(log: DecryptedTransferLog):
    # Get full transaction receipt
    receipt = w3.eth.get_transaction_receipt(log.transaction_hash)
    print(f"Status: {receipt['status']}")
    print(f"Gas used: {receipt['gasUsed']}")
```

### Immutability

```python
from seismic_web3.src20 import DecryptedTransferLog

def on_transfer(log: DecryptedTransferLog):
    # This will raise AttributeError (frozen dataclass)
    try:
        log.decrypted_amount = 1000
    except AttributeError as e:
        print(f"Cannot modify: {e}")
```

## Notes

- All instances are immutable (frozen dataclass)
- `from_address` and `to_address` are checksummed addresses
- `decrypted_amount` is the raw token amount (consider decimals when displaying)
- `encrypted_amount` includes the AES-GCM authentication tag
- `encrypt_key_hash` matches the key hash in event topics
- Created automatically by event watchers during decryption

## See Also

- [DecryptedApprovalLog](decrypted-approval-log.md) - Approval event data
- [watch_src20_events](../event-watching/watch-src20-events.md) - Watch with automatic key fetching
- [watch_src20_events_with_key](../event-watching/watch-src20-events-with-key.md) - Watch with explicit key
- [SRC20EventWatcher](../event-watching/src20-event-watcher.md) - Sync watcher class
- [AsyncSRC20EventWatcher](../event-watching/async-src20-event-watcher.md) - Async watcher class
