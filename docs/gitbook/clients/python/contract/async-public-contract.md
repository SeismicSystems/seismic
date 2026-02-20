---
description: Async contract wrapper with transparent read-only access
icon: book-open
---

# AsyncPublicContract

Asynchronous contract wrapper for read-only transparent access to Seismic contracts.

## Overview

`AsyncPublicContract` is the async version of `PublicContract`, providing non-blocking read-only access to contract state. It exposes only the `.tread` namespace for standard async `eth_call` operations. All methods return coroutines that must be awaited. Use this class in async applications when you only need to read public contract data.

## Definition

```python
class AsyncPublicContract:
    def __init__(
        self,
        w3: AsyncWeb3,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
    ) -> None:
        ...
```

## Constructor Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `w3` | `AsyncWeb3` | Yes | Asynchronous AsyncWeb3 instance connected to RPC endpoint |
| `address` | `ChecksumAddress` | Yes | Contract address (checksummed Ethereum address) |
| `abi` | `list[dict[str, Any]]` | Yes | Contract ABI (list of function entries) |

## Namespace

### `.tread` - Transparent Read

Executes standard async `eth_call` with unencrypted calldata. This is the only namespace available on `AsyncPublicContract`.

**Returns**: `Coroutine[HexBytes]` (raw result bytes)

**Optional Parameters**: None (pass positional arguments only)

## Examples

### Basic Read Operations

```python
import asyncio
from seismic_web3 import create_async_public_client, AsyncPublicContract, SEISMIC_TESTNET

async def main():
    # Create async client without private key
    w3 = await create_async_public_client(
        rpc_url="https://testnet-rpc.seismic.network",
        chain=SEISMIC_TESTNET,
    )

    # Create read-only contract instance
    contract = AsyncPublicContract(
        w3=w3.eth,
        address="0x1234567890123456789012345678901234567890",
        abi=CONTRACT_ABI,
    )

    # Read public contract state (must await)
    result = await contract.tread.getTotalSupply()
    print(f"Total supply: {result.to_0x_hex()}")

    balance = await contract.tread.balanceOf("0xAddress...")
    print(f"Balance: {balance.to_0x_hex()}")

asyncio.run(main())
```

### Concurrent Reads

```python
async def concurrent_reads(contract: AsyncPublicContract):
    # Execute multiple reads concurrently
    results = await asyncio.gather(
        contract.tread.totalSupply(),
        contract.tread.decimals(),
        contract.tread.symbol(),
        contract.tread.name(),
    )

    total_supply, decimals, symbol, name = results

    # Decode all results
    from eth_abi import decode

    print(f"Name: {decode(['string'], name)[0]}")
    print(f"Symbol: {decode(['string'], symbol)[0]}")
    print(f"Decimals: {decode(['uint8'], decimals)[0]}")
    print(f"Supply: {decode(['uint256'], total_supply)[0]}")
```

### Batch Balance Queries

```python
async def get_balances(contract: AsyncPublicContract, addresses: list[str]):
    """Get balances for multiple addresses concurrently."""
    # Query all balances in parallel
    balance_results = await asyncio.gather(
        *[contract.tread.balanceOf(addr) for addr in addresses]
    )

    # Decode results
    from eth_abi import decode

    balances = [decode(['uint256'], result)[0] for result in balance_results]

    # Return address -> balance mapping
    return dict(zip(addresses, balances))

addresses = ["0xAddress1...", "0xAddress2...", "0xAddress3..."]
balances = await get_balances(contract, addresses)

for addr, balance in balances.items():
    print(f"{addr}: {balance}")
```

### Decoding Results

```python
async def decode_example(contract: AsyncPublicContract):
    from eth_abi import decode

    # Read and decode uint256
    result = await contract.tread.getNumber()
    decoded = decode(['uint256'], result)[0]
    print(f"Number: {decoded}")

    # Read and decode address
    owner = await contract.tread.owner()
    decoded_address = decode(['address'], owner)[0]
    print(f"Owner: {decoded_address}")

    # Read and decode string
    name = await contract.tread.name()
    decoded_name = decode(['string'], name)[0]
    print(f"Name: {decoded_name}")
```

### Complex Return Types

```python
async def complex_types(contract: AsyncPublicContract):
    from eth_abi import decode

    # Function returning multiple values
    result = await contract.tread.getInfo()

    # Decode tuple return
    name, version, admin = decode(['string', 'uint256', 'address'], result)
    print(f"Name: {name}")
    print(f"Version: {version}")
    print(f"Admin: {admin}")
```

### Array Results

```python
async def array_example(contract: AsyncPublicContract):
    from eth_abi import decode

    # Read array of addresses
    result = await contract.tread.getAllHolders()

    # Decode dynamic array
    holders = decode(['address[]'], result)[0]
    print(f"Found {len(holders)} holders")

    # Query additional data for each holder concurrently
    balances = await asyncio.gather(
        *[contract.tread.balanceOf(holder) for holder in holders]
    )

    for holder, balance_bytes in zip(holders, balances):
        balance = decode(['uint256'], balance_bytes)[0]
        print(f"  {holder}: {balance}")
```

### Instantiation via Client

```python
async def client_pattern():
    # Most common pattern - let the client create the contract
    w3 = await create_async_public_client(
        rpc_url="https://testnet-rpc.seismic.network",
        chain=SEISMIC_TESTNET,
    )

    # Client's contract() method creates AsyncPublicContract
    contract = w3.seismic.contract(address=contract_address, abi=CONTRACT_ABI)

    # Only .tread namespace is available (must await)
    result = await contract.tread.myFunction()
```

### Pagination with Concurrency

```python
async def get_all_items(contract: AsyncPublicContract, batch_size: int = 100):
    """Read paginated data with concurrent batch requests."""
    from eth_abi import decode

    # Get total count
    total_bytes = await contract.tread.getTotalCount()
    total = decode(['uint256'], total_bytes)[0]

    # Calculate batch offsets
    offsets = range(0, total, batch_size)

    # Fetch all batches concurrently
    batch_results = await asyncio.gather(
        *[contract.tread.getItems(offset, batch_size) for offset in offsets]
    )

    # Flatten results
    items = []
    for result in batch_results:
        batch = decode(['uint256[]'], result)[0]
        items.extend(batch)

    return items

items = await get_all_items(contract)
print(f"Retrieved {len(items)} items")
```

### Monitoring Contract State

```python
async def monitor_supply(contract: AsyncPublicContract, interval: int = 10):
    """Monitor total supply every N seconds."""
    from eth_abi import decode

    previous = None

    while True:
        result = await contract.tread.totalSupply()
        current = decode(['uint256'], result)[0]

        if previous is not None and current != previous:
            change = current - previous
            print(f"Supply changed: {previous} -> {current} ({change:+d})")

        previous = current
        await asyncio.sleep(interval)

# Run monitor in background
asyncio.create_task(monitor_supply(contract))
```

### Error Handling

```python
async def error_handling(contract: AsyncPublicContract):
    try:
        result = await contract.tread.getRiskyData()

        from eth_abi import decode
        value = decode(['uint256'], result)[0]
        print(f"Value: {value}")

    except ValueError as e:
        print(f"RPC error: {e}")
    except asyncio.TimeoutError:
        print("Request timed out")
    except Exception as e:
        print(f"Unexpected error: {e}")
```

### With Timeout

```python
async def with_timeout(contract: AsyncPublicContract):
    try:
        # Set 5-second timeout for read
        result = await asyncio.wait_for(
            contract.tread.slowFunction(),
            timeout=5.0
        )
        print(f"Result: {result.to_0x_hex()}")

    except asyncio.TimeoutError:
        print("Operation timed out after 5 seconds")
```

### Context Manager Pattern

```python
async def context_pattern():
    async with await create_async_public_client(...) as w3:
        contract = AsyncPublicContract(
            w3=w3.eth,
            address=contract_address,
            abi=CONTRACT_ABI,
        )

        result = await contract.tread.getData()
        print(f"Result: {result.to_0x_hex()}")
    # Connection automatically closed
```

### Multi-Contract Queries

```python
async def multi_contract_query(
    contracts: list[AsyncPublicContract],
    function_name: str,
    args: list,
):
    """Query same function across multiple contracts."""
    results = await asyncio.gather(
        *[
            getattr(contract.tread, function_name)(*args)
            for contract in contracts
        ]
    )
    return results

# Query multiple token contracts
token_contracts = [
    AsyncPublicContract(w3, addr1, ERC20_ABI),
    AsyncPublicContract(w3, addr2, ERC20_ABI),
    AsyncPublicContract(w3, addr3, ERC20_ABI),
]

supplies = await multi_contract_query(token_contracts, "totalSupply", [])
```

### Retry Logic

```python
async def read_with_retry(
    contract: AsyncPublicContract,
    function_name: str,
    args: list,
    max_retries: int = 3,
):
    """Read with automatic retry on failure."""
    for attempt in range(max_retries):
        try:
            result = await getattr(contract.tread, function_name)(*args)
            return result
        except Exception as e:
            if attempt == max_retries - 1:
                raise
            print(f"Attempt {attempt + 1} failed: {e}, retrying...")
            await asyncio.sleep(1 * (attempt + 1))  # Exponential backoff

result = await read_with_retry(contract, "getData", [])
```

## Notes

- **All methods return coroutines**: Must use `await` with every `.tread` call
- **Concurrent reads**: Use `asyncio.gather()` for parallel queries
- **Read-only**: No write operations available
- **No encryption required**: Does not use `EncryptionState` or private keys
- **No authentication**: Standard unsigned async `eth_call` operations
- **View functions only**: Can only call `view` or `pure` functions
- **Gas not consumed**: `eth_call` is free (doesn't create transactions)
- **Connection pooling**: AsyncWeb3 efficiently manages connection reuse
- **Error handling**: Use try/except around await calls for RPC errors
- **Timeout support**: Use `asyncio.wait_for()` to limit operation duration

## Performance Tips

- **Batch related queries**: Use `asyncio.gather()` to execute multiple reads in parallel
- **Avoid sequential waits**: Don't await in loops; collect coroutines and use `gather()`
- **Connection limits**: Be aware of RPC provider's concurrent request limits
- **Retry logic**: Implement retries for transient network failures
- **Caching**: Cache frequently accessed immutable data (like token decimals)

## Use Cases

### High-Throughput Queries

Perfect for:
- Scanning contract events and state concurrently
- Building real-time dashboards
- Aggregating data from multiple contracts
- Monitoring blockchain state
- Batch processing of contract reads
- Websocket-based applications

### When to Use AsyncShieldedContract Instead

Use `AsyncShieldedContract` when you need:
- Write operations (state changes)
- Encrypted reads of shielded state
- Authenticated operations
- Transaction signing
- Access to private/shielded data

## See Also

- [PublicContract](public-contract.md) - Synchronous version of this class
- [AsyncShieldedContract](async-shielded-contract.md) - Full async contract wrapper with write operations
- [create_async_public_client](../client/create-async-public-client.md) - Create async client without private key
- [Contract Instance Guide](README.md) - Overview of contract interaction patterns
