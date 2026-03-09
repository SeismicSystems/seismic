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

**Returns**: `Coroutine[Any]` (ABI-decoded Python value)

**Optional Parameters**: None (pass positional arguments only)

## Examples

### Basic Read Operations

```python
import asyncio
from seismic_web3 import create_async_public_client, AsyncPublicContract

async def main():
    # Create async client without private key
    w3 = create_async_public_client(
        provider_url="https://gcp-1.seismictest.net/rpc",
    )

    # Create read-only contract instance
    contract = AsyncPublicContract(
        w3=w3,
        address="0x1234567890123456789012345678901234567890",
        abi=CONTRACT_ABI,
    )

    # Read public contract state (must await, auto-decoded)
    total_supply = await contract.tread.totalSupply()  # int
    print(f"Total supply: {total_supply}")

    balance = await contract.tread.balanceOf("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")  # int
    print(f"Balance: {balance}")

asyncio.run(main())
```

### Concurrent Reads

```python
async def concurrent_reads(contract: AsyncPublicContract):
    # Execute multiple reads concurrently (all auto-decoded)
    total_supply, decimals, symbol, name = await asyncio.gather(
        contract.tread.totalSupply(),
        contract.tread.decimals(),
        contract.tread.symbol(),
        contract.tread.name(),
    )

    print(f"Name: {name}")
    print(f"Symbol: {symbol}")
    print(f"Decimals: {decimals}")
    print(f"Supply: {total_supply}")
```

### Single and Multiple Returns

```python
async def return_types(contract: AsyncPublicContract):
    # Single output values are returned directly
    number = await contract.tread.getNumber()    # int
    name = await contract.tread.getName()        # str
    active = await contract.tread.isActive()     # bool

    # Multiple outputs return a tuple
    user_name, balance, is_active = await contract.tread.getUserInfo(
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    )
```

### Array Results

```python
async def array_example(contract: AsyncPublicContract):
    # Read array of addresses (auto-decoded to list)
    holders = await contract.tread.getHolders()
    print(f"Found {len(holders)} holders")

    # Query additional data for each holder concurrently
    balances = await asyncio.gather(
        *[contract.tread.balanceOf(holder) for holder in holders]
    )

    for holder, balance in zip(holders, balances):
        print(f"  {holder}: {balance}")
```

### Error Handling

```python
async def error_handling(contract: AsyncPublicContract):
    try:
        value = await contract.tread.getNumber()
        print(f"Value: {value}")

    except ValueError as e:
        print(f"RPC error: {e}")
    except asyncio.TimeoutError:
        print("Request timed out")
    except Exception as e:
        print(f"Unexpected error: {e}")
```

## Notes

- **All methods return coroutines**: Must use `await` with every `.tread` call
- **Read-only**: No write operations available
- **No encryption required**: Does not use `EncryptionState` or private keys
- **No authentication**: Standard unsigned async `eth_call` operations
- **Gas not consumed**: `eth_call` is free (doesn't create transactions)

## See Also

- [PublicContract](public-contract.md) - Synchronous version of this class
- [AsyncShieldedContract](async-shielded-contract.md) - Full async contract wrapper with write operations
- [create_async_public_client](../client/create-async-public-client.md) - Create async client without private key
- [Contract Instance Guide](README.md) - Overview of contract interaction patterns
