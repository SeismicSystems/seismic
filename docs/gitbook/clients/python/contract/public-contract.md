---
description: Sync contract wrapper with transparent read-only access
icon: book-open
---

# PublicContract

Synchronous contract wrapper for read-only transparent access to Seismic contracts.

## Overview

`PublicContract` provides a simplified interface for reading public contract state without requiring encryption or a private key. It exposes only the `.tread` namespace for standard `eth_call` operations. Use this class when you only need to read public contract data and don't need shielded operations.

## Definition

```python
class PublicContract:
    def __init__(
        self,
        w3: Web3,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        ...
```

## Constructor Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `w3` | `Web3` | Yes | Synchronous Web3 instance connected to RPC endpoint |
| `address` | `ChecksumAddress` | Yes | Contract address (checksummed Ethereum address) |
| `abi` | `list[dict[str, Any]]` | Yes | Contract ABI (list of function entries) |

## Namespace

### `.tread` - Transparent Read

Executes standard `eth_call` with unencrypted calldata. This is the only namespace available on `PublicContract`.

**Returns**: `Any` (ABI-decoded Python value)

**Optional Parameters**: None (pass positional arguments only)

## Examples

### Basic Read Operations

```python
from seismic_web3 import create_public_client, PublicContract

# Create client without private key
w3 = create_public_client(
    rpc_url="https://gcp-1.seismictest.net/rpc",
)

# Create read-only contract instance
contract = PublicContract(
    w3=w3,
    address="0x1234567890123456789012345678901234567890",
    abi=CONTRACT_ABI,
)

# Read public contract state (auto-decoded)
total_supply = contract.tread.totalSupply()  # int
print(f"Total supply: {total_supply}")

balance = contract.tread.balanceOf("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")  # int
print(f"Balance: {balance}")
```

### Single and Multiple Returns

```python
# Single return values are returned directly
number = contract.tread.getNumber()      # int
name = contract.tread.getName()          # str
is_active = contract.tread.isActive()    # bool

# Multiple outputs are returned as a tuple
user_name, user_balance, active = contract.tread.getUserInfo(
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
)
```

### Multiple Reads

```python
# Read multiple values sequentially
total_supply = contract.tread.totalSupply()
decimals = contract.tread.decimals()
symbol = contract.tread.symbol()

print(f"Token: {symbol}, Decimals: {decimals}, Supply: {total_supply}")
```

### Array Results

```python
# Read array of addresses (auto-decoded to list)
holders = contract.tread.getHolders()
print(f"Found {len(holders)} holders")
for holder in holders:
    print(f"  - {holder}")
```

### Instantiation via Client

```python
# Most common pattern - let the client create the contract
from seismic_web3 import create_public_client

w3 = create_public_client(
    rpc_url="https://gcp-1.seismictest.net/rpc",
)

# Client's contract() method creates PublicContract
contract = w3.seismic.contract(address=contract_address, abi=CONTRACT_ABI)

# Only .tread namespace is available
result = contract.tread.getNumber()  # int
```

### Checking Contract State

```python
# Check if address has role
is_admin = contract.tread.hasRole(
    b'\x00' * 32,  # DEFAULT_ADMIN_ROLE
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
)
print(f"Is admin: {is_admin}")
```

### Pagination Pattern

```python
def get_all_items(contract: PublicContract, batch_size: int = 100):
    """Read paginated data from contract."""
    # Get total count
    total = contract.tread.getItemCount()

    items = []
    for offset in range(0, total, batch_size):
        # Read batch
        batch = contract.tread.getItems(offset, batch_size)
        items.extend(batch)

    return items

all_items = get_all_items(contract)
print(f"Retrieved {len(all_items)} items")
```

### Call Semantics

```python
# .tread uses eth_call semantics
# It is best for reads; non-view calls are simulated and do not persist state

# Typical usage:
balance = contract.tread.balanceOf("0xAddress...")  # Works
allowance = contract.tread.allowance(owner, spender)  # Works
```

### Error Handling

```python
try:
    value = contract.tread.getNumber()
    print(f"Value: {value}")

except ValueError as e:
    print(f"RPC error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

### With Custom RPC

```python
from web3 import Web3

# Use custom RPC provider
w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

contract = PublicContract(
    w3=w3,
    address=contract_address,
    abi=CONTRACT_ABI,
)

result = contract.tread.getNumber()
```

## Notes

- **Read-only**: No write operations available (no `.write`, `.twrite`, or `.dwrite` namespaces)
- **No encryption required**: Does not use `EncryptionState` or private keys
- **No authentication**: Standard unsigned `eth_call` operations
- **`eth_call` semantics**: Best suited for `view`/`pure`; non-view calls are simulated and do not persist state
- **Gas not consumed**: `eth_call` is free (doesn't create transactions)
- **No state changes**: Cannot modify blockchain state
- **Public data only**: Cannot access shielded/encrypted contract state
- **ABI remapping**: Same ABI type remapping as `ShieldedContract`
- **Dynamic methods**: Contract methods resolved via `__getattr__`

## Use Cases

### Public Data Queries

Perfect for:
- Querying ERC-20 token balances
- Reading public contract configuration
- Checking access control roles
- Viewing public state variables
- Aggregating data from multiple contracts
- Building read-only dashboards
- Monitoring contract state

### When to Use ShieldedContract Instead

Use `ShieldedContract` when you need:
- Write operations (state changes)
- Encrypted reads of shielded state
- Authenticated operations
- Transaction signing
- Access to private/shielded data

## See Also

- [AsyncPublicContract](async-public-contract.md) - Async version of this class
- [ShieldedContract](shielded-contract.md) - Full contract wrapper with write operations
- [create_public_client](../client/create-public-client.md) - Create client without private key
- [Contract Instance Guide](README.md) - Overview of contract interaction patterns
