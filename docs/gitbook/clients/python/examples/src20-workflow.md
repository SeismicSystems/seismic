---
description: Full SRC20 workflow - register viewing key, transfer, watch events
icon: coins
---

# SRC20 Workflow

This example demonstrates a complete SRC20 token workflow including token metadata retrieval, balance queries, transfers, approvals, and event monitoring. SRC20 is Seismic's privacy-preserving token standard with shielded balances and amounts.

## Prerequisites

```bash
# Install the SDK
pip install seismic-web3

# Set environment variables
export PRIVATE_KEY="your_64_char_hex_private_key"
export TOKEN_ADDRESS="0x..." # SRC20 token contract address
```

## Basic SRC20 Token Interaction

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET, SRC20_ABI
from hexbytes import HexBytes


def decode_uint256(raw: HexBytes) -> int:
    """Decode a uint256 from ABI-encoded bytes."""
    return int.from_bytes(raw[-32:], "big")


# Setup
private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

# Create SRC20 token instance (SDK provides SRC20_ABI)
token_address = os.environ["TOKEN_ADDRESS"]
token = w3.seismic.contract(token_address, SRC20_ABI)

# Get token metadata (transparent reads - metadata is public)
print("Token Metadata:")
name = token.tread.name()
symbol = token.tread.symbol()
decimals_raw = token.tread.decimals()

print(f"  Name: {name.decode('utf-8') if isinstance(name, bytes) else name}")
print(f"  Symbol: {symbol.decode('utf-8') if isinstance(symbol, bytes) else symbol}")
print(f"  Decimals: {decode_uint256(decimals_raw)}")

# Get your balance (signed read - proves identity)
print("\nChecking balance...")
balance_raw = token.read.balanceOf()
balance = decode_uint256(balance_raw)
decimals = decode_uint256(decimals_raw)
print(f"  Your balance: {balance / (10 ** decimals)} tokens")
print(f"  Raw balance: {balance}")
```

## Complete Transfer Workflow

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET, SRC20_ABI
from hexbytes import HexBytes


def decode_uint256(raw: HexBytes) -> int:
    return int.from_bytes(raw[-32:], "big")


# Setup
private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

token = w3.seismic.contract(os.environ["TOKEN_ADDRESS"], SRC20_ABI)

recipient = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
amount = 100  # Raw amount (not adjusted for decimals)

# 1. Check initial balance
print("Checking initial balances...")
initial_balance_raw = token.read.balanceOf()
initial_balance = decode_uint256(initial_balance_raw)
print(f"Your balance: {initial_balance}")

# Verify sufficient balance
if initial_balance < amount:
    raise ValueError(f"Insufficient balance: {initial_balance} < {amount}")

# 2. Execute transfer (shielded write)
print(f"\nTransferring {amount} tokens to {recipient}...")
tx_hash = token.write.transfer(recipient, amount)
print(f"Transaction hash: {tx_hash.hex()}")

# 3. Wait for confirmation
print("Waiting for confirmation...")
receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=60)

if receipt["status"] == 1:
    print(f"Transfer confirmed in block {receipt['blockNumber']}")
else:
    raise RuntimeError("Transfer failed")

# 4. Verify new balance
print("\nVerifying balance after transfer...")
final_balance_raw = token.read.balanceOf()
final_balance = decode_uint256(final_balance_raw)
print(f"Your new balance: {final_balance}")

expected_balance = initial_balance - amount
assert final_balance == expected_balance, \
    f"Balance mismatch: expected {expected_balance}, got {final_balance}"
print("Balance verification successful!")
```

## Approval and TransferFrom Pattern

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET, SRC20_ABI
from hexbytes import HexBytes


def decode_uint256(raw: HexBytes) -> int:
    return int.from_bytes(raw[-32:], "big")


private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

token = w3.seismic.contract(os.environ["TOKEN_ADDRESS"], SRC20_ABI)

owner = w3.eth.default_account
spender = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
amount = 500

print(f"Approval workflow:")
print(f"  Owner: {owner}")
print(f"  Spender: {spender}")
print(f"  Amount: {amount}")

# 1. Check current allowance (transparent read)
print("\nChecking current allowance...")
allowance_raw = token.tread.allowance(owner, spender)
current_allowance = decode_uint256(allowance_raw)
print(f"Current allowance: {current_allowance}")

# 2. Approve spender (shielded write)
print(f"\nApproving spender for {amount} tokens...")
tx_hash = token.write.approve(spender, amount)
print(f"Transaction hash: {tx_hash.hex()}")

receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=60)
assert receipt["status"] == 1, "Approval failed"
print(f"Approval confirmed in block {receipt['blockNumber']}")

# 3. Verify new allowance
print("\nVerifying new allowance...")
new_allowance_raw = token.tread.allowance(owner, spender)
new_allowance = decode_uint256(new_allowance_raw)
print(f"New allowance: {new_allowance}")

assert new_allowance == amount, f"Allowance mismatch: expected {amount}, got {new_allowance}"
print("Approval successful!")

# Note: The spender would now call token.write.transferFrom(owner, recipient, amount)
```

## Minting Tokens (Admin Operation)

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET, SRC20_ABI
from hexbytes import HexBytes


def decode_uint256(raw: HexBytes) -> int:
    return int.from_bytes(raw[-32:], "big")


# Setup (requires admin/owner private key)
private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

token = w3.seismic.contract(os.environ["TOKEN_ADDRESS"], SRC20_ABI)

recipient = w3.eth.default_account
mint_amount = 1000

print(f"Minting {mint_amount} tokens to {recipient}")

# 1. Check balance before mint
balance_before_raw = token.read.balanceOf()
balance_before = decode_uint256(balance_before_raw)
print(f"Balance before: {balance_before}")

# 2. Mint tokens (shielded write - admin only)
tx_hash = token.write.mint(recipient, mint_amount)
print(f"Mint transaction: {tx_hash.hex()}")

receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=60)
assert receipt["status"] == 1, "Mint failed"
print(f"Mint confirmed in block {receipt['blockNumber']}")

# 3. Verify new balance
balance_after_raw = token.read.balanceOf()
balance_after = decode_uint256(balance_after_raw)
print(f"Balance after: {balance_after}")

expected_balance = balance_before + mint_amount
assert balance_after == expected_balance, \
    f"Balance mismatch: expected {expected_balance}, got {balance_after}"
print(f"Successfully minted {mint_amount} tokens!")
```

## Watching Transfer Events

```python
import os
import time
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET, SRC20_ABI


private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

token_address = os.environ["TOKEN_ADDRESS"]

# Get the standard web3 contract for event filtering
# (Seismic events work with standard web3.py event filters)
standard_contract = w3.eth.contract(address=token_address, abi=SRC20_ABI)

# Get current block
current_block = w3.eth.block_number
print(f"Starting event watch from block {current_block}")

# Create event filter for Transfer events
event_filter = standard_contract.events.Transfer.create_filter(fromBlock=current_block)

print("Watching for Transfer events... (Ctrl+C to stop)")

try:
    while True:
        # Poll for new events
        for event in event_filter.get_new_entries():
            print(f"\nTransfer Event:")
            print(f"  Block: {event['blockNumber']}")
            print(f"  Transaction: {event['transactionHash'].hex()}")
            print(f"  From: {event['args']['from']}")
            print(f"  To: {event['args']['to']}")
            print(f"  Amount: {event['args']['value']}")

        time.sleep(2)  # Poll every 2 seconds

except KeyboardInterrupt:
    print("\nStopped watching events")
```

## Async SRC20 Workflow

```python
import asyncio
import os
from seismic_web3 import create_async_wallet_client, PrivateKey, SEISMIC_TESTNET, SRC20_ABI
from hexbytes import HexBytes


def decode_uint256(raw: HexBytes) -> int:
    return int.from_bytes(raw[-32:], "big")


async def main():
    private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
    w3 = await SEISMIC_TESTNET.async_wallet_client(private_key)

    try:
        token = w3.seismic.contract(os.environ["TOKEN_ADDRESS"], SRC20_ABI)

        # Execute multiple operations concurrently
        print("Fetching token info concurrently...")

        name_task = token.tread.name()
        symbol_task = token.tread.symbol()
        decimals_task = token.tread.decimals()
        balance_task = token.read.balanceOf()

        # Await all
        name, symbol, decimals_raw, balance_raw = await asyncio.gather(
            name_task,
            symbol_task,
            decimals_task,
            balance_task,
        )

        decimals = decode_uint256(decimals_raw)
        balance = decode_uint256(balance_raw)

        print(f"\nToken Info:")
        print(f"  Name: {name.decode('utf-8') if isinstance(name, bytes) else name}")
        print(f"  Symbol: {symbol.decode('utf-8') if isinstance(symbol, bytes) else symbol}")
        print(f"  Decimals: {decimals}")
        print(f"  Your balance: {balance / (10 ** decimals)} tokens")

        # Execute transfer
        recipient = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
        amount = 50

        print(f"\nTransferring {amount} tokens to {recipient}...")
        tx_hash = await token.write.transfer(recipient, amount)
        print(f"Transaction: {tx_hash.hex()}")

        receipt = await w3.eth.wait_for_transaction_receipt(tx_hash, timeout=60)
        print(f"Confirmed in block {receipt['blockNumber']}")

    finally:
        await w3.provider.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
```

## Batch Transfers

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET, SRC20_ABI
from hexbytes import HexBytes


def decode_uint256(raw: HexBytes) -> int:
    return int.from_bytes(raw[-32:], "big")


private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

token = w3.seismic.contract(os.environ["TOKEN_ADDRESS"], SRC20_ABI)

# Recipients and amounts
transfers = [
    ("0x70997970C51812dc3A010C7d01b50e0d17dc79C8", 100),
    ("0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", 200),
    ("0x90F79bf6EB2c4f870365E785982E1f101E93b906", 150),
]

total_amount = sum(amount for _, amount in transfers)

# Verify sufficient balance
balance_raw = token.read.balanceOf()
balance = decode_uint256(balance_raw)
print(f"Current balance: {balance}")
print(f"Total to transfer: {total_amount}")

if balance < total_amount:
    raise ValueError(f"Insufficient balance: {balance} < {total_amount}")

# Execute transfers
print(f"\nSending {len(transfers)} transfers...")
tx_hashes = []

for recipient, amount in transfers:
    tx_hash = token.write.transfer(recipient, amount)
    tx_hashes.append((recipient, amount, tx_hash))
    print(f"  Sent {amount} to {recipient}: {tx_hash.hex()}")

# Wait for all confirmations
print("\nWaiting for confirmations...")
for recipient, amount, tx_hash in tx_hashes:
    receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=60)
    print(f"  Confirmed: {amount} to {recipient} (block {receipt['blockNumber']})")

# Verify final balance
final_balance_raw = token.read.balanceOf()
final_balance = decode_uint256(final_balance_raw)
print(f"\nFinal balance: {final_balance}")

expected_balance = balance - total_amount
assert final_balance == expected_balance, "Balance verification failed"
print("All transfers successful!")
```

## Error Handling

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET, SRC20_ABI
from hexbytes import HexBytes
from web3.exceptions import ContractLogicError


def safe_transfer(token, recipient: str, amount: int, timeout: int = 60):
    """Execute SRC20 transfer with comprehensive error handling."""
    try:
        # Decode helper
        def decode_uint256(raw):
            return int.from_bytes(raw[-32:], "big")

        # Check balance
        balance_raw = token.read.balanceOf()
        balance = decode_uint256(balance_raw)

        if balance < amount:
            raise ValueError(f"Insufficient balance: {balance} < {amount}")

        # Execute transfer
        print(f"Transferring {amount} to {recipient}...")
        tx_hash = token.write.transfer(recipient, amount)
        print(f"Transaction: {tx_hash.hex()}")

        # Wait for receipt
        receipt = token._w3.eth.wait_for_transaction_receipt(tx_hash, timeout=timeout)

        if receipt["status"] == 0:
            raise ContractLogicError("Transfer transaction reverted")

        print(f"Success! Block: {receipt['blockNumber']}")
        return receipt

    except ValueError as e:
        print(f"Validation error: {e}")
        raise
    except ContractLogicError as e:
        print(f"Contract error: {e}")
        raise
    except Exception as e:
        print(f"Unexpected error: {e}")
        raise


# Usage
private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))
w3 = SEISMIC_TESTNET.wallet_client(private_key)

token = w3.seismic.contract(os.environ["TOKEN_ADDRESS"], SRC20_ABI)
receipt = safe_transfer(token, "0x70997970C51812dc3A010C7d01b50e0d17dc79C8", 100)
```

## Expected Output

```
Token Metadata:
  Name: TestToken
  Symbol: TEST
  Decimals: 18

Checking balance...
  Your balance: 10.0 tokens
  Raw balance: 10000000000000000000

Transferring 100 tokens to 0x70997970C51812dc3A010C7d01b50e0d17dc79C8...
Transaction hash: 0xabc123...
Waiting for confirmation...
Transfer confirmed in block 12347

Verifying balance after transfer...
Your new balance: 9999999999999999900
Balance verification successful!
```

## Common Variations

### With Decimal Conversion

```python
def to_token_units(amount: float, decimals: int) -> int:
    """Convert human-readable amount to token units."""
    return int(amount * (10 ** decimals))


def from_token_units(amount: int, decimals: int) -> float:
    """Convert token units to human-readable amount."""
    return amount / (10 ** decimals)


# Transfer 1.5 tokens (with 18 decimals)
amount = to_token_units(1.5, 18)
tx_hash = token.write.transfer(recipient, amount)
```

### Check Multiple Allowances

```python
spenders = [
    "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
    "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
]

owner = w3.eth.default_account

print("Checking allowances:")
for spender in spenders:
    allowance_raw = token.tread.allowance(owner, spender)
    allowance = decode_uint256(allowance_raw)
    print(f"  {spender}: {allowance}")
```

## Next Steps

- [Basic Wallet Setup](basic-wallet-setup.md) - Client configuration
- [Shielded Write Complete](shielded-write-complete.md) - Transaction patterns
- [Signed Read Pattern](signed-read-pattern.md) - Read operations
- [Async Patterns](async-patterns.md) - Concurrent operations

## See Also

- [SRC20 Documentation](../src20/) - SRC20 standard details
- [SRC20_ABI](../abis/) - Built-in ABI
- [ShieldedContract](../contract/shielded-contract.md) - Contract API
- [Event Filtering](https://web3py.readthedocs.io/en/stable/filters.html) - web3.py events
