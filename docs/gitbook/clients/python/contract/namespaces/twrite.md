---
description: Transparent write namespace for standard contract transactions
icon: eye
---

# .twrite Namespace

The `.twrite` namespace provides standard (non-encrypted) contract write operations using `eth_sendTransaction`. Use this when privacy is not required and you want to minimize gas costs.

***

## Overview

When you call `contract.twrite.functionName(...)`, the SDK:

1. Encodes your function call using the contract ABI
2. Constructs a standard Ethereum transaction
3. Sends the transaction using `eth_sendTransaction`
4. Returns the transaction hash

**No encryption** is applied — calldata is visible on-chain to anyone.

***

## Usage Pattern

```python
tx_hash = contract.twrite.functionName(arg1, arg2, ...)
```

- **Sync**: Returns `HexBytes` transaction hash immediately
- **Async**: Returns `HexBytes` transaction hash (must `await`)

***

## Parameters

### Function Arguments

Pass function arguments as positional parameters:

```python
# Single argument
tx_hash = contract.twrite.setNumber(42)

# Multiple arguments
tx_hash = contract.twrite.transfer(recipient_address, 1000)

# Complex types
tx_hash = contract.twrite.batchTransfer(
    ["0x123...", "0x456..."],
    [100, 200],
)
```

### Transaction Options (Keyword Arguments)

Transaction options are passed as keyword arguments:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `value` | `int` | `0` | ETH value to send (in wei) |
| Any `eth_sendTransaction` param | `Any` | — | Standard web3.py transaction parameters |

Common additional parameters:
- `gas` — Gas limit
- `gasPrice` — Gas price (legacy)
- `maxFeePerGas` — Max fee per gas (EIP-1559)
- `maxPriorityFeePerGas` — Max priority fee (EIP-1559)
- `nonce` — Transaction nonce (auto-generated if omitted)

***

## Examples

### Sync Usage

```python
from seismic_web3 import create_wallet_client

# Create client and contract
w3 = create_wallet_client(...)
contract = w3.seismic.contract(address="0x...", abi=ABI)

# Basic transparent write
tx_hash = contract.twrite.setNumber(42)
print(f"Transaction: {tx_hash.to_0x_hex()}")

# Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
print(f"Status: {receipt['status']}")
```

### Async Usage

```python
from seismic_web3 import create_async_wallet_client

# Create async client and contract
w3 = await create_async_wallet_client(...)
contract = w3.seismic.contract(address="0x...", abi=ABI)

# Basic transparent write
tx_hash = await contract.twrite.setNumber(42)
print(f"Transaction: {tx_hash.to_0x_hex()}")

# Wait for confirmation
receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
print(f"Status: {receipt['status']}")
```

### Sending ETH

```python
# Send 1 ETH with the transaction
tx_hash = contract.twrite.deposit(
    value=10**18,  # 1 ETH in wei
)
```

### Custom Gas (Legacy)

```python
# Legacy gas pricing
tx_hash = contract.twrite.transfer(
    recipient,
    amount,
    gas=100_000,
    gasPrice=20 * 10**9,  # 20 gwei
)
```

### EIP-1559 Gas Parameters

```python
# EIP-1559 gas pricing
tx_hash = contract.twrite.transfer(
    recipient,
    amount,
    gas=100_000,
    maxFeePerGas=50 * 10**9,          # 50 gwei max
    maxPriorityFeePerGas=2 * 10**9,   # 2 gwei priority
)
```

### Custom Nonce

```python
# Explicitly set nonce
nonce = w3.eth.get_transaction_count(w3.eth.default_account)
tx_hash = contract.twrite.transfer(
    recipient,
    amount,
    nonce=nonce,
)
```

### Combining Parameters

```python
tx_hash = contract.twrite.deposit(
    value=10**17,               # 0.1 ETH
    gas=200_000,
    maxFeePerGas=40 * 10**9,    # 40 gwei
    maxPriorityFeePerGas=2 * 10**9,  # 2 gwei
)
```

***

## Return Value

Returns `HexBytes` containing the transaction hash.

```python
# Sync
tx_hash = contract.twrite.transfer(recipient, amount)
assert isinstance(tx_hash, HexBytes)

# Async
tx_hash = await contract.twrite.transfer(recipient, amount)
assert isinstance(tx_hash, HexBytes)
```

You can:
- Convert to hex string: `tx_hash.to_0x_hex()`
- Convert to bytes: `bytes(tx_hash)`
- Wait for receipt: `w3.eth.wait_for_transaction_receipt(tx_hash)`

***

## Privacy Implications

### What's Visible

**Everything is visible on-chain:**
- Your address (transaction sender)
- Contract address (transaction recipient)
- Function selector (first 4 bytes of calldata)
- All function arguments (decoded in calldata)
- Value transferred
- Gas parameters
- Transaction result (success/revert)

### Example

```python
# This transaction is completely visible
tx_hash = contract.twrite.transfer(
    "0x1234...",  # Recipient visible
    1000,         # Amount visible
)
```

Anyone can:
- See you called `transfer`
- See the recipient address
- See the amount transferred
- Decode all parameters from calldata

***

## When to Use .twrite

### Good Use Cases

- **Public operations** — No sensitive data in calldata
- **Cost optimization** — Lower gas costs (no encryption overhead)
- **Public protocols** — DEX trades, public votes, open registrations
- **Testing** — Quick tests where privacy doesn't matter
- **Compatibility** — Interacting with contracts that expect standard transactions

### Examples

```python
# Public token approval (amount is public anyway)
tx_hash = contract.twrite.approve(spender, amount)

# Public state update (no sensitive data)
tx_hash = contract.twrite.setNumber(42)
```

***

## When NOT to Use .twrite

### Use `.write` Instead When

- Function arguments contain sensitive data
- Privacy is required (trading, private voting, auctions)
- Amounts or addresses should be hidden
- Compliance requires confidentiality

### Examples (Use `.write` for these)

```python
# Private balance transfer — use .write
tx_hash = contract.write.transfer(recipient, secret_amount)

# Private withdrawal — use .write
tx_hash = contract.write.withdraw(amount)
```

***

## Comparison with Other Namespaces

| Namespace | Encryption | Transaction Type | Gas Cost | Use Case |
|-----------|-----------|-----------------|----------|----------|
| `.write` | Encrypted calldata | `TxSeismic` (0x4a) | Standard + encryption overhead | Privacy-sensitive writes |
| `.twrite` | No encryption | `eth_sendTransaction` | Standard | Public writes, lower cost |
| `.dwrite` | Encrypted + debug info | `TxSeismic` (0x4a) | Same as `.write` | Development/debugging |

***

## Error Handling

```python
from web3.exceptions import TransactionNotFound, TimeExhausted

try:
    tx_hash = contract.twrite.transfer(recipient, amount)
    receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=120)

    if receipt['status'] == 0:
        print("Transaction reverted")
    else:
        print("Transaction succeeded")

except ValueError as e:
    print(f"Transaction failed: {e}")
except TimeExhausted:
    print("Transaction not mined within timeout")
```

***

## Standard Web3.py Behavior

The `.twrite` namespace uses standard `eth_sendTransaction` under the hood. All web3.py transaction features work:

### Account Management

```python
# Uses default account from web3
w3.eth.default_account = "0x..."
tx_hash = contract.twrite.transfer(recipient, amount)
```

### Transaction Middleware

```python
# All middleware is applied (gas estimation, nonce management, etc.)
from web3.middleware import geth_poa_middleware
w3.middleware_onion.inject(geth_poa_middleware, layer=0)

tx_hash = contract.twrite.transfer(recipient, amount)
```

### Gas Estimation

```python
# Gas is auto-estimated if not provided
tx_hash = contract.twrite.setNumber(42)  # Gas estimated automatically
```

***

## Best Practices

### Security Checklist

- **Verify calldata is not sensitive** — Anyone can see it
- **Check if privacy is required** — Use `.write` if in doubt
- **Validate recipient addresses** — Mistakes are public and permanent
- **Test with small amounts first** — Errors are visible to everyone

### Optimization Tips

- Let web3.py estimate gas (don't hardcode unless necessary)
- Use EIP-1559 pricing on supported networks
- Batch multiple operations if possible
- Consider using `.write` for high-value transactions (extra privacy worth the cost)

### Common Patterns

```python
# Approve + transfer pattern
approve_tx = contract.twrite.approve(spender, amount)
w3.eth.wait_for_transaction_receipt(approve_tx)

transfer_tx = contract.twrite.transferFrom(owner, recipient, amount)
w3.eth.wait_for_transaction_receipt(transfer_tx)
```

***

## Low-Level Alternative

Direct `eth_sendTransaction` call:

```python
tx_hash = w3.eth.send_transaction({
    "to": contract_address,
    "data": "0x...",  # Encoded calldata
    "value": 0,
    "gas": 100_000,
})
```

The `.twrite` namespace is more convenient as it handles ABI encoding automatically.

***

## See Also

- [.write Namespace](write.md) — Encrypted writes for privacy
- [.tread Namespace](tread.md) — Transparent reads
- [.dwrite Namespace](dwrite.md) — Debug writes
- [Contract Instance](../README.md) — Contract wrapper reference
- [Web3.py Documentation](https://web3py.readthedocs.io/) — Standard Ethereum transactions
