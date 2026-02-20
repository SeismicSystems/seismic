---
description: Pre-configured local development network
icon: computer
---

# SANVIL

Pre-defined [`ChainConfig`](chain-config.md) for the local Sanvil (Seismic Anvil) development network.

## Overview

`SANVIL` is a ready-to-use chain configuration pointing to a local Seismic node running on `127.0.0.1:8545`. It's designed for local development and testing workflows.

## Definition

```python
SANVIL: ChainConfig = ChainConfig(
    chain_id=31337,
    rpc_url="http://127.0.0.1:8545",
    ws_url="ws://127.0.0.1:8545",
    name="Sanvil (local)",
)
```

## Configuration

| Property | Value |
|----------|-------|
| `chain_id` | `31337` |
| `rpc_url` | `"http://127.0.0.1:8545"` |
| `ws_url` | `"ws://127.0.0.1:8545"` |
| `name` | `"Sanvil (local)"` |

## Usage

### Import and Access Properties

```python
from seismic_web3 import SANVIL

# Access configuration
print(SANVIL.rpc_url)   # "http://127.0.0.1:8545"
print(SANVIL.ws_url)    # "ws://127.0.0.1:8545"
print(SANVIL.chain_id)  # 31337
print(SANVIL.name)      # "Sanvil (local)"
```

### Create Wallet Client (Sync)

```python
import os
from seismic_web3 import SANVIL, PrivateKey

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SANVIL.wallet_client(pk)

# Now use w3.seismic methods
balance = w3.eth.get_balance("0xYourAddress")
```

### Create Wallet Client (Async)

```python
import os
from seismic_web3 import SANVIL, PrivateKey

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))

# HTTP
w3 = await SANVIL.async_wallet_client(pk)

# WebSocket
w3 = await SANVIL.async_wallet_client(pk, ws=True)
```

### Create Public Client

```python
from seismic_web3 import SANVIL

# Sync
public = SANVIL.public_client()

# Async
public = await SANVIL.async_public_client()

# Read-only operations
block = await public.eth.get_block("latest")
```

## Examples

### Local Development Workflow

```python
from seismic_web3 import SANVIL, PrivateKey

# Use well-known anvil test account
TEST_PRIVATE_KEY = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
pk = PrivateKey(TEST_PRIVATE_KEY)

w3 = SANVIL.wallet_client(pk)

# Deploy and interact with contracts locally
contract = w3.seismic.contract(
    address="0xYourContractAddress",
    abi=[...],
)
```

### Testing with Multiple Accounts

```python
from seismic_web3 import SANVIL, PrivateKey

# Anvil provides 10 test accounts with known private keys
accounts = [
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
]

# Create clients for different accounts
clients = [SANVIL.wallet_client(PrivateKey(pk)) for pk in accounts]
```

### Checking Local Node Connection

```python
from seismic_web3 import SANVIL

public = SANVIL.public_client()

try:
    chain_id = public.eth.chain_id
    block_number = public.eth.block_number
    print(f"Connected to local node (chain {chain_id}) at block {block_number}")
except Exception as e:
    print(f"Local node not running: {e}")
```

### Integration Testing

```python
import pytest
from seismic_web3 import SANVIL, PrivateKey

@pytest.fixture
def w3():
    """Fixture for local Sanvil client."""
    pk = PrivateKey("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
    return SANVIL.wallet_client(pk)

def test_shielded_write(w3):
    """Test shielded write on local node."""
    contract = w3.seismic.contract(address="0x...", abi=[...])
    tx_hash = contract.swrite.my_function(42)
    receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
    assert receipt.status == 1
```

## Prerequisites

To use `SANVIL`, you need a local Seismic node running. This is typically:

1. **Sanvil** (Seismic Anvil) - Local development node
2. Running on `127.0.0.1:8545`
3. Compatible with Seismic protocol extensions

## Notes

- Chain ID `31337` matches the default Anvil chain ID
- Suitable for local development and testing only
- Fast block times and instant finality
- No real value on chain
- WebSocket endpoint available for subscriptions
- Typically used with well-known test private keys

## Chain ID Constant

The chain ID is also available as a standalone constant:

```python
from seismic_web3 import SANVIL_CHAIN_ID

SANVIL_CHAIN_ID  # 31337
```

## Well-Known Test Accounts

When running Sanvil/Anvil, the following test accounts are pre-funded:

```python
# First test account (10000 ETH)
ACCOUNT_0 = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"

# Second test account (10000 ETH)
ACCOUNT_1 = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"

# And 8 more...
```

These are **unsafe for production** and should only be used for local testing.

## See Also

- [ChainConfig](chain-config.md) - Chain configuration dataclass
- [SEISMIC_TESTNET](seismic-testnet.md) - Public testnet configuration
- [create_wallet_client](../client/create-wallet-client.md) - Direct client creation
- [PrivateKey](../api-reference/types/private-key.md) - Private key type
