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

**Returns**: `HexBytes` (raw result bytes)

**Optional Parameters**: None (pass positional arguments only)

## Examples

### Basic Read Operations

```python
from seismic_web3 import create_public_client, PublicContract, SEISMIC_TESTNET

# Create client without private key
w3 = create_public_client(
    rpc_url="https://gcp-1.seismictest.net/rpc",
    chain=SEISMIC_TESTNET,
)

# Create read-only contract instance
contract = PublicContract(
    w3=w3.eth,
    address="0x1234567890123456789012345678901234567890",
    abi=CONTRACT_ABI,
)

# Read public contract state
result = contract.tread.getTotalSupply()
print(f"Total supply: {result.to_0x_hex()}")

balance = contract.tread.balanceOf("0xAddress...")
print(f"Balance: {balance.to_0x_hex()}")
```

### Decoding Results

```python
from eth_abi import decode

# Read and decode uint256
result = contract.tread.getNumber()
decoded = decode(['uint256'], result)[0]
print(f"Number: {decoded}")

# Read and decode address
owner = contract.tread.owner()
decoded_address = decode(['address'], owner)[0]
print(f"Owner: {decoded_address}")

# Read and decode string
name = contract.tread.name()
decoded_name = decode(['string'], name)[0]
print(f"Name: {decoded_name}")
```

### Multiple Reads

```python
# Read multiple values sequentially
total_supply = contract.tread.totalSupply()
decimals = contract.tread.decimals()
symbol = contract.tread.symbol()

# Decode results
from eth_abi import decode

total = decode(['uint256'], total_supply)[0]
dec = decode(['uint8'], decimals)[0]
sym = decode(['string'], symbol)[0]

print(f"Token: {sym}, Decimals: {dec}, Supply: {total}")
```

### Complex Return Types

```python
# Function returning multiple values
result = contract.tread.getInfo()

# Decode tuple return
from eth_abi import decode

name, version, admin = decode(['string', 'uint256', 'address'], result)
print(f"Name: {name}")
print(f"Version: {version}")
print(f"Admin: {admin}")
```

### Array Results

```python
# Read array of addresses
result = contract.tread.getAllHolders()

# Decode dynamic array
from eth_abi import decode

holders = decode(['address[]'], result)[0]
print(f"Found {len(holders)} holders")
for holder in holders:
    print(f"  - {holder}")
```

### Instantiation via Client

```python
# Most common pattern - let the client create the contract
from seismic_web3 import create_public_client, SEISMIC_TESTNET

w3 = create_public_client(
    rpc_url="https://gcp-1.seismictest.net/rpc",
    chain=SEISMIC_TESTNET,
)

# Client's contract() method creates PublicContract
contract = w3.seismic.contract(address=contract_address, abi=CONTRACT_ABI)

# Only .tread namespace is available
result = contract.tread.myFunction()
```

### Checking Contract State

```python
# Check if address has role
has_role = contract.tread.hasRole(
    b'\x00' * 32,  # DEFAULT_ADMIN_ROLE
    "0xAddress..."
)

from eth_abi import decode
is_admin = decode(['bool'], has_role)[0]
print(f"Is admin: {is_admin}")
```

### Pagination Pattern

```python
def get_all_items(contract: PublicContract, batch_size: int = 100):
    """Read paginated data from contract."""
    from eth_abi import decode

    # Get total count
    total = decode(['uint256'], contract.tread.getTotalCount())[0]

    items = []
    for offset in range(0, total, batch_size):
        # Read batch
        result = contract.tread.getItems(offset, batch_size)
        batch = decode(['uint256[]'], result)[0]
        items.extend(batch)

    return items

all_items = get_all_items(contract)
print(f"Retrieved {len(all_items)} items")
```

### View Functions Only

```python
# PublicContract only works with view/pure functions
# These would fail (require transaction):
# contract.tread.transfer(...)  # Not a view function
# contract.tread.approve(...)   # Not a view function

# Only read operations work:
balance = contract.tread.balanceOf("0xAddress...")  # Works
allowance = contract.tread.allowance(owner, spender)  # Works
```

### Error Handling

```python
try:
    result = contract.tread.getRiskyData()

    from eth_abi import decode
    value = decode(['uint256'], result)[0]
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

result = contract.tread.getData()
```

## Notes

- **Read-only**: No write operations available (no `.write`, `.twrite`, or `.dwrite` namespaces)
- **No encryption required**: Does not use `EncryptionState` or private keys
- **No authentication**: Standard unsigned `eth_call` operations
- **View functions only**: Can only call `view` or `pure` functions
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
