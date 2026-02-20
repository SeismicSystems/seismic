---
description: Decoded SRC20 Approval event with decrypted amount
icon: check-circle
---

# DecryptedApprovalLog

Frozen dataclass representing a decoded SRC20 Approval event with decrypted amount.

## Overview

`DecryptedApprovalLog` contains all data from an SRC20 Approval event, including the decrypted amount. It is returned by event watcher callbacks when an Approval event is detected and successfully decrypted.

SRC20 Approval events have the signature:
```solidity
event Approval(
    address indexed owner,
    address indexed spender,
    bytes32 indexed encryptKeyHash,
    bytes encryptedAmount
)
```

## Definition

```python
@dataclass(frozen=True)
class DecryptedApprovalLog:
    """Decoded SRC20 Approval event with decrypted amount."""

    owner: ChecksumAddress
    spender: ChecksumAddress
    encrypt_key_hash: bytes
    encrypted_amount: bytes
    decrypted_amount: int
    transaction_hash: HexBytes
    block_number: int
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `owner` | `ChecksumAddress` | Address granting the approval (checksummed) |
| `spender` | `ChecksumAddress` | Address receiving the approval (checksummed) |
| `encrypt_key_hash` | `bytes` | 32-byte `keccak256` hash of the viewing key |
| `encrypted_amount` | `bytes` | Raw encrypted amount bytes (AES-256-GCM ciphertext) |
| `decrypted_amount` | `int` | Decrypted approval amount as integer |
| `transaction_hash` | `HexBytes` | Transaction hash containing this event |
| `block_number` | `int` | Block number containing this event |

## Properties

- **Immutable** - All fields are frozen after creation
- **Type-safe** - Uses proper types (`ChecksumAddress`, `HexBytes`)
- **Complete** - Contains both encrypted and decrypted data

## Examples

### Basic Callback Usage

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import watch_src20_events, DecryptedApprovalLog

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

def on_approval(log: DecryptedApprovalLog):
    print(f"Approval from {log.owner} to {log.spender}")
    print(f"Amount: {log.decrypted_amount}")
    print(f"Block: {log.block_number}")
    print(f"Tx: {log.transaction_hash.hex()}")

watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    on_approval=on_approval,
)
```

### Access All Fields

```python
from seismic_web3.src20 import DecryptedApprovalLog

def on_approval(log: DecryptedApprovalLog):
    # Address fields (checksummed)
    print(f"Owner: {log.owner}")
    print(f"Spender: {log.spender}")

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
from seismic_web3.src20 import DecryptedApprovalLog

MY_ADDRESS = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

def on_approval(log: DecryptedApprovalLog):
    if log.owner == MY_ADDRESS:
        print(f"Granted approval of {log.decrypted_amount} to {log.spender}")
    elif log.spender == MY_ADDRESS:
        print(f"Received approval of {log.decrypted_amount} from {log.owner}")
```

### Store to Database

```python
from seismic_web3.src20 import DecryptedApprovalLog
import sqlite3

def on_approval(log: DecryptedApprovalLog):
    conn = sqlite3.connect("events.db")
    conn.execute(
        """
        INSERT INTO approvals
        (owner, spender, amount, tx_hash, block_number)
        VALUES (?, ?, ?, ?, ?)
        """,
        (
            log.owner,
            log.spender,
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
from seismic_web3.src20 import DecryptedApprovalLog
import asyncio

async def on_approval(log: DecryptedApprovalLog):
    print(f"Approval: {log.decrypted_amount}")

    # Perform async operations
    await save_to_database(log)
    await send_notification(log)
    await update_allowance(log.owner, log.spender, log.decrypted_amount)
```

### Format Amount

```python
from seismic_web3.src20 import DecryptedApprovalLog

def format_token_amount(amount: int, decimals: int = 18) -> str:
    """Format token amount with decimals."""
    return f"{amount / 10 ** decimals:.{decimals}f}"

def on_approval(log: DecryptedApprovalLog):
    # Assume 18 decimals (like most ERC20 tokens)
    formatted = format_token_amount(log.decrypted_amount, decimals=18)
    print(f"Approval: {formatted} tokens")
```

### Track Allowances

```python
from seismic_web3.src20 import DecryptedApprovalLog
from collections import defaultdict

# allowances[owner][spender] = amount
allowances = defaultdict(lambda: defaultdict(int))

def on_approval(log: DecryptedApprovalLog):
    # Update allowance
    allowances[log.owner][log.spender] = log.decrypted_amount

    print(f"Allowance {log.owner} -> {log.spender}: {log.decrypted_amount}")
```

### Watch for Specific Spender

```python
from seismic_web3.src20 import DecryptedApprovalLog

ROUTER_ADDRESS = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"

def on_approval(log: DecryptedApprovalLog):
    if log.spender == ROUTER_ADDRESS:
        print(f"Router approved for {log.decrypted_amount} by {log.owner}")
        # Send notification, etc.
```

### Wait for Transaction Confirmation

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import DecryptedApprovalLog

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

def on_approval(log: DecryptedApprovalLog):
    # Get full transaction receipt
    receipt = w3.eth.get_transaction_receipt(log.transaction_hash)
    print(f"Status: {receipt['status']}")
    print(f"Gas used: {receipt['gasUsed']}")
```

### Combine with Transfer Events

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import (
    watch_src20_events,
    DecryptedTransferLog,
    DecryptedApprovalLog,
)

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

def on_transfer(log: DecryptedTransferLog):
    print(f"Transfer: {log.decrypted_amount} from {log.from_address}")

def on_approval(log: DecryptedApprovalLog):
    print(f"Approval: {log.decrypted_amount} to {log.spender}")

watcher = watch_src20_events(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    on_transfer=on_transfer,
    on_approval=on_approval,
)
```

### Immutability

```python
from seismic_web3.src20 import DecryptedApprovalLog

def on_approval(log: DecryptedApprovalLog):
    # This will raise AttributeError (frozen dataclass)
    try:
        log.decrypted_amount = 1000
    except AttributeError as e:
        print(f"Cannot modify: {e}")
```

### Check for Infinite Approval

```python
from seismic_web3.src20 import DecryptedApprovalLog

MAX_UINT256 = 2**256 - 1

def on_approval(log: DecryptedApprovalLog):
    if log.decrypted_amount == MAX_UINT256:
        print(f"WARNING: Infinite approval granted to {log.spender}")
    else:
        print(f"Approval: {log.decrypted_amount}")
```

## Notes

- All instances are immutable (frozen dataclass)
- `owner` is the address granting the approval
- `spender` is the address receiving the approval (can spend on behalf of owner)
- `decrypted_amount` is the raw token amount (consider decimals when displaying)
- `encrypted_amount` includes the AES-GCM authentication tag
- `encrypt_key_hash` matches the key hash in event topics
- Created automatically by event watchers during decryption

## Common Use Cases

- Track allowances for DeFi protocols
- Monitor approvals to router contracts
- Detect infinite approvals (security risk)
- Audit approval history for compliance
- Trigger alerts for large approvals

## See Also

- [DecryptedTransferLog](decrypted-transfer-log.md) - Transfer event data
- [watch_src20_events](../event-watching/watch-src20-events.md) - Watch with automatic key fetching
- [watch_src20_events_with_key](../event-watching/watch-src20-events-with-key.md) - Watch with explicit key
- [SRC20EventWatcher](../event-watching/src20-event-watcher.md) - Sync watcher class
- [AsyncSRC20EventWatcher](../event-watching/async-src20-event-watcher.md) - Async watcher class
