---
description: Pre-configured Seismic public testnet
icon: cloud
---

# SEISMIC_TESTNET

Pre-defined [`ChainConfig`](chain-config.md) for the Seismic public testnet (GCP instance 1).

## Overview

`SEISMIC_TESTNET` is a ready-to-use chain configuration pointing to the primary Seismic testnet instance. It's the recommended starting point for developers building on Seismic.

## Definition

```python
SEISMIC_TESTNET: ChainConfig = make_seismic_testnet(1)
```

Internally, this is equivalent to:

```python
ChainConfig(
    chain_id=5124,
    rpc_url="https://gcp-1.seismictest.net/rpc",
    ws_url="wss://gcp-1.seismictest.net/ws",
    name="Seismic Testnet (GCP-1)",
)
```

## Configuration

| Property | Value |
|----------|-------|
| `chain_id` | `5124` |
| `rpc_url` | `"https://gcp-1.seismictest.net/rpc"` |
| `ws_url` | `"wss://gcp-1.seismictest.net/ws"` |
| `name` | `"Seismic Testnet (GCP-1)"` |

## Usage

### Import and Access Properties

```python
from seismic_web3 import SEISMIC_TESTNET

# Access configuration
print(SEISMIC_TESTNET.rpc_url)   # "https://gcp-1.seismictest.net/rpc"
print(SEISMIC_TESTNET.ws_url)    # "wss://gcp-1.seismictest.net/ws"
print(SEISMIC_TESTNET.chain_id)  # 5124
print(SEISMIC_TESTNET.name)      # "Seismic Testnet (GCP-1)"
```

### Create Wallet Client (Sync)

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Now use w3.seismic methods
balance = w3.eth.get_balance("0xYourAddress")
```

### Create Wallet Client (Async)

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))

# HTTP
w3 = await SEISMIC_TESTNET.async_wallet_client(pk)

# WebSocket (auto-selects wss://gcp-1.seismictest.net/ws)
w3 = await SEISMIC_TESTNET.async_wallet_client(pk, ws=True)
```

### Create Public Client

```python
from seismic_web3 import SEISMIC_TESTNET

# Sync
public = SEISMIC_TESTNET.public_client()

# Async
public = SEISMIC_TESTNET.async_public_client()

# Read-only operations
block = await public.eth.get_block("latest")
tee_key = await public.seismic.get_tee_public_key()
```

## Examples

### Basic Shielded Transaction

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Get contract
contract = w3.seismic.contract(
    address="0xYourContractAddress",
    abi=[...],
)

# Send shielded write
tx_hash = contract.swrite.your_function(arg1, arg2)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
```

### Using with Environment Variables

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

# Load private key from environment
pk = PrivateKey(os.environ["PRIVATE_KEY"])

# Create client
w3 = SEISMIC_TESTNET.wallet_client(pk)
```

### Checking Connection

```python
from seismic_web3 import SEISMIC_TESTNET

public = SEISMIC_TESTNET.public_client()

# Verify connection
try:
    chain_id = public.eth.chain_id
    block_number = public.eth.block_number
    print(f"Connected to chain {chain_id} at block {block_number}")
except Exception as e:
    print(f"Connection failed: {e}")
```

## Notes

- This is the primary public testnet instance
- Suitable for development and testing
- WebSocket endpoint is available for subscriptions
- Chain ID `5124` is used for EIP-712 transaction signing
- For alternate testnet instances, use [`make_seismic_testnet(n)`](make-seismic-testnet.md)

## Chain ID Constant

The chain ID is also available as a standalone constant:

```python
from seismic_web3 import SEISMIC_TESTNET_CHAIN_ID

SEISMIC_TESTNET_CHAIN_ID  # 5124
```

## See Also

- [ChainConfig](chain-config.md) - Chain configuration dataclass
- [make_seismic_testnet](make-seismic-testnet.md) - Factory for alternate testnet instances
- [SANVIL](sanvil.md) - Local development network
- [create_wallet_client](../client/create-wallet-client.md) - Direct client creation
- [PrivateKey](../api-reference/types/private-key.md) - Private key type
