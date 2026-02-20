---
description: Immutable network configuration dataclass
icon: gear
---

# ChainConfig

Immutable configuration for a Seismic network, including RPC endpoints, chain ID, and client factory methods.

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
```

## Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `chain_id` | `int` | Yes | Numeric chain identifier (e.g., `5124` for testnet) |
| `rpc_url` | `str` | Yes | HTTP(S) JSON-RPC endpoint |
| `ws_url` | `str \| None` | No | WebSocket endpoint (default: `None`) |
| `name` | `str` | No | Human-readable network name (default: `""`) |

## Construction

### Basic Usage

```python
from seismic_web3 import ChainConfig

config = ChainConfig(
    chain_id=5124,
    rpc_url="https://gcp-1.seismictest.net/rpc",
    ws_url="wss://gcp-1.seismictest.net/ws",
    name="Seismic Testnet",
)
```

### Without WebSocket

```python
config = ChainConfig(
    chain_id=5124,
    rpc_url="https://gcp-1.seismictest.net/rpc",
    name="Seismic Testnet",
)
```

### Using Pre-Defined Configs

```python
from seismic_web3 import SEISMIC_TESTNET, SANVIL

# Pre-configured testnet
testnet = SEISMIC_TESTNET

# Pre-configured local Sanvil
sanvil = SANVIL
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

- `Web3` - A synchronous `Web3` instance with `w3.seismic` namespace attached

#### Example

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Now use w3.seismic methods
balance = w3.eth.get_balance("0xYourAddress")
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

- `AsyncWeb3` - An asynchronous `Web3` instance with `w3.seismic` namespace attached

#### Behavior

- When `ws=True` and `ws_url` is set, the WebSocket URL is used automatically
- When `ws=False` or `ws_url` is `None`, the HTTP `rpc_url` is used

#### Example

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))

# HTTP
w3 = await SEISMIC_TESTNET.async_wallet_client(pk)

# WebSocket
w3 = await SEISMIC_TESTNET.async_wallet_client(pk, ws=True)

# Use async methods
balance = await w3.eth.get_balance("0xYourAddress")
```

***

### public_client()

Create a synchronous `Web3` instance with public (read-only) access.

```python
def public_client(self) -> Web3:
```

#### Parameters

None. No private key required.

#### Returns

- `Web3` - A synchronous `Web3` instance with `w3.seismic` namespace attached (read-only)

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
async def async_public_client(
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

- `AsyncWeb3` - An asynchronous `Web3` instance with `w3.seismic` namespace attached (read-only)

#### Example

```python
from seismic_web3 import SEISMIC_TESTNET

# HTTP
public = await SEISMIC_TESTNET.async_public_client()

# WebSocket
public = await SEISMIC_TESTNET.async_public_client(ws=True)

# Read-only operations
block = await public.eth.get_block("latest")
```

## Properties

- **Immutable** - Cannot be modified after construction (frozen dataclass)
- **Type-Safe** - All attributes are type-checked at construction
- **Self-Contained** - Includes all information needed to connect to the network

## Examples

### Custom Network Configuration

```python
from seismic_web3 import ChainConfig, PrivateKey

# Create custom config
my_network = ChainConfig(
    chain_id=5124,
    rpc_url="https://gcp-1.seismictest.net/rpc",
    ws_url="wss://gcp-1.seismictest.net/ws",
    name="Seismic Testnet",
)

# Use it
pk = PrivateKey(...)
w3 = my_network.wallet_client(pk)
```

### Multiple Environments

```python
from seismic_web3 import ChainConfig, SEISMIC_TESTNET, SANVIL

# Different environments
environments = {
    "testnet": SEISMIC_TESTNET,
    "local": SANVIL,
    "staging": ChainConfig(
        chain_id=5124,
        rpc_url="https://gcp-1.seismictest.net/rpc",
        name="Staging",
    ),
}

# Select environment
env = "testnet"
w3 = environments[env].wallet_client(pk)
```

### Accessing Configuration

```python
from seismic_web3 import SEISMIC_TESTNET

# Read configuration properties
print(f"Chain ID: {SEISMIC_TESTNET.chain_id}")
print(f"RPC URL: {SEISMIC_TESTNET.rpc_url}")
print(f"WS URL: {SEISMIC_TESTNET.ws_url}")
print(f"Name: {SEISMIC_TESTNET.name}")
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
- The `w3.seismic` namespace is automatically attached to all clients
- WebSocket connections provide better performance for subscription-based workflows
- HTTP connections are suitable for simple request-response patterns
- Chain ID is used internally for EIP-712 transaction signing

## See Also

- [SEISMIC_TESTNET](seismic-testnet.md) - Pre-defined testnet configuration
- [SANVIL](sanvil.md) - Pre-defined local development configuration
- [make_seismic_testnet](make-seismic-testnet.md) - Factory for testnet instances
- [create_wallet_client](../client/create-wallet-client.md) - Direct wallet client creation
- [create_public_client](../client/create-public-client.md) - Direct public client creation
- [PrivateKey](../api-reference/types/private-key.md) - Private key type
