---
description: Complete signed read pattern with result decoding
icon: signature
---

# Signed Read Pattern

This example demonstrates signed reads (`.read`), which encrypt both the calldata you send and the result you receive. Signed reads prove your identity to the contract via `msg.sender`, enabling access-controlled data retrieval.

## Prerequisites

```bash
# Install the SDK
pip install seismic-web3

# Set environment variables
export PRIVATE_KEY="your_64_char_hex_private_key"
export CONTRACT_ADDRESS="0x..." # Your deployed contract address
```

## Why Use Signed Reads

Signed reads are necessary when:

1. The contract function depends on `msg.sender` (e.g., `balanceOf()` with no arguments)
2. You need to prove your identity for access control
3. You want to hide what data you're querying from external observers

A transparent read (`.tread`) sets `msg.sender` to the zero address, which usually returns incorrect data.

## Basic Signed Read

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

# Contract ABI
ABI = [
    {
        "name": "balanceOf",
        "type": "function",
        "inputs": [],  # No arguments - uses msg.sender
        "outputs": [{"name": "", "type": "uint256"}],
    }
]

contract_address = os.environ["CONTRACT_ADDRESS"]
contract = w3.seismic.contract(contract_address, ABI)

# Execute signed read (proves identity)
print("Executing signed read...")
raw_result = contract.read.balanceOf()
balance = decode_uint256(raw_result)
print(f"Your balance: {balance}")
```

## Complete Read Workflow with Type Decoding

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET
from hexbytes import HexBytes


class ABIDecoder:
    """Helper class for decoding common ABI types."""

    @staticmethod
    def decode_uint256(raw: HexBytes) -> int:
        """Decode uint256."""
        return int.from_bytes(raw[-32:], "big")

    @staticmethod
    def decode_bool(raw: HexBytes) -> bool:
        """Decode bool."""
        return int.from_bytes(raw[-32:], "big") != 0

    @staticmethod
    def decode_address(raw: HexBytes) -> str:
        """Decode address."""
        return "0x" + raw[-20:].hex()

    @staticmethod
    def decode_bytes32(raw: HexBytes) -> HexBytes:
        """Decode bytes32."""
        return HexBytes(raw[-32:])

    @staticmethod
    def decode_string(raw: HexBytes) -> str:
        """Decode string (dynamic type)."""
        # Skip offset (32 bytes) and length (32 bytes), then read string
        length = int.from_bytes(raw[32:64], "big")
        return raw[64 : 64 + length].decode("utf-8")


# Setup
private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

# Example contract with multiple read functions
ABI = [
    {
        "name": "balanceOf",
        "type": "function",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
    },
    {
        "name": "isActive",
        "type": "function",
        "inputs": [],
        "outputs": [{"name": "", "type": "bool"}],
    },
    {
        "name": "owner",
        "type": "function",
        "inputs": [],
        "outputs": [{"name": "", "type": "address"}],
    },
]

contract = w3.seismic.contract(os.environ["CONTRACT_ADDRESS"], ABI)
decoder = ABIDecoder()

# Execute multiple signed reads
print("Querying contract state...")

balance_raw = contract.read.balanceOf()
balance = decoder.decode_uint256(balance_raw)
print(f"Balance: {balance}")

active_raw = contract.read.isActive()
is_active = decoder.decode_bool(active_raw)
print(f"Active: {is_active}")

owner_raw = contract.read.owner()
owner = decoder.decode_address(owner_raw)
print(f"Owner: {owner}")
```

## Signed Read vs Transparent Read

This example shows the difference between signed and transparent reads:

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET
from hexbytes import HexBytes


def decode_uint256(raw: HexBytes) -> int:
    return int.from_bytes(raw[-32:], "big")


private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

ABI = [
    {
        "name": "balanceOf",
        "type": "function",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
    }
]

contract = w3.seismic.contract(os.environ["CONTRACT_ADDRESS"], ABI)

# Signed read - proves identity (correct)
print("Signed read (.read):")
signed_result = contract.read.balanceOf()
signed_balance = decode_uint256(signed_result)
print(f"  Your balance: {signed_balance}")

# Transparent read - msg.sender is 0x0 (incorrect)
print("\nTransparent read (.tread):")
transparent_result = contract.tread.balanceOf()
transparent_balance = decode_uint256(transparent_result)
print(f"  Zero address balance: {transparent_balance}")

# They should differ if you have a non-zero balance
assert signed_balance != transparent_balance or signed_balance == 0, \
    "Expected different results between signed and transparent reads"
```

## Custom Security Parameters

Like shielded writes, signed reads support custom security parameters:

```python
import os
from seismic_web3 import (
    create_wallet_client,
    PrivateKey,
    SEISMIC_TESTNET,
    SeismicSecurityParams,
)
from hexbytes import HexBytes


def decode_uint256(raw: HexBytes) -> int:
    return int.from_bytes(raw[-32:], "big")


private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

ABI = [
    {
        "name": "balanceOf",
        "type": "function",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
    }
]

contract = w3.seismic.contract(os.environ["CONTRACT_ADDRESS"], ABI)

# Custom security parameters
params = SeismicSecurityParams(
    blocks_window=50,  # Shorter expiry window
    encryption_nonce=None,  # Random
    recent_block_hash=None,  # Latest block
    expires_at_block=None,  # Computed
)

# Signed read with custom parameters
result = contract.read.balanceOf(security=params)
balance = decode_uint256(result)
print(f"Balance: {balance}")
```

## Low-Level Signed Call

For pre-encoded calldata or advanced use cases:

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET
from hexbytes import HexBytes

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

# Pre-encoded calldata (balanceOf())
calldata = HexBytes("0x70a08231")  # Function selector

# Execute signed call
result = w3.seismic.signed_call(
    to=os.environ["CONTRACT_ADDRESS"],
    data=calldata,
    gas=30_000_000,  # Gas limit for the call
)

print(f"Result: {result.hex()}")
balance = int.from_bytes(result[-32:], "big")
print(f"Balance: {balance}")
```

## Error Handling

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET
from hexbytes import HexBytes
from web3.exceptions import ContractLogicError


def safe_signed_read(contract, method_name: str, *args):
    """Execute signed read with error handling."""
    try:
        # Get the read method
        method = getattr(contract.read, method_name)

        # Execute call
        print(f"Calling {method_name}({', '.join(map(str, args))})")
        result = method(*args)

        if result is None or len(result) == 0:
            raise ValueError("Empty result returned")

        return result

    except ContractLogicError as e:
        print(f"Contract error: {e}")
        raise
    except AttributeError:
        print(f"Method '{method_name}' not found in contract")
        raise
    except Exception as e:
        print(f"Unexpected error: {e}")
        raise


# Usage
private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

ABI = [
    {
        "name": "balanceOf",
        "type": "function",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
    }
]

contract = w3.seismic.contract(os.environ["CONTRACT_ADDRESS"], ABI)
result = safe_signed_read(contract, "balanceOf")
balance = int.from_bytes(result[-32:], "big")
print(f"Balance: {balance}")
```

## Async Signed Reads

```python
import asyncio
import os
from seismic_web3 import create_async_wallet_client, PrivateKey, SEISMIC_TESTNET
from hexbytes import HexBytes


def decode_uint256(raw: HexBytes) -> int:
    return int.from_bytes(raw[-32:], "big")


async def main():
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
    w3 = await SEISMIC_TESTNET.async_wallet_client(private_key)

    try:
        ABI = [
            {
                "name": "balanceOf",
                "type": "function",
                "inputs": [],
                "outputs": [{"name": "", "type": "uint256"}],
            },
            {
                "name": "isActive",
                "type": "function",
                "inputs": [],
                "outputs": [{"name": "", "type": "bool"}],
            },
        ]

        contract = w3.seismic.contract(os.environ["CONTRACT_ADDRESS"], ABI)

        # Execute multiple reads concurrently
        print("Executing concurrent signed reads...")
        balance_task = contract.read.balanceOf()
        active_task = contract.read.isActive()

        # Await both
        balance_raw, active_raw = await asyncio.gather(balance_task, active_task)

        balance = decode_uint256(balance_raw)
        is_active = int.from_bytes(active_raw[-32:], "big") != 0

        print(f"Balance: {balance}")
        print(f"Active: {is_active}")

    finally:
        await w3.provider.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
```

## Batch Reads

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET
from hexbytes import HexBytes


def decode_uint256(raw: HexBytes) -> int:
    return int.from_bytes(raw[-32:], "big")


private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

ABI = [
    {
        "name": "getDataPoint",
        "type": "function",
        "inputs": [{"name": "index", "type": "uint256"}],
        "outputs": [{"name": "", "type": "uint256"}],
    }
]

contract = w3.seismic.contract(os.environ["CONTRACT_ADDRESS"], ABI)

# Read multiple data points
indices = range(5)
print("Reading multiple data points...")

results = []
for i in indices:
    result = contract.read.getDataPoint(i)
    value = decode_uint256(result)
    results.append(value)
    print(f"  Data point {i}: {value}")

print(f"\nTotal results: {len(results)}")
```

## Expected Output

```
Executing signed read...
Your balance: 1000

Querying contract state...
Balance: 1000
Active: True
Owner: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
```

## Common Variations

### Reading Complex Types

```python
from hexbytes import HexBytes


def decode_tuple(raw: HexBytes) -> tuple:
    """Decode a tuple (struct) with multiple fields."""
    # Example: (uint256, bool, address)
    value = int.from_bytes(raw[0:32], "big")
    flag = int.from_bytes(raw[32:64], "big") != 0
    addr = "0x" + raw[76:96].hex()
    return (value, flag, addr)


result = contract.read.getStruct()
value, flag, addr = decode_tuple(result)
print(f"Struct: value={value}, flag={flag}, address={addr}")
```

### Handling View Functions with Arguments

```python
# View function with argument that uses msg.sender internally
result = contract.read.getAllowance("0xSpender...")
allowance = decode_uint256(result)
print(f"Allowance: {allowance}")
```

## Next Steps

- [Shielded Write Complete](shielded-write-complete.md) - Send encrypted transactions
- [SRC20 Workflow](src20-workflow.md) - Private token operations
- [Async Patterns](async-patterns.md) - Concurrent operations
- [Basic Wallet Setup](basic-wallet-setup.md) - Client configuration

## See Also

- [Signed Reads Guide](../guides/signed-reads.md) - Detailed guide
- [ShieldedContract](../contract/shielded-contract.md) - API reference
- [signed_call](../namespaces/seismic-namespace.md) - Low-level API
- [SeismicSecurityParams](../api-reference/transaction-types/seismic-security-params.md) - Security parameters
