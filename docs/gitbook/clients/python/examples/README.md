---
description: Complete runnable examples
icon: code
---

# Examples

Complete, runnable code examples demonstrating common Seismic Python SDK patterns.

## Available Examples

### Basic Setup
- **Wallet Setup** - Setting up sync and async wallet clients
- **Public Client** - Setting up read-only public clients
- **Chain Configuration** - Using built-in chain configs

### Transaction Patterns
- **Shielded Write** - Complete shielded transaction workflow
- **Signed Read** - Executing signed reads with encryption
- **Debug Mode** - Using debug writes for development

### Token Operations
- **SRC20 Workflow** - Complete SRC20 token interaction
- **Event Watching** - Monitoring SRC20 events
- **Viewing Keys** - Managing viewing keys with directory

### Advanced Patterns
- **Async Patterns** - Async client best practices
- **Error Handling** - Comprehensive error handling
- **Batch Operations** - Efficiently batching multiple operations

## Example Template

Each example follows this structure:

```python
# 1. Imports and setup
from seismic_web3 import create_wallet_client, PrivateKey

# 2. Configuration
private_key = PrivateKey(...)
w3 = create_wallet_client("https://rpc.example.com", private_key=private_key)

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
export RPC_URL="https://rpc.example.com"
```

## Quick Example: Basic Wallet Setup

```python
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET
import os

# Setup
private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))

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
