---
description: Create async Web3 instance with public (read-only) Seismic access
icon: tower-observation
---

# create_async_public_client

Create an asynchronous `AsyncWeb3` instance with public (read-only) Seismic access.

## Overview

`create_async_public_client()` creates an async client for read-only operations on the Seismic network. No private key is required. The `w3.seismic` namespace provides only public read operations: `get_tee_public_key()`, `get_deposit_root()`, `get_deposit_count()`, and `contract()` (with `.tread` only).

Supports both HTTP and WebSocket connections for efficient async queries and real-time monitoring.

## Signature

```python
async def create_async_public_client(
    provider_url: str,
    *,
    ws: bool = False,
) -> AsyncWeb3
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `provider_url` | `str` | Yes | HTTP(S) or WS(S) URL of the Seismic node |
| `ws` | `bool` | No | If `True`, uses `WebSocketProvider` (persistent connection, supports subscriptions). Otherwise uses `AsyncHTTPProvider`. Default: `False`. WebSocket is only available on async clients — sync clients are HTTP-only |

## Returns

| Type | Description |
|------|-------------|
| `AsyncWeb3` | An `AsyncWeb3` instance with `w3.seismic` namespace attached ([`AsyncSeismicPublicNamespace`](../namespaces/async-seismic-public-namespace.md)) |

## Examples

### Basic Usage (HTTP)

```python
from seismic_web3 import create_async_public_client

# Create async public client
w3 = await create_async_public_client("https://gcp-1.seismictest.net/rpc")

# Query TEE public key
tee_pk = await w3.seismic.get_tee_public_key()
print(f"TEE public key: {tee_pk.to_0x_hex()}")

# Query deposit info
root = await w3.seismic.get_deposit_root()
count = await w3.seismic.get_deposit_count()
print(f"Deposit root: {root.to_0x_hex()}, count: {count}")
```

### WebSocket Connection

```python
from seismic_web3 import create_async_public_client

# WebSocket provider for persistent connection
w3 = await create_async_public_client(
    "wss://gcp-1.seismictest.net/ws",
    ws=True,
)

# Subscribe to new blocks
async for block in w3.eth.subscribe("newHeads"):
    print(f"New block: {block['number']}")

    # Query deposit count at each new block
    count = await w3.seismic.get_deposit_count()
    print(f"Current deposit count: {count}")
```

### Using Chain Configuration

```python
from seismic_web3 import SEISMIC_TESTNET

# Recommended: use chain config with HTTP
w3 = await SEISMIC_TESTNET.async_public_client()

# Or with WebSocket (uses ws_url from chain config)
w3 = await SEISMIC_TESTNET.async_public_client(ws=True)
```

### Async Application

```python
import asyncio
from seismic_web3 import create_async_public_client

async def main():
    w3 = await create_async_public_client("https://gcp-1.seismictest.net/rpc")

    # Get current block
    block = await w3.eth.get_block("latest")
    print(f"Latest block: {block['number']}")

    # Get balance
    address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    balance = await w3.eth.get_balance(address)
    print(f"Balance: {w3.from_wei(balance, 'ether')} ETH")

    # Query deposit info
    deposit_count = await w3.seismic.get_deposit_count()
    print(f"Total deposits: {deposit_count}")

asyncio.run(main())
```

### Read-Only Contract Access

```python
from seismic_web3 import create_async_public_client

async def query_contract():
    w3 = await create_async_public_client("https://gcp-1.seismictest.net/rpc")

    # Create contract wrapper (read-only)
    contract = w3.seismic.contract(
        address="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        abi=contract_abi,
    )

    # Only transparent reads are available
    result = await contract.tread.balanceOf("0x1234...")
    print(f"Balance: {result}")
```

### Monitoring Pattern

```python
from seismic_web3 import create_async_public_client
import asyncio

async def monitor_deposits():
    w3 = await create_async_public_client("wss://gcp-1.seismictest.net/ws", ws=True)

    last_count = await w3.seismic.get_deposit_count()
    print(f"Starting deposit count: {last_count}")

    async for block in w3.eth.subscribe("newHeads"):
        current_count = await w3.seismic.get_deposit_count()

        if current_count > last_count:
            print(f"New deposits detected! Count: {current_count}")
            print(f"Block: {block['number']}")
            last_count = current_count

asyncio.run(monitor_deposits())
```

### Parallel Queries

```python
from seismic_web3 import create_async_public_client
import asyncio

async def get_chain_stats():
    w3 = await create_async_public_client("https://gcp-1.seismictest.net/rpc")

    # Run multiple queries in parallel
    block, tee_pk, deposit_root, deposit_count = await asyncio.gather(
        w3.eth.get_block("latest"),
        w3.seismic.get_tee_public_key(),
        w3.seismic.get_deposit_root(),
        w3.seismic.get_deposit_count(),
    )

    return {
        "block_number": block["number"],
        "tee_public_key": tee_pk.to_0x_hex(),
        "deposit_root": deposit_root.to_0x_hex(),
        "deposit_count": deposit_count,
    }
```

### Context Manager Pattern

```python
from seismic_web3 import create_async_public_client

async with await create_async_public_client(
    "wss://gcp-1.seismictest.net/ws",
    ws=True,
) as w3:
    # WebSocket connection will be properly closed
    block = await w3.eth.get_block("latest")
    print(f"Block: {block['number']}")
```

## How It Works

The function performs three steps:

1. **Create provider**
   ```python
   if ws:
       provider = WebSocketProvider(provider_url)
   else:
       provider = AsyncHTTPProvider(provider_url)
   ```

2. **Create AsyncWeb3 instance**
   ```python
   w3 = AsyncWeb3(provider)
   ```

3. **Attach public Seismic namespace**
   ```python
   w3.seismic = AsyncSeismicPublicNamespace(w3)
   ```

No TEE public key fetching or encryption setup is performed since the client cannot perform shielded operations.

## Client Capabilities

### Standard AsyncWeb3 Methods (`w3.eth`)
- `await get_block()`, `await get_transaction()`, `await get_balance()`
- `await call()`, `await estimate_gas()`
- All other standard read-only async `web3.py` functionality

### Public Seismic Methods (`w3.seismic`)
- `await get_tee_public_key()` - Get TEE public key
- `await get_deposit_root()` - Query deposit merkle root
- `await get_deposit_count()` - Query deposit count
- `contract()` - Create contract wrappers (`.tread` methods are async)

### NOT Available
- `send_shielded_transaction()` - Requires private key
- `debug_send_shielded_transaction()` - Requires private key
- `signed_call()` - Requires private key and encryption
- `deposit()` - Requires private key
- Contract `.swrite` and `.sread` methods - Require private key

## HTTP vs WebSocket

| Aspect | AsyncHTTPProvider (`ws=False`) | WebSocketProvider (`ws=True`) |
|--------|-------------------------------|-------------------------------|
| **Connection** | New connection per request | Persistent connection |
| **Latency** | Higher per-request overhead | Lower latency |
| **Subscriptions** | Not supported | Supported (`eth.subscribe`) |
| **Resource usage** | Lower idle usage | Keeps connection open |
| **Use case** | One-off queries | Real-time monitoring, subscriptions |

## Notes

- The function is `async` and must be `await`-ed
- No private key required or accepted
- No encryption setup performed
- No RPC calls during client creation (lightweight)
- Cannot perform any write operations or shielded reads
- Contract wrappers only expose `.tread` (transparent read, async)
- All `w3.seismic` methods are async and must be `await`-ed
- WebSocket connections should be properly closed when done
- For write operations, use [`create_async_wallet_client()`](create-async-wallet-client.md)
- For sync operations, use [`create_public_client()`](create-public-client.md)

## Use Cases

- Async block explorers and chain analytics
- Real-time monitoring dashboards with WebSocket subscriptions
- High-throughput read-only services
- Async data aggregation pipelines
- Event monitoring and alerting systems
- Price oracles with low-latency requirements

## Warnings

- **Connection cleanup** - Close WebSocket connections properly to avoid resource leaks
- **Error handling** - WebSocket connections can drop; implement reconnection logic for production
- **HTTPS/WSS recommended** - Use secure protocols in production to prevent MITM attacks

## See Also

- [create_public_client](create-public-client.md) - Sync variant (HTTP only)
- [create_async_wallet_client](create-async-wallet-client.md) - Async client with private key
- [AsyncSeismicPublicNamespace](../namespaces/async-seismic-public-namespace.md) - The async public `w3.seismic` namespace
- [Chains Configuration](../chains.md) - Pre-configured chain constants
- [Contract Instances](../contract-instance.md) - Working with contract wrappers
