---
description: Best practices for async usage - concurrent operations, error handling, connection pooling
icon: bolt
---

# Async Patterns

This example demonstrates best practices for using the async Seismic client including concurrent operations, proper resource management, error handling, and WebSocket connections.

## Prerequisites

```bash
# Install the SDK with async support
pip install seismic-web3

# Set environment variables
export PRIVATE_KEY="your_64_char_hex_private_key"
export RPC_URL="https://gcp-1.seismictest.net/rpc"
export WS_URL="wss://gcp-1.seismictest.net/ws"
export CONTRACT_ADDRESS="0x..."
```

## Basic Async Client Setup

```python
import asyncio
import os
from seismic_web3 import create_async_wallet_client, PrivateKey


async def main():
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))

    # Create async client
    w3 = await create_async_wallet_client(
        os.environ["RPC_URL"],
        private_key=private_key,
    )

    try:
        # All operations must be awaited
        chain_id = await w3.eth.chain_id
        block_number = await w3.eth.block_number
        print(f"Connected to chain {chain_id}, block {block_number}")

    finally:
        # Always disconnect to clean up resources
        await w3.provider.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
```

## Concurrent Operations

Execute multiple independent operations in parallel for better performance:

```python
import asyncio
import os
from seismic_web3 import create_async_wallet_client, PrivateKey, SEISMIC_TESTNET
from hexbytes import HexBytes


def decode_uint256(raw: HexBytes) -> int:
    return int.from_bytes(raw[-32:], "big")


async def main():
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
    w3 = await SEISMIC_TESTNET.async_wallet_client(private_key)

    try:
        # Method 1: asyncio.gather for multiple operations
        print("Fetching data concurrently...")

        chain_id_task = w3.eth.chain_id
        block_number_task = w3.eth.block_number
        balance_task = w3.eth.get_balance(w3.eth.default_account)
        tee_pk_task = w3.seismic.get_tee_public_key()

        # Execute all concurrently
        chain_id, block_number, balance, tee_pk = await asyncio.gather(
            chain_id_task,
            block_number_task,
            balance_task,
            tee_pk_task,
        )

        print(f"Chain ID: {chain_id}")
        print(f"Block: {block_number}")
        print(f"Balance: {w3.from_wei(balance, 'ether')} ETH")
        print(f"TEE PK: {tee_pk.to_0x_hex()[:20]}...")

        # Method 2: asyncio.create_task for immediate scheduling
        ABI = [
            {
                "name": "getValue",
                "type": "function",
                "inputs": [{"name": "index", "type": "uint256"}],
                "outputs": [{"name": "", "type": "uint256"}],
            }
        ]

        contract = w3.seismic.contract(os.environ["CONTRACT_ADDRESS"], ABI)

        # Create tasks immediately
        tasks = [
            asyncio.create_task(contract.read.getValue(i))
            for i in range(5)
        ]

        # Wait for all tasks
        results = await asyncio.gather(*tasks)

        print("\nContract values:")
        for i, result in enumerate(results):
            value = decode_uint256(result)
            print(f"  Value {i}: {value}")

    finally:
        await w3.provider.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
```

## Concurrent Writes with Progress Tracking

```python
import asyncio
import os
from seismic_web3 import create_async_wallet_client, PrivateKey, SEISMIC_TESTNET


async def send_transaction_with_tracking(contract, value: int, index: int):
    """Send a transaction and track its progress."""
    print(f"[{index}] Sending transaction with value {value}...")

    # Send transaction
    tx_hash = await contract.write.setValue(value)
    print(f"[{index}] Transaction sent: {tx_hash.hex()}")

    # Wait for receipt
    receipt = await contract._w3.eth.wait_for_transaction_receipt(tx_hash, timeout=60)
    print(f"[{index}] Confirmed in block {receipt['blockNumber']}")

    return receipt


async def main():
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
    w3 = await SEISMIC_TESTNET.async_wallet_client(private_key)

    try:
        ABI = [
            {
                "name": "setValue",
                "type": "function",
                "inputs": [{"name": "value", "type": "uint256"}],
                "outputs": [],
            }
        ]

        contract = w3.seismic.contract(os.environ["CONTRACT_ADDRESS"], ABI)

        # Send multiple transactions concurrently
        values = [10, 20, 30, 40, 50]

        tasks = [
            send_transaction_with_tracking(contract, value, i)
            for i, value in enumerate(values)
        ]

        # Wait for all to complete
        receipts = await asyncio.gather(*tasks)

        print(f"\nAll {len(receipts)} transactions confirmed!")

    finally:
        await w3.provider.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
```

## Error Handling with Retries

```python
import asyncio
import os
from typing import TypeVar, Callable, Any
from seismic_web3 import create_async_wallet_client, PrivateKey, SEISMIC_TESTNET
from web3.exceptions import Web3Exception

T = TypeVar("T")


async def retry_async(
    func: Callable[..., Any],
    *args: Any,
    max_retries: int = 3,
    backoff_base: float = 2.0,
    **kwargs: Any,
) -> T:
    """Retry an async function with exponential backoff."""
    for attempt in range(max_retries):
        try:
            return await func(*args, **kwargs)
        except Web3Exception as e:
            if attempt == max_retries - 1:
                print(f"Failed after {max_retries} attempts: {e}")
                raise

            wait_time = backoff_base ** attempt
            print(f"Attempt {attempt + 1} failed: {e}")
            print(f"Retrying in {wait_time} seconds...")
            await asyncio.sleep(wait_time)

    raise RuntimeError("Retry logic failed")


async def main():
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))

    # Create client with retry
    w3 = await retry_async(
        SEISMIC_TESTNET.async_wallet_client,
        private_key,
        max_retries=3,
    )

    try:
        ABI = [
            {
                "name": "setValue",
                "type": "function",
                "inputs": [{"name": "value", "type": "uint256"}],
                "outputs": [],
            }
        ]

        contract = w3.seismic.contract(os.environ["CONTRACT_ADDRESS"], ABI)

        # Execute write with retry
        tx_hash = await retry_async(
            contract.write.setValue,
            42,
            max_retries=3,
        )

        print(f"Transaction sent: {tx_hash.hex()}")

        # Wait for receipt with retry
        receipt = await retry_async(
            w3.eth.wait_for_transaction_receipt,
            tx_hash,
            timeout=60,
            max_retries=3,
        )

        print(f"Confirmed in block {receipt['blockNumber']}")

    finally:
        await w3.provider.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
```

## Context Manager Pattern

```python
import asyncio
import os
from typing import AsyncIterator
from contextlib import asynccontextmanager
from seismic_web3 import create_async_wallet_client, PrivateKey
from web3 import AsyncWeb3


@asynccontextmanager
async def seismic_client(rpc_url: str, private_key: PrivateKey) -> AsyncIterator[AsyncWeb3]:
    """Context manager for automatic client cleanup."""
    w3 = await create_async_wallet_client(rpc_url, private_key=private_key)
    try:
        yield w3
    finally:
        await w3.provider.disconnect()
        print("Client disconnected")


async def main():
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))

    # Automatic cleanup with context manager
    async with seismic_client(os.environ["RPC_URL"], private_key) as w3:
        chain_id = await w3.eth.chain_id
        block = await w3.eth.block_number
        print(f"Chain {chain_id}, block {block}")

        # Client automatically disconnects when exiting context

    print("Context exited, client cleaned up")


if __name__ == "__main__":
    asyncio.run(main())
```

## WebSocket Connection for Events

```python
import asyncio
import os
from seismic_web3 import create_async_wallet_client, PrivateKey, SRC20_ABI


async def watch_events_ws(token_address: str, duration: int = 30):
    """Watch Transfer events using WebSocket connection."""
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))

    # Create WebSocket client
    w3 = await create_async_wallet_client(
        os.environ["WS_URL"],
        private_key=private_key,
        ws=True,  # Enable WebSocket
    )

    try:
        # Get standard contract for event filtering
        contract = w3.eth.contract(address=token_address, abi=SRC20_ABI)

        # Create event filter
        event_filter = await contract.events.Transfer.create_filter(
            fromBlock="latest"
        )

        print(f"Watching Transfer events for {duration} seconds...")
        start_time = asyncio.get_event_loop().time()

        while asyncio.get_event_loop().time() - start_time < duration:
            # Get new events
            events = await event_filter.get_new_entries()

            for event in events:
                print(f"\nTransfer Event:")
                print(f"  Block: {event['blockNumber']}")
                print(f"  From: {event['args']['from']}")
                print(f"  To: {event['args']['to']}")
                print(f"  Value: {event['args']['value']}")

            await asyncio.sleep(1)  # Poll every second

        print("Stopped watching events")

    finally:
        await w3.provider.disconnect()


async def main():
    await watch_events_ws(os.environ["TOKEN_ADDRESS"], duration=30)


if __name__ == "__main__":
    asyncio.run(main())
```

## Connection Pooling Pattern

```python
import asyncio
import os
from typing import List
from seismic_web3 import create_async_wallet_client, PrivateKey, SEISMIC_TESTNET


class ClientPool:
    """Simple connection pool for async clients."""

    def __init__(self, rpc_url: str, private_key: PrivateKey, pool_size: int = 5):
        self.rpc_url = rpc_url
        self.private_key = private_key
        self.pool_size = pool_size
        self.clients: List = []
        self.semaphore = asyncio.Semaphore(pool_size)

    async def initialize(self):
        """Initialize the connection pool."""
        print(f"Initializing pool with {self.pool_size} clients...")
        for i in range(self.pool_size):
            client = await create_async_wallet_client(
                self.rpc_url,
                private_key=self.private_key,
            )
            self.clients.append(client)
            print(f"  Client {i+1} ready")

    async def cleanup(self):
        """Clean up all connections."""
        print("Cleaning up connection pool...")
        for client in self.clients:
            await client.provider.disconnect()
        self.clients.clear()

    def get_client(self):
        """Get a client from the pool (round-robin)."""
        if not self.clients:
            raise RuntimeError("Pool not initialized")
        # Simple round-robin
        client = self.clients[0]
        self.clients.rotate(1)  # Requires deque
        return client


async def worker(pool: ClientPool, worker_id: int, num_operations: int):
    """Worker that performs operations using a pooled client."""
    client = pool.get_client()

    for i in range(num_operations):
        async with pool.semaphore:
            block = await client.eth.block_number
            print(f"Worker {worker_id}, operation {i+1}: block {block}")
            await asyncio.sleep(0.1)


async def main():
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))

    # Create connection pool
    pool = ClientPool(os.environ["RPC_URL"], private_key, pool_size=3)
    await pool.initialize()

    try:
        # Spawn multiple workers
        workers = [
            worker(pool, worker_id=i, num_operations=5)
            for i in range(10)
        ]

        # Run all workers concurrently
        await asyncio.gather(*workers)

        print("All workers completed")

    finally:
        await pool.cleanup()


if __name__ == "__main__":
    asyncio.run(main())
```

## Timeout Handling

```python
import asyncio
import os
from seismic_web3 import create_async_wallet_client, PrivateKey, SEISMIC_TESTNET


async def operation_with_timeout(w3, timeout: float = 5.0):
    """Execute an operation with a timeout."""
    try:
        # Wrap operation in timeout
        result = await asyncio.wait_for(
            w3.eth.block_number,
            timeout=timeout,
        )
        print(f"Operation completed: {result}")
        return result

    except asyncio.TimeoutError:
        print(f"Operation timed out after {timeout} seconds")
        raise


async def main():
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
    w3 = await SEISMIC_TESTNET.async_wallet_client(private_key)

    try:
        # Operation with 5 second timeout
        await operation_with_timeout(w3, timeout=5.0)

        # Multiple operations with overall timeout
        operations = [
            w3.eth.chain_id,
            w3.eth.block_number,
            w3.eth.get_balance(w3.eth.default_account),
        ]

        results = await asyncio.wait_for(
            asyncio.gather(*operations),
            timeout=10.0,
        )

        print(f"All operations completed: {results}")

    except asyncio.TimeoutError:
        print("Overall timeout exceeded")
    finally:
        await w3.provider.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
```

## Rate Limiting

```python
import asyncio
import os
import time
from seismic_web3 import create_async_wallet_client, PrivateKey, SEISMIC_TESTNET


class RateLimiter:
    """Simple token bucket rate limiter."""

    def __init__(self, rate: float, burst: int):
        """
        Args:
            rate: Operations per second
            burst: Maximum burst size
        """
        self.rate = rate
        self.burst = burst
        self.tokens = burst
        self.last_update = time.monotonic()
        self.lock = asyncio.Lock()

    async def acquire(self):
        """Acquire a token, waiting if necessary."""
        async with self.lock:
            now = time.monotonic()
            elapsed = now - self.last_update

            # Add tokens based on elapsed time
            self.tokens = min(self.burst, self.tokens + elapsed * self.rate)
            self.last_update = now

            if self.tokens < 1:
                # Wait for token to become available
                wait_time = (1 - self.tokens) / self.rate
                await asyncio.sleep(wait_time)
                self.tokens = 0
            else:
                self.tokens -= 1


async def rate_limited_operation(w3, limiter: RateLimiter, operation_id: int):
    """Execute an operation with rate limiting."""
    await limiter.acquire()
    block = await w3.eth.block_number
    print(f"Operation {operation_id}: block {block}")


async def main():
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
    w3 = await SEISMIC_TESTNET.async_wallet_client(private_key)

    try:
        # Rate limiter: 5 operations per second, burst of 10
        limiter = RateLimiter(rate=5.0, burst=10)

        # Execute many operations (will be rate limited)
        operations = [
            rate_limited_operation(w3, limiter, i)
            for i in range(20)
        ]

        await asyncio.gather(*operations)
        print("All operations completed")

    finally:
        await w3.provider.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
```

## Complete Production-Ready Pattern

```python
import asyncio
import os
import logging
from typing import Optional
from contextlib import asynccontextmanager
from seismic_web3 import create_async_wallet_client, PrivateKey, SEISMIC_TESTNET
from web3.exceptions import Web3Exception

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class SeismicAsyncClient:
    """Production-ready async Seismic client wrapper."""

    def __init__(self, rpc_url: str, private_key: PrivateKey):
        self.rpc_url = rpc_url
        self.private_key = private_key
        self.w3: Optional = None
        self.connected = False

    async def connect(self, max_retries: int = 3):
        """Connect with retry logic."""
        for attempt in range(max_retries):
            try:
                self.w3 = await create_async_wallet_client(
                    self.rpc_url,
                    private_key=self.private_key,
                )
                self.connected = True
                logger.info(f"Connected to {self.rpc_url}")
                return

            except Web3Exception as e:
                logger.warning(f"Connection attempt {attempt + 1} failed: {e}")
                if attempt < max_retries - 1:
                    await asyncio.sleep(2 ** attempt)
                else:
                    raise

    async def disconnect(self):
        """Disconnect and cleanup."""
        if self.w3 and self.connected:
            await self.w3.provider.disconnect()
            self.connected = False
            logger.info("Disconnected")

    @asynccontextmanager
    async def transaction(self):
        """Context manager for transaction operations."""
        if not self.connected:
            await self.connect()

        try:
            yield self.w3
        except Exception as e:
            logger.error(f"Transaction error: {e}")
            raise

    async def execute_with_timeout(self, coro, timeout: float = 30.0):
        """Execute coroutine with timeout."""
        try:
            return await asyncio.wait_for(coro, timeout=timeout)
        except asyncio.TimeoutError:
            logger.error(f"Operation timed out after {timeout} seconds")
            raise


async def main():
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))

    # Create client
    client = SeismicAsyncClient(os.environ["RPC_URL"], private_key)

    try:
        # Connect
        await client.connect()

        # Execute operations within transaction context
        async with client.transaction() as w3:
            # Operation with timeout
            chain_id = await client.execute_with_timeout(
                w3.eth.chain_id,
                timeout=5.0,
            )
            logger.info(f"Chain ID: {chain_id}")

            # Contract interaction
            ABI = [
                {
                    "name": "getValue",
                    "type": "function",
                    "inputs": [],
                    "outputs": [{"name": "", "type": "uint256"}],
                }
            ]

            contract = w3.seismic.contract(os.environ["CONTRACT_ADDRESS"], ABI)
            result = await client.execute_with_timeout(
                contract.read.getValue(),
                timeout=10.0,
            )

            value = int.from_bytes(result[-32:], "big")
            logger.info(f"Contract value: {value}")

    except Exception as e:
        logger.error(f"Error: {e}")
    finally:
        await client.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
```

## Expected Output

```
Fetching data concurrently...
Chain ID: 31337
Block: 12345
Balance: 100.0 ETH
TEE PK: 0x028e76821eb4d77f...

Contract values:
  Value 0: 10
  Value 1: 20
  Value 2: 30
  Value 3: 40
  Value 4: 50
```

## Best Practices

1. **Always disconnect** - Use `finally` blocks or context managers
2. **Concurrent operations** - Use `asyncio.gather()` for independent operations
3. **Error handling** - Implement retries with exponential backoff
4. **Timeouts** - Set reasonable timeouts for all operations
5. **Rate limiting** - Respect RPC provider rate limits
6. **Connection pooling** - For high-throughput applications
7. **WebSocket for events** - Use WebSocket connections for event streaming
8. **Logging** - Add comprehensive logging for production

## Next Steps

- [Basic Wallet Setup](basic-wallet-setup.md) - Client setup
- [Shielded Write Complete](shielded-write-complete.md) - Transaction patterns
- [SRC20 Workflow](src20-workflow.md) - Token operations
- [Signed Read Pattern](signed-read-pattern.md) - Read operations

## See Also

- [create_async_wallet_client](../client/create-async-wallet-client.md) - API reference
- [AsyncShieldedContract](../contract/async-shielded-contract.md) - Async contract API
- [AsyncSeismicNamespace](../namespaces/async-seismic-namespace.md) - Async namespace
- [asyncio documentation](https://docs.python.org/3/library/asyncio.html) - Python async
