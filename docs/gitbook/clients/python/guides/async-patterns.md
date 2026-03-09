---
description: Concurrent operations and async best practices
icon: bolt
---

# Async Patterns

The async client mirrors the sync API but requires `await` on all operations.

## Setup

```python
import asyncio
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])


async def main():
    w3 = await SEISMIC_TESTNET.async_wallet_client(pk)

    try:
        chain_id = await w3.eth.chain_id
        block = await w3.eth.block_number
        print(f"Chain {chain_id}, block {block}")
    finally:
        await w3.provider.disconnect()


asyncio.run(main())
```

## Concurrent reads with `asyncio.gather`

```python
import asyncio
from eth_abi import decode
from seismic_web3 import SRC20_ABI

# ... setup as above ...

token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)

# Fetch metadata and balance concurrently
name_raw, symbol_raw, balance_raw = await asyncio.gather(
    token.tread.name(),
    token.tread.symbol(),
    token.read.balanceOf(),
)

name = decode(["string"], bytes(name_raw))[0]
symbol = decode(["string"], bytes(symbol_raw))[0]
balance = decode(["uint256"], bytes(balance_raw))[0]
print(f"{name} ({symbol}): {balance}")
```

## Context manager for cleanup

```python
from contextlib import asynccontextmanager
from seismic_web3 import create_async_wallet_client, PrivateKey


@asynccontextmanager
async def seismic_client(rpc_url: str, private_key: PrivateKey):
    w3 = await create_async_wallet_client(rpc_url, private_key=private_key)
    try:
        yield w3
    finally:
        await w3.provider.disconnect()


async def main():
    pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

    async with seismic_client("https://gcp-1.seismictest.net/rpc", pk) as w3:
        chain_id = await w3.eth.chain_id
        print(f"Chain {chain_id}")
    # Client automatically disconnected
```

## WebSocket connection

Use `ws=True` for event streaming:

```python
from seismic_web3 import create_async_wallet_client

w3 = await create_async_wallet_client(
    "wss://gcp-1.seismictest.net/ws",
    private_key=pk,
    ws=True,
)
```

## See Also

- [create\_async\_wallet\_client](../client/create-async-wallet-client.md) — Async client API reference
- [AsyncShieldedContract](../contract/async-shielded-contract.md) — Async contract wrapper
- [Event Watching](../src20/event-watching/) — Decrypt SRC20 events
