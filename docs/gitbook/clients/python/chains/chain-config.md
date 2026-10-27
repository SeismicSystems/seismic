---
description: Immutable network configuration dataclass
icon: gear
---

# ChainConfig

Immutable configuration for a Seismic network, including RPC endpoints, chain ID, and client factory methods.

> Balance semantics depend on the backend, not the chain configuration. On reth with PR #502, `get_balance()` returns a compatibility placeholder, not native or sUSDC holdings. See [Balance RPCs](../../../reference/balance-rpcs.md).

## Overview

`ChainConfig` is a frozen dataclass that encapsulates all information needed to connect to a Seismic network. Instead of passing RPC URLs and chain IDs separately, you create a `ChainConfig` once and use its convenience methods to create clients.

## Definition

```python
@dataclass(frozen=True)
class ChainConfig:
    """Immutable configuration for a Seismic network.

    Attributes:
        chain_id: Numeric chain identifier.
        rpc_url: HTTP(S) JSON-RPC endpoint.
        ws_url: WebSocket endpoint (``None`` if not available).
        name: Human-readable network name.
    """
    chain_id: int
    rpc_url: str
    ws_url: str | None = None
    name: str = ""
    explorer_url: str | None = None
```

## Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `chain_id` | `int` | Yes | Numeric chain identifier (e.g., `5124` for testnet) |
| `rpc_url` | `str` | Yes | HTTP(S) JSON-RPC endpoint |
| `ws_url` | `str \| None` | No | WebSocket endpoint (default: `None`) |
| `name` | `str` | No | Human-readable network name (default: `""`) |
| `explorer_url` | `str \| None` | No | Block explorer URL (default: `None`) |

## Construction

### Basic Usage

```python
from seismic_web3 import ChainConfig

config = ChainConfig(
    chain_id=5124,
    rpc_url="https://testnet-1.seismictest.net/rpc",
    ws_url="wss://testnet-1.seismictest.net/ws",
    name="Seismic Testnet",
)
```

### Without WebSocket

```python
config = ChainConfig(
    chain_id=5124,
    rpc_url="https://testnet-1.seismictest.net/rpc",
    name="Seismic Testnet",
)
```

### Using Pre-Defined Configs

```python
from seismic_web3 import SEISMIC_TESTNET, SANVIL

# Pre-configured testnet
SEISMIC_TESTNET

# Pre-configured local Sanvil
SANVIL
```

## Methods

### wallet_client()

Create a synchronous `Web3` instance with wallet capabilities.

```python
def wallet_client(
    self,
    private_key: PrivateKey,
    *,
    encryption_sk: PrivateKey | None = None,
) -> Web3:
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `private_key` | `PrivateKey` | Yes | 32-byte signing key for transactions |
| `encryption_sk` | `PrivateKey \| None` | No | Optional 32-byte key for ECDH |

#### Returns

- `Web3` - A synchronous `Web3` instance with [`w3.seismic`](../namespaces/seismic-namespace.md) namespace attached

#### Example

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Now use w3.seismic methods
balance = w3.eth.get_balance("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
```

***

### async_wallet_client()

Create an asynchronous `Web3` instance with wallet capabilities.

```python
async def async_wallet_client(
    self,
    private_key: PrivateKey,
    *,
    encryption_sk: PrivateKey | None = None,
    ws: bool = False,
) -> AsyncWeb3:
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `private_key` | `PrivateKey` | Yes | 32-byte signing key for transactions |
| `encryption_sk` | `PrivateKey \| None` | No | Optional 32-byte key for ECDH |
| `ws` | `bool` | No | If `True`, uses `WebSocketProvider` (default: `False`) |

#### Returns

- `AsyncWeb3` - An asynchronous `Web3` instance with [`w3.seismic`](../namespaces/async-seismic-namespace.md) namespace attached

#### Behavior

- When `ws=True` and `ws_url` is set, the WebSocket URL is used automatically
- When `ws=False`, the HTTP `rpc_url` is used
- Raises `ValueError` if `ws=True` but `ws_url` is `None`

#### Example

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

# HTTP
w3 = await SEISMIC_TESTNET.async_wallet_client(pk)

# WebSocket
w3 = await SEISMIC_TESTNET.async_wallet_client(pk, ws=True)

# Use async methods
balance = await w3.eth.get_balance("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
```

***

### public_client()

Create a synchronous `Web3` instance with public (read-only) access.

```python
def public_client(self) -> Web3:
```

#### Parameters

None. (public clients don't handle private keys)

#### Returns

- `Web3` - A synchronous `Web3` instance with [`w3.seismic`](../namespaces/seismic-public-namespace.md) namespace attached (read-only)

#### Example

```python
from seismic_web3 import SEISMIC_TESTNET

public = SEISMIC_TESTNET.public_client()

# Read-only operations
block = public.eth.get_block("latest")
tee_key = public.seismic.get_tee_public_key()
```

***

### async_public_client()

Create an asynchronous `Web3` instance with public (read-only) access.

```python
def async_public_client(
    self,
    *,
    ws: bool = False,
) -> AsyncWeb3:
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `ws` | `bool` | No | If `True`, uses `WebSocketProvider` (default: `False`) |

#### Returns

- `AsyncWeb3` - An asynchronous `Web3` instance with [`w3.seismic`](../namespaces/async-seismic-public-namespace.md) namespace attached (read-only)

#### Example

```python
from seismic_web3 import SEISMIC_TESTNET

# HTTP
public = SEISMIC_TESTNET.async_public_client()

# WebSocket
public = SEISMIC_TESTNET.async_public_client(ws=True)

# Read-only operations
block = await public.eth.get_block("latest")
```

***

### Explorer URL Helpers

When a `ChainConfig` has an `explorer_url` configured (such as [`SEISMIC_TESTNET`](seismic-testnet.md)), you can generate block explorer URLs for transactions, addresses, tokens, and blocks directly from the configuration or via attached clients.

#### Methods

- `tx_explorer_url(tx_hash, *, tab=None)` - Explorer URL for a transaction
- `address_explorer_url(address, *, tab=None)` - Explorer URL for an account or contract address
- `block_explorer_url(block_number, *, tab=None)` - Explorer URL for a block
- `token_explorer_url(address, *, tab=None)` - Explorer URL for an SRC20 or ERC20 token
- `get_explorer_url(*, item=None, id=None, tab=None)` - Low-level URL generator

Each helper accepts an optional `tab` parameter to deep-link into a specific explorer view, and returns `None` if the chain has no configured explorer (such as local [`SANVIL`](sanvil.md)).

#### Example

```python
from seismic_web3 import SEISMIC_TESTNET

# Transaction URL
tx_url = SEISMIC_TESTNET.tx_explorer_url("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890")
# "https://seismic-testnet.socialscan.io/tx/0xabcdef..."

# Transaction URL with tab
tx_logs = SEISMIC_TESTNET.tx_explorer_url("0xabcdef...", tab="logs")
# "https://seismic-testnet.socialscan.io/tx/0xabcdef...?tab=logs"

# Address URL
addr_url = SEISMIC_TESTNET.address_explorer_url("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
# "https://seismic-testnet.socialscan.io/address/0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

# Token URL
token_url = SEISMIC_TESTNET.token_explorer_url("0xYourTokenAddress", tab="holders")
# "https://seismic-testnet.socialscan.io/token/0xYourTokenAddress?tab=holders"
```

## Deprecated Methods

### create_client()

Deprecated: use `wallet_client()` instead.

```python
# Old (deprecated)
w3 = config.create_client(private_key)

# New
w3 = config.wallet_client(private_key)
```

### create_async_client()

Deprecated: use `async_wallet_client()` instead.

```python
# Old (deprecated)
w3 = await config.create_async_client(private_key)

# New
w3 = await config.async_wallet_client(private_key)
```

## Notes

- All client creation methods return standard `web3.py` instances
- The [`w3.seismic`](../namespaces/seismic-namespace.md) namespace is automatically attached to all clients
- WebSocket connections provide better performance for subscription-based workflows
- HTTP connections are suitable for simple request-response patterns

## See Also

- [SEISMIC_TESTNET](seismic-testnet.md) - Pre-defined testnet configuration
- [SANVIL](sanvil.md) - Pre-defined local development configuration
- [make_seismic_testnet](make-seismic-testnet.md) - Factory for testnet instances
- [create_wallet_client](../client/create-wallet-client.md) - Direct wallet client creation
- [create_public_client](../client/create-public-client.md) - Direct public client creation
- [PrivateKey](../api-reference/types/private-key.md) - Private key type
