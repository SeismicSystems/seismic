---
description: Create sync Web3 instance with public (read-only) Seismic access
icon: eye
---

# create_public_client

Create a synchronous `Web3` instance with public (read-only) Seismic access.

## Overview

`create_public_client()` creates a client for read-only operations on the Seismic network. No private key is required. The [`w3.seismic`](../namespaces/seismic-public-namespace.md) namespace provides only public read operations: [`get_tee_public_key()`](../namespaces/methods/get-tee-public-key.md), [`get_deposit_root()`](../namespaces/methods/get-deposit-root.md), [`get_deposit_count()`](../namespaces/methods/get-deposit-count.md), and [`contract()`](../contract/) (with `.tread` only).

This is useful for applications that only need to query chain state without submitting transactions, such as block explorers, analytics dashboards, or read-only dApps.

## Signature

```python
def create_public_client(rpc_url: str) -> Web3
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `rpc_url` | `str` | Yes | HTTP(S) URL of the Seismic node (e.g., `"https://gcp-1.seismictest.net/rpc"`). WebSocket URLs are not supported — see note below |

## Returns

| Type | Description |
|------|-------------|
| `Web3` | A `Web3` instance with `w3.seismic` namespace attached ([`SeismicPublicNamespace`](../namespaces/seismic-public-namespace.md)) |

## Examples

### Basic Usage

```python
from seismic_web3 import create_public_client

# Create public client
w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

# Query TEE public key
tee_pk = w3.seismic.get_tee_public_key()
print(f"TEE public key: {tee_pk.to_0x_hex()}")

# Query deposit info
root = w3.seismic.get_deposit_root()
count = w3.seismic.get_deposit_count()
print(f"Deposit root: {root.to_0x_hex()}, count: {count}")
```

### Using Chain Configuration

```python
from seismic_web3 import SEISMIC_TESTNET

# Recommended: use chain config instead of raw URL
w3 = SEISMIC_TESTNET.public_client()

# Equivalent to:
# w3 = create_public_client(SEISMIC_TESTNET.rpc_url)
```

### Read-Only Contract Access

```python
from seismic_web3 import create_public_client

w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

# Create contract wrapper (read-only)
contract = w3.seismic.contract(
    address="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    abi=contract_abi,
)

# Only transparent reads are available
result = contract.tread.balanceOf("0x1234...")

# Shielded operations are NOT available
# contract.swrite.transfer(...)  # AttributeError: no swrite on public client
# contract.sread.getBalance(...)  # AttributeError: no sread on public client
```

### Standard Web3 Operations

```python
from seismic_web3 import create_public_client

w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

# All standard web3.py read operations work
block = w3.eth.get_block("latest")
print(f"Latest block: {block['number']}")

balance = w3.eth.get_balance("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
print(f"Balance: {w3.from_wei(balance, 'ether')} ETH")

chain_id = w3.eth.chain_id
print(f"Chain ID: {chain_id}")
```

### Block Explorer Pattern

```python
from seismic_web3 import create_public_client

def get_chain_stats(rpc_url: str):
    w3 = create_public_client(rpc_url)

    # Get latest block
    block = w3.eth.get_block("latest")

    # Get deposit info
    deposit_root = w3.seismic.get_deposit_root()
    deposit_count = w3.seismic.get_deposit_count()

    # Get TEE info
    tee_pk = w3.seismic.get_tee_public_key()

    return {
        "block_number": block["number"],
        "block_hash": block["hash"].to_0x_hex(),
        "deposit_root": deposit_root.to_0x_hex(),
        "deposit_count": deposit_count,
        "tee_public_key": tee_pk.to_0x_hex(),
    }

stats = get_chain_stats("https://gcp-1.seismictest.net/rpc")
print(stats)
```

## How It Works

The function performs two steps:

1. **Create Web3 instance**
   ```python
   w3 = Web3(Web3.HTTPProvider(rpc_url))
   ```

2. **Attach public Seismic namespace**
   ```python
   w3.seismic = SeismicPublicNamespace(w3)
   ```

No TEE public key fetching or encryption setup is performed since the client cannot perform shielded operations.

## Client Capabilities

### Standard Web3 Methods (e.g. `w3.eth`, `w3.net`)
- `get_block()`, `get_transaction()`, `get_balance()`
- `get_code()`, `call()`, `estimate_gas()`
- All other standard read-only `web3.py` functionality

### Public Seismic Methods (`w3.seismic`)
- [`get_tee_public_key()`](../namespaces/methods/get-tee-public-key.md) - Get TEE public key
- [`get_deposit_root()`](../namespaces/methods/get-deposit-root.md) - Query deposit merkle root
- [`get_deposit_count()`](../namespaces/methods/get-deposit-count.md) - Query deposit count
- [`contract()`](../contract/) - Create contract wrappers (`.tread` only)

### NOT Available
- [`send_shielded_transaction()`](../namespaces/methods/send-shielded-transaction.md) - Requires private key
- [`debug_send_shielded_transaction()`](../namespaces/methods/debug-send-shielded-transaction.md) - Requires private key
- [`signed_call()`](../namespaces/methods/signed-call.md) - Requires private key and encryption
- [`deposit()`](../namespaces/methods/deposit.md) - Requires private key
- Contract `.swrite` and `.sread` methods - Require private key

## Public vs Wallet Client

| Feature | Public Client | Wallet Client |
|---------|--------------|---------------|
| **Private key** | Not required | Required |
| **Shielded writes** | No | Yes |
| **Signed reads** | No | Yes |
| **Transparent reads** | Yes | Yes |
| **Deposits** | No | Yes |
| **TEE queries** | Yes | Yes |
| **Standard Web3** | All read operations | All operations |

## Notes

- **HTTP only** — Sync clients use `Web3` with `HTTPProvider`, which does not support WebSocket connections. This is a limitation of the underlying `web3.py` library (`WebSocketProvider` is async-only). If you need WebSocket support (persistent connections, subscriptions), use [`create_async_public_client()`](create-async-public-client.md) with `ws=True`
- No private key required or accepted
- No encryption setup performed
- No RPC calls during client creation (lightweight)
- Cannot perform any write operations or shielded reads
- Contract wrappers only expose `.tread` (transparent read)
- For write operations, use [`create_wallet_client()`](create-wallet-client.md)
- For async operations, use [`create_async_public_client()`](create-async-public-client.md)

## Use Cases

- Block explorers and chain analytics
- Read-only dApps that display public data
- Monitoring and alerting systems
- Price oracles and data aggregators
- Public dashboards and visualizations
- Testing and validation tools

## See Also

- [create_async_public_client](create-async-public-client.md) - Async variant with WebSocket support
- [create_wallet_client](create-wallet-client.md) - Full-featured client with private key
- [SeismicPublicNamespace](../namespaces/seismic-public-namespace.md) - The public `w3.seismic` namespace
- [Chains Configuration](../chains.md) - Pre-configured chain constants
- [Contract Instances](../contract-instance.md) - Working with contract wrappers
