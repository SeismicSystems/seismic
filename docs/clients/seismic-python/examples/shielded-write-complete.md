---
description: Full shielded write workflow from setup to confirmation
icon: shield-halved
---

# Shielded Write Complete

This example demonstrates a complete shielded write workflow including contract setup, encrypted transaction submission, receipt verification, and advanced patterns like custom security parameters.

## Prerequisites

```bash
# Install the SDK
pip install seismic-web3

# Set environment variables
export PRIVATE_KEY="your_64_char_hex_private_key"
export CONTRACT_ADDRESS="0x..." # Your deployed contract address
```

## Basic Shielded Write

A shielded write encrypts the transaction calldata so external observers cannot see what function you're calling or what arguments you're passing.

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET
from hexbytes import HexBytes

# Setup
private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

# Contract ABI (simplified example)
ABI = [
    {
        "name": "setNumber",
        "type": "function",
        "inputs": [{"name": "value", "type": "uint256"}],
        "outputs": [],
    },
    {
        "name": "getNumber",
        "type": "function",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
    },
]

# Create contract instance
contract_address = os.environ["CONTRACT_ADDRESS"]
contract = w3.seismic.contract(contract_address, ABI)

# Execute shielded write
print("Sending shielded transaction...")
tx_hash = contract.write.setNumber(42)
print(f"Transaction hash: {tx_hash.hex()}")

# Wait for confirmation
print("Waiting for confirmation...")
receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=60)

# Verify success
if receipt["status"] == 1:
    print(f"Transaction successful!")
    print(f"Block number: {receipt['blockNumber']}")
    print(f"Gas used: {receipt['gasUsed']}")
    print(f"Transaction type: 0x{receipt['type']:02x}")
else:
    print("Transaction failed!")
    raise RuntimeError(f"Transaction reverted: {tx_hash.hex()}")
```

## Complete Workflow with Verification

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET
from hexbytes import HexBytes


def decode_uint256(raw: HexBytes) -> int:
    """Decode a uint256 from ABI-encoded bytes."""
    return int.from_bytes(raw[-32:], "big")


# Setup
private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

ABI = [
    {
        "name": "setNumber",
        "type": "function",
        "inputs": [{"name": "value", "type": "uint256"}],
        "outputs": [],
    },
    {
        "name": "getNumber",
        "type": "function",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
    },
]

contract_address = os.environ["CONTRACT_ADDRESS"]
contract = w3.seismic.contract(contract_address, ABI)

# 1. Read initial value (signed read)
print("Reading initial value...")
initial_value = decode_uint256(contract.read.getNumber())
print(f"Initial value: {initial_value}")

# 2. Write new value (shielded write)
new_value = 12345
print(f"\nWriting new value: {new_value}")
tx_hash = contract.write.setNumber(new_value)
print(f"Transaction hash: {tx_hash.hex()}")

# 3. Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=60)
assert receipt["status"] == 1, "Transaction failed"
print(f"Confirmed in block {receipt['blockNumber']}")

# 4. Verify new value
print("\nVerifying new value...")
updated_value = decode_uint256(contract.read.getNumber())
print(f"Updated value: {updated_value}")

assert updated_value == new_value, f"Expected {new_value}, got {updated_value}"
print("Verification successful!")
```

## Custom Security Parameters

Shielded transactions include freshness checks and expiry windows. You can customize these parameters:

```python
import os
from seismic_web3 import (
    create_wallet_client,
    PrivateKey,
    SEISMIC_TESTNET,
    SeismicSecurityParams,
)

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

ABI = [
    {
        "name": "setNumber",
        "type": "function",
        "inputs": [{"name": "value", "type": "uint256"}],
        "outputs": [],
    }
]

contract_address = os.environ["CONTRACT_ADDRESS"]
contract = w3.seismic.contract(contract_address, ABI)

# Custom security parameters
params = SeismicSecurityParams(
    blocks_window=50,  # Expires after 50 blocks instead of default 100
    encryption_nonce=None,  # Random (default)
    recent_block_hash=None,  # Use latest block (default)
    expires_at_block=None,  # Computed from blocks_window (default)
)

# Write with custom security parameters
tx_hash = contract.write.setNumber(42, security=params)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=60)
print(f"Transaction confirmed: {receipt['status'] == 1}")
```

## Low-Level Shielded Write

For advanced use cases like contract deployment or pre-encoded calldata:

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET
from hexbytes import HexBytes

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

# Pre-encoded calldata (e.g., from web3.py contract.encodeABI)
calldata = HexBytes("0x3fb5c1cb000000000000000000000000000000000000000000000000000000000000002a")

# Send shielded transaction with raw calldata
tx_hash = w3.seismic.send_shielded_transaction(
    to=os.environ["CONTRACT_ADDRESS"],
    data=calldata,
    value=0,  # No ETH transfer
    gas=100_000,  # Gas limit
    gas_price=10**9,  # 1 gwei
)

receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=60)
print(f"Transaction status: {receipt['status']}")
```

## Debug Mode

Debug mode returns additional information including plaintext and encrypted data:

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

ABI = [
    {
        "name": "setNumber",
        "type": "function",
        "inputs": [{"name": "value", "type": "uint256"}],
        "outputs": [],
    }
]

contract_address = os.environ["CONTRACT_ADDRESS"]
contract = w3.seismic.contract(contract_address, ABI)

# Execute debug write
result = contract.dwrite.setNumber(42)

# Access debug information
print(f"Transaction hash: {result.tx_hash.hex()}")
print(f"Plaintext calldata: {result.plaintext_tx.data.hex()}")
print(f"Encrypted calldata: {result.shielded_tx.seismic_elements.encrypted_calldata.hex()}")
print(f"Encryption pubkey: {result.shielded_tx.seismic_elements.encryption_pubkey.to_0x_hex()}")
print(f"Encryption nonce: {result.shielded_tx.seismic_elements.encryption_nonce.to_0x_hex()}")

# Transaction still broadcasts normally
receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash, timeout=60)
print(f"Confirmed: {receipt['status'] == 1}")
```

## Error Handling

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET
from web3.exceptions import ContractLogicError, TimeExhausted, TransactionNotFound


def safe_shielded_write(contract, method_name: str, *args, timeout: int = 60):
    """Execute shielded write with comprehensive error handling."""
    try:
        # Get the write method
        method = getattr(contract.write, method_name)

        # Send transaction
        print(f"Calling {method_name}({', '.join(map(str, args))})")
        tx_hash = method(*args)
        print(f"Transaction sent: {tx_hash.hex()}")

        # Wait for receipt
        receipt = contract._w3.eth.wait_for_transaction_receipt(tx_hash, timeout=timeout)

        # Check status
        if receipt["status"] == 0:
            raise ContractLogicError("Transaction reverted")

        print(f"Success! Block: {receipt['blockNumber']}, Gas: {receipt['gasUsed']}")
        return receipt

    except ContractLogicError as e:
        print(f"Contract error: {e}")
        raise
    except TimeExhausted:
        print(f"Transaction not mined within {timeout} seconds")
        raise
    except TransactionNotFound:
        print("Transaction not found (possibly dropped)")
        raise
    except Exception as e:
        print(f"Unexpected error: {e}")
        raise


# Usage
private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

ABI = [
    {
        "name": "setNumber",
        "type": "function",
        "inputs": [{"name": "value", "type": "uint256"}],
        "outputs": [],
    }
]

contract = w3.seismic.contract(os.environ["CONTRACT_ADDRESS"], ABI)
receipt = safe_shielded_write(contract, "setNumber", 42)
```

## Batch Operations

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

ABI = [
    {
        "name": "setNumber",
        "type": "function",
        "inputs": [{"name": "value", "type": "uint256"}],
        "outputs": [],
    }
]

contract = w3.seismic.contract(os.environ["CONTRACT_ADDRESS"], ABI)

# Send multiple transactions
values = [10, 20, 30, 40, 50]
tx_hashes = []

print("Sending batch transactions...")
for value in values:
    tx_hash = contract.write.setNumber(value)
    tx_hashes.append(tx_hash)
    print(f"Sent: value={value}, tx={tx_hash.hex()}")

# Wait for all confirmations
print("\nWaiting for confirmations...")
receipts = []
for i, tx_hash in enumerate(tx_hashes):
    receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=60)
    receipts.append(receipt)
    print(f"Confirmed {i+1}/{len(tx_hashes)}: block={receipt['blockNumber']}, status={receipt['status']}")

# Verify all succeeded
assert all(r["status"] == 1 for r in receipts), "Some transactions failed"
print("\nAll transactions successful!")
```

## Expected Output

```
Sending shielded transaction...
Transaction hash: 0xabc123...
Waiting for confirmation...
Transaction successful!
Block number: 12346
Gas used: 43521
Transaction type: 0x4a
```

## Common Variations

### With Value Transfer

```python
# Send ETH along with shielded call
tx_hash = w3.seismic.send_shielded_transaction(
    to=contract_address,
    data=calldata,
    value=w3.to_wei(0.1, "ether"),  # Send 0.1 ETH
    gas=100_000,
    gas_price=10**9,
)
```

### Gas Estimation

```python
# Estimate gas before sending (uses transparent call)
estimated_gas = w3.eth.estimate_gas({
    "to": contract_address,
    "data": calldata,
})
print(f"Estimated gas: {estimated_gas}")

# Add 20% buffer and send
tx_hash = w3.seismic.send_shielded_transaction(
    to=contract_address,
    data=calldata,
    gas=int(estimated_gas * 1.2),
    gas_price=10**9,
)
```

## Next Steps

* [Signed Read Pattern](signed-read-pattern.md) - Read encrypted data
* [SRC20 Workflow](src20-workflow.md) - Private token operations
* [Async Patterns](async-patterns.md) - Concurrent transaction handling
* [Basic Wallet Setup](basic-wallet-setup.md) - Client configuration

## See Also

* [Shielded Write Guide](../guides/shielded-write.md) - Detailed guide
* [ShieldedContract](../contract/shielded-contract.md) - API reference
* [SeismicSecurityParams](../api-reference/signature/seismic-security-params.md) - Security parameters
* [send\_shielded\_transaction](../namespaces/seismic-namespace.md) - Low-level API
