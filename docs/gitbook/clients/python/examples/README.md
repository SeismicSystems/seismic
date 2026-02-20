---
description: Complete runnable examples
icon: code
---

# Examples

Complete, runnable code examples demonstrating common Seismic Python SDK patterns.

## Available Examples

### Getting Started

| Example | Description |
|---------|-------------|
| [Basic Wallet Setup](basic-wallet-setup.md) | Complete wallet client setup with both sync and async variants, environment configuration, and connection verification |

### Core Workflows

| Example | Description |
|---------|-------------|
| [Shielded Write Complete](shielded-write-complete.md) | Full shielded write workflow from setup to confirmation, including custom security parameters and error handling |
| [Signed Read Pattern](signed-read-pattern.md) | Complete signed read pattern with result decoding, demonstrating identity-proving reads and type conversion |
| [SRC20 Workflow](src20-workflow.md) | Full SRC20 token workflow including metadata, balances, transfers, approvals, minting, and event watching |

### Advanced Patterns

| Example | Description |
|---------|-------------|
| [Async Patterns](async-patterns.md) | Best practices for async usage including concurrent operations, error handling, connection pooling, rate limiting, and WebSocket events |

## Example Template

Each example follows this structure:

```python
# 1. Imports and setup
from seismic_web3 import create_wallet_client, PrivateKey

# 2. Configuration
private_key = PrivateKey(...)
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# 3. Main operation
result = await contract.write.method(args)

# 4. Confirmation and verification
receipt = w3.eth.wait_for_transaction_receipt(result)
print(f"Status: {receipt['status']}")
```

## Running Examples

All examples assume you have:

```bash
# Install the SDK
pip install seismic-web3

# Set environment variables
export PRIVATE_KEY="0x..."
export RPC_URL="https://gcp-1.seismictest.net/rpc"
```

## Quick Example: Basic Wallet Setup

```python
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET
import os

# Setup
private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

# Method 1: Using chain config (recommended)
w3 = SEISMIC_TESTNET.wallet_client(private_key)

# Method 2: Using RPC URL directly
w3 = create_wallet_client(
    os.environ["RPC_URL"],
    private_key=private_key,
)

# Verify connection
print(f"Connected to chain ID: {w3.eth.chain_id}")
print(f"Block number: {w3.eth.block_number}")
print(f"TEE public key: {w3.seismic.get_tee_public_key().to_0x_hex()}")
```

## See Also

- [Guides](../guides/) - Step-by-step tutorials
- [API Reference](../api-reference/) - Complete API documentation
- [Client Documentation](../client/) - Client setup
- [Contract Documentation](../contract/) - Contract patterns
