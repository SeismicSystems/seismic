---
description: Network configurations and chain constants
icon: link
---

# Chains

The Seismic SDK provides pre-configured network settings and utilities for connecting to Seismic chains. Chain configurations encapsulate RPC endpoints, WebSocket URLs, chain IDs, and convenience methods for creating clients.

## Overview

Instead of manually passing RPC URLs and chain IDs when creating clients, you can use pre-defined chain configurations:

```python
import os
from seismic_web3 import SEISMIC_TESTNET, SANVIL, PrivateKey

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))

# Connect to testnet
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Connect to local Sanvil
w3 = SANVIL.wallet_client(pk)
```

## Components

### Chain Configurations

| Component | Description |
|-----------|-------------|
| [ChainConfig](chain-config.md) | Immutable dataclass for network configuration |
| [SEISMIC_TESTNET](seismic-testnet.md) | Public testnet configuration (GCP-1) |
| [SANVIL](sanvil.md) | Local development network configuration |
| [make_seismic_testnet](make-seismic-testnet.md) | Factory for alternate testnet instances |

### Protocol Constants

| Constant | Description |
|----------|-------------|
| [SEISMIC_TX_TYPE](seismic-tx-type.md) | Transaction type byte for Seismic transactions |

## Quick Start

### Using Pre-Defined Chains

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))

# Wallet client (requires private key)
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Async wallet (auto-selects ws_url when ws=True)
w3 = await SEISMIC_TESTNET.async_wallet_client(pk, ws=True)

# Public client (no private key needed)
public = SEISMIC_TESTNET.public_client()
```

### Accessing Chain Properties

```python
from seismic_web3 import SEISMIC_TESTNET, SANVIL

# Testnet properties
SEISMIC_TESTNET.rpc_url   # "https://gcp-1.seismictest.net/rpc"
SEISMIC_TESTNET.ws_url    # "wss://gcp-1.seismictest.net/ws"
SEISMIC_TESTNET.chain_id  # 5124
SEISMIC_TESTNET.name      # "Seismic Testnet (GCP-1)"

# Sanvil properties
SANVIL.rpc_url   # "http://127.0.0.1:8545"
SANVIL.ws_url    # "ws://127.0.0.1:8545"
SANVIL.chain_id  # 31337
SANVIL.name      # "Sanvil (local)"
```

### Alternate Testnet Instances

```python
from seismic_web3 import make_seismic_testnet

# Connect to different GCP testnet instances
testnet_2 = make_seismic_testnet(2)  # gcp-2.seismictest.net
testnet_3 = make_seismic_testnet(3)  # gcp-3.seismictest.net

w3 = testnet_2.wallet_client(pk)
```

### Custom Chains

```python
from seismic_web3 import ChainConfig, PrivateKey

custom = ChainConfig(
    chain_id=5124,
    rpc_url="https://gcp-1.seismictest.net/rpc",
    ws_url="wss://gcp-1.seismictest.net/ws",
    name="Seismic Testnet",
)

pk = PrivateKey(...)
w3 = custom.wallet_client(pk)
```

## Client Creation Methods

All chain configurations provide convenience methods for creating clients:

### Wallet Clients (Require Private Key)

```python
# Sync wallet client
w3 = chain.wallet_client(private_key, encryption_sk=None)

# Async wallet client (HTTP)
w3 = await chain.async_wallet_client(private_key, encryption_sk=None, ws=False)

# Async wallet client (WebSocket)
w3 = await chain.async_wallet_client(private_key, ws=True)
```

### Public Clients (No Private Key)

```python
# Sync public client
public = chain.public_client()

# Async public client (HTTP)
public = chain.async_public_client(ws=False)

# Async public client (WebSocket)
public = chain.async_public_client(ws=True)
```

## Chain IDs

The SDK defines constants for chain IDs used in transaction signing:

```python
from seismic_web3 import SEISMIC_TESTNET_CHAIN_ID, SANVIL_CHAIN_ID

SEISMIC_TESTNET_CHAIN_ID  # 5124
SANVIL_CHAIN_ID           # 31337
```

These are automatically included in the chain configurations and used by the signing infrastructure.

## Notes

- Chain configurations are **immutable** (frozen dataclasses)
- WebSocket support is automatic when `ws=True` and `ws_url` is configured
- HTTP endpoints are used as fallback if WebSocket URL is unavailable
- All methods return standard `web3.py` instances with the `w3.seismic` namespace attached

## See Also

- [Client Creation](../client/) - Detailed client setup documentation
- [Wallet Client](../client/create-wallet-client.md) - Full wallet client capabilities
- [Public Client](../client/create-public-client.md) - Read-only client features
- [PrivateKey](../api-reference/types/private-key.md) - Private key type documentation
