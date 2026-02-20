---
description: AsyncSeismicPublicNamespace class - Read-only public operations (async)
icon: book-open
---

# AsyncSeismicPublicNamespace

The `AsyncSeismicPublicNamespace` class provides read-only Seismic functionality for asynchronous public clients. It does not require a private key and is designed for querying blockchain state without performing transactions.

## Overview

This namespace is automatically attached as `w3.seismic` when you create an async public client with `create_async_public_client()`. It provides:

- TEE public key retrieval (async)
- Deposit contract queries (root, count, async)
- Async public contract wrappers with transparent read capabilities
- No transaction signing or encryption capabilities

## Access

```python
from seismic_web3 import create_async_public_client

public = await create_async_public_client("https://gcp-1.seismictest.net/rpc")

# Access the namespace (all methods are async)
await public.seismic.get_tee_public_key()
contract = public.seismic.contract(address, abi)
```

## Public vs Wallet Namespaces

| Feature | Public Namespace | Wallet Namespace |
|---------|------------------|------------------|
| **Read-only queries** | ✅ Yes | ✅ Yes (inherited) |
| **Shielded writes** | ❌ No | ✅ Yes |
| **Signed reads** | ❌ No | ✅ Yes |
| **Validator deposits** | ❌ No | ✅ Yes |
| **Contract writes** | ❌ No | ✅ Yes |
| **Requires private key** | ❌ No | ✅ Yes |

The public namespace is ideal for:
- Monitoring blockchain state asynchronously
- Reading public contract data with async/await
- Querying deposit information concurrently
- High-performance applications that don't need to send transactions

For write operations, use `AsyncSeismicNamespace` with an async wallet client.

## Class Definition

```python
class AsyncSeismicPublicNamespace:
    """Async public Seismic namespace -- attached as ``w3.seismic``.

    Provides read-only convenience methods that do not require
    a private key or encryption state.

    Args:
        w3: Async AsyncWeb3 instance.
    """
```

## Methods

### Query Methods

#### get_tee_public_key()

Fetch the TEE's compressed secp256k1 public key (async).

```python
tee_key = await public.seismic.get_tee_public_key()
```

**Returns:** 33-byte compressed public key (bytes)

**Use case:** Required for client-side encryption before sending shielded transactions (though this namespace cannot send transactions itself).

[See detailed documentation →](methods/get-tee-public-key.md)

---

#### get_deposit_root()

Read the current deposit Merkle root from the deposit contract (async).

```python
root = await public.seismic.get_deposit_root(
    address="0x4242424242424242424242424242424242424242",
)
```

**Parameters:**
- `address` (str, optional): Deposit contract address (defaults to genesis contract at `0x4242424242424242424242424242424242424242`)

**Returns:** 32-byte deposit root hash

**Use case:** Verify deposit inclusion or monitor deposit tree state.

[See detailed documentation →](methods/get-deposit-root.md)

---

#### get_deposit_count()

Read the current deposit count from the deposit contract (async).

```python
count = await public.seismic.get_deposit_count(
    address="0x4242424242424242424242424242424242424242",
)
```

**Parameters:**
- `address` (str, optional): Deposit contract address (defaults to genesis contract at `0x4242424242424242424242424242424242424242`)

**Returns:** Number of deposits as a Python int

**Use case:** Monitor validator deposits or calculate deposit tree depth.

[See detailed documentation →](methods/get-deposit-count.md)

---

### Contract Factory

#### contract()

Create an `AsyncPublicContract` wrapper with transparent read-only capabilities.

```python
contract = public.seismic.contract(
    address=contract_address,
    abi=contract_abi,
)
```

**Parameters:**
- `address` (ChecksumAddress): Contract address
- `abi` (list[dict[str, Any]]): Contract ABI (list of function entries)

**Returns:** `AsyncPublicContract` instance with namespace:
- `.tread` - Transparent reads (unencrypted, read-only, async)

**Note:** This returns an `AsyncPublicContract`, not an `AsyncShieldedContract`. It only supports transparent reads via the `.tread` namespace. For full contract capabilities including writes and encrypted operations, use an async wallet client with `AsyncSeismicNamespace`.

[See contract documentation →](../contract/)

---

## Usage Examples

### Basic Query Operations

```python
import asyncio
from seismic_web3 import create_async_public_client

async def main():
    # Create async public client (no private key required)
    public = await create_async_public_client("https://gcp-1.seismictest.net/rpc")

    # Get TEE public key
    tee_key = await public.seismic.get_tee_public_key()
    print(f"TEE public key: {tee_key.hex()}")

    # Query deposit information
    deposit_count = await public.seismic.get_deposit_count()
    deposit_root = await public.seismic.get_deposit_root()
    print(f"Total deposits: {deposit_count}")
    print(f"Deposit root: {deposit_root.hex()}")

asyncio.run(main())
```

### Concurrent Query Operations

```python
async def concurrent_queries():
    public = await create_async_public_client("https://gcp-1.seismictest.net/rpc")

    # Execute multiple queries concurrently
    results = await asyncio.gather(
        public.seismic.get_tee_public_key(),
        public.seismic.get_deposit_count(),
        public.seismic.get_deposit_root(),
    )

    tee_key, deposit_count, deposit_root = results
    print(f"TEE key: {tee_key.hex()}")
    print(f"Deposits: {deposit_count}")
    print(f"Root: {deposit_root.hex()}")
```

### Reading Contract Data

```python
async def read_contract():
    public = await create_async_public_client("https://gcp-1.seismictest.net/rpc")

    # Create contract wrapper
    contract = public.seismic.contract(
        address="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        abi=[
            {
                "name": "balanceOf",
                "type": "function",
                "inputs": [{"name": "account", "type": "address"}],
                "outputs": [{"name": "", "type": "uint256"}],
                "stateMutability": "view",
            },
            {
                "name": "totalSupply",
                "type": "function",
                "inputs": [],
                "outputs": [{"name": "", "type": "uint256"}],
                "stateMutability": "view",
            },
        ],
    )

    # Read contract state (transparent, unencrypted)
    balance = await contract.tread.balanceOf(
        "0x1234567890123456789012345678901234567890",
    )
    total_supply = await contract.tread.totalSupply()

    print(f"Balance: {balance}")
    print(f"Total supply: {total_supply}")
```

### Concurrent Contract Reads

```python
async def concurrent_contract_reads():
    public = await create_async_public_client("https://gcp-1.seismictest.net/rpc")
    contract = public.seismic.contract(address, abi)

    # Read multiple accounts concurrently
    accounts = [
        "0x1234567890123456789012345678901234567890",
        "0x2234567890123456789012345678901234567890",
        "0x3234567890123456789012345678901234567890",
    ]

    balances = await asyncio.gather(
        *[contract.tread.balanceOf(account) for account in accounts]
    )

    for account, balance in zip(accounts, balances):
        print(f"{account}: {balance}")
```

### Monitoring Deposits

```python
async def monitor_deposits():
    public = await create_async_public_client("https://gcp-1.seismictest.net/rpc")
    last_count = 0

    while True:
        current_count = await public.seismic.get_deposit_count()

        if current_count > last_count:
            print(f"New deposits detected: {current_count - last_count}")
            print(f"Total deposits: {current_count}")

            # Get updated root
            root = await public.seismic.get_deposit_root()
            print(f"Current root: {root.hex()}")

            last_count = current_count

        await asyncio.sleep(12)  # Check every block (~12 seconds)
```

### Multiple Networks Monitoring

```python
async def monitor_multiple_networks():
    # Create clients for multiple networks
    networks = {
        "testnet": "https://gcp-1.seismictest.net/rpc",
        "sanvil": "http://127.0.0.1:8545",
    }

    clients = {
        name: await create_async_public_client(rpc)
        for name, rpc in networks.items()
    }

    # Query all networks concurrently
    async def get_chain_info(name, client):
        deposit_count = await client.seismic.get_deposit_count()
        deposit_root = await client.seismic.get_deposit_root()
        return {
            "chain": name,
            "deposits": deposit_count,
            "root": deposit_root.hex(),
        }

    results = await asyncio.gather(
        *[get_chain_info(name, client) for name, client in clients.items()]
    )

    for info in results:
        print(f"{info['chain']}: {info['deposits']} deposits")
```

### Context Manager Pattern

```python
async def context_manager_example():
    async with await create_async_public_client("https://gcp-1.seismictest.net/rpc") as public:
        # Use the client
        tee_key = await public.seismic.get_tee_public_key()
        deposit_count = await public.seismic.get_deposit_count()

        print(f"TEE key: {tee_key.hex()}")
        print(f"Deposits: {deposit_count}")
    # Client automatically cleaned up
```

### Data Aggregation

```python
async def get_chain_summary(rpc_url: str) -> dict:
    """Aggregate public chain data asynchronously."""
    async with await create_async_public_client(rpc_url) as public:
        # Execute all queries concurrently
        tee_key, deposit_count, deposit_root, block_number, chain_id = await asyncio.gather(
            public.seismic.get_tee_public_key(),
            public.seismic.get_deposit_count(),
            public.seismic.get_deposit_root(),
            public.eth.block_number,
            public.eth.chain_id,
        )

        return {
            "tee_public_key": tee_key.hex(),
            "deposit_count": deposit_count,
            "deposit_root": deposit_root.hex(),
            "block_number": block_number,
            "chain_id": chain_id,
        }

summary = await get_chain_summary("https://gcp-1.seismictest.net/rpc")
print(summary)
```

### Custom Deposit Contract

```python
async def custom_deposit_contract():
    public = await create_async_public_client("https://gcp-1.seismictest.net/rpc")

    # Query custom deposit contract (non-default address)
    custom_deposit_address = "0x1234567890123456789012345678901234567890"

    count, root = await asyncio.gather(
        public.seismic.get_deposit_count(address=custom_deposit_address),
        public.seismic.get_deposit_root(address=custom_deposit_address),
    )

    print(f"Custom contract deposits: {count}")
    print(f"Custom contract root: {root.hex()}")
```

## Async Best Practices

### Use asyncio.gather() for Parallel Operations

```python
# Good: Execute operations in parallel
tee_key, count, root = await asyncio.gather(
    public.seismic.get_tee_public_key(),
    public.seismic.get_deposit_count(),
    public.seismic.get_deposit_root(),
)

# Bad: Execute operations sequentially
tee_key = await public.seismic.get_tee_public_key()
count = await public.seismic.get_deposit_count()
root = await public.seismic.get_deposit_root()
```

### Handle Connection Cleanup

```python
# Use context manager or explicitly close connections
async with await create_async_public_client(rpc_url) as public:
    # Perform operations
    pass
# Connection automatically closed
```

### Error Handling with Async

```python
try:
    deposit_count = await public.seismic.get_deposit_count()
except Exception as e:
    print(f"Query failed: {e}")
```

### Batch Operations

```python
async def batch_query_multiple_contracts():
    public = await create_async_public_client("https://gcp-1.seismictest.net/rpc")

    # Create multiple contract instances
    contracts = [
        public.seismic.contract(address, abi)
        for address in contract_addresses
    ]

    # Query all contracts concurrently
    results = await asyncio.gather(
        *[contract.tread.totalSupply() for contract in contracts]
    )

    return results
```

## When to Use Public vs Wallet Client

### Use Async Public Client When:

1. **Read-only operations**: You only need to query blockchain state
2. **No private key**: You don't have or want to handle private keys
3. **High performance**: Need concurrent query execution
4. **Monitoring**: Building async monitoring or analytics tools
5. **Public data**: Accessing transparent contract data
6. **Security**: Minimizing attack surface by not handling keys

```python
# Good use case for async public client
public = await create_async_public_client("https://gcp-1.seismictest.net/rpc")
deposit_stats = await asyncio.gather(
    public.seismic.get_deposit_count(),
    public.seismic.get_deposit_root(),
)
```

### Use Async Wallet Client When:

1. **Transactions**: You need to send transactions
2. **Shielded operations**: Working with encrypted contract calls
3. **Signed reads**: Performing authenticated queries
4. **Deposits**: Submitting validator deposits
5. **Full contract interaction**: Using write operations

```python
# Requires async wallet client
wallet = await create_async_wallet_client(rpc_url, private_key=key)
tx_hash = await wallet.seismic.send_shielded_transaction(
    to=address,
    data=data,
)
```

## Limitations

The async public namespace does **not** support:

- Sending transactions (use `AsyncSeismicNamespace.send_shielded_transaction()`)
- Signed reads (use `AsyncSeismicNamespace.signed_call()`)
- Shielded contract writes (use `AsyncShieldedContract.write`)
- Debug transactions (use `AsyncSeismicNamespace.debug_send_shielded_transaction()`)
- Validator deposits (use `AsyncSeismicNamespace.deposit()`)

Attempting to perform these operations requires upgrading to an async wallet client:

```python
# Upgrade to async wallet client for write operations
from seismic_web3 import create_async_wallet_client, PrivateKey

wallet = await create_async_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=PrivateKey(b"...32 bytes..."),
)

# Now you can perform write operations
tx_hash = await wallet.seismic.send_shielded_transaction(...)
```

## Performance Considerations

### Connection Pooling

```python
# Reuse client connections for better performance
async with await create_async_public_client(rpc_url) as public:
    # Multiple operations share the same connection
    for _ in range(100):
        count = await public.seismic.get_deposit_count()
```

### Concurrent Limits

```python
# Use semaphore to limit concurrent requests
semaphore = asyncio.Semaphore(10)  # Max 10 concurrent requests

async def limited_query(public, address):
    async with semaphore:
        return await public.seismic.get_deposit_count()

# Query many addresses with concurrency limit
results = await asyncio.gather(
    *[limited_query(public, addr) for addr in addresses]
)
```

## See Also

- [SeismicPublicNamespace](seismic-public-namespace.md) - Sync version of this namespace
- [AsyncSeismicNamespace](async-seismic-namespace.md) - Full async wallet namespace with write capabilities
- [Contract Documentation](../contract/) - Contract interaction patterns
- [Client Documentation](../client/) - Client creation and configuration
- [Guides](../guides/) - Step-by-step tutorials
