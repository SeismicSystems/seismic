---
description: Shielded write namespace for encrypted contract transactions
icon: shield-halved
---

# .write Namespace

The `.write` namespace provides encrypted contract write operations using Seismic's `TxSeismic` (type `0x4a`) transaction format. Calldata is encrypted end-to-end from your client to the node's TEE, ensuring on-chain privacy.

***

## Overview

When you call `contract.write.functionName(...)`, the SDK:

1. Encodes your function call using the contract ABI
2. Encrypts the calldata using AES-GCM with a shared key derived via ECDH
3. Constructs a `TxSeismic` with encryption metadata (nonce, block hash, expiry)
4. Signs and broadcasts the transaction
5. Returns the transaction hash

The encrypted calldata is bound to the transaction context (chain ID, nonce, block hash, expiry) via AES-GCM additional authenticated data, preventing replay attacks and tampering.

***

## Usage Pattern

```python
tx_hash = contract.write.functionName(arg1, arg2, ...)
```

- **Sync**: Returns `HexBytes` transaction hash immediately
- **Async**: Returns `HexBytes` transaction hash (must `await`)

***

## Parameters

### Function Arguments

Pass function arguments as positional parameters:

```python
# Single argument
tx_hash = contract.write.setNumber(42)

# Multiple arguments
tx_hash = contract.write.transfer(recipient_address, 1000)

# Complex types
tx_hash = contract.write.batchTransfer(
    ["0x123...", "0x456..."],
    [100, 200],
)
```

### Transaction Options (Keyword Arguments)

All transaction options are **optional** keyword arguments:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `value` | `int` | `0` | ETH value to send (in wei) |
| `gas` | `int \| None` | `None` | Gas limit (auto-estimated if `None`) |
| `gas_price` | `int \| None` | `None` | Gas price in wei (uses network default if `None`) |
| `security` | [`SeismicSecurityParams`](../../api-reference/transaction-types/seismic-security-params.md) \| `None` | `None` | Custom security parameters (block expiry, nonce, etc.) |

***

## Examples

### Sync Usage

```python
from seismic_web3 import create_wallet_client

# Create client and contract
w3 = create_wallet_client(...)
contract = w3.seismic.contract(address="0x...", abi=ABI)

# Basic write
tx_hash = contract.write.setNumber(42)
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

# Basic write
tx_hash = await contract.write.setNumber(42)
print(f"Transaction: {tx_hash.to_0x_hex()}")

# Wait for confirmation
receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
print(f"Status: {receipt['status']}")
```

### Sending ETH

```python
# Send 1 ETH with the transaction
tx_hash = contract.write.deposit(
    value=10**18,  # 1 ETH in wei
)
```

### Custom Gas Parameters

```python
# Set gas limit and gas price
tx_hash = contract.write.transfer(
    recipient,
    amount,
    gas=100_000,
    gas_price=20 * 10**9,  # 20 gwei
)
```

### Custom Security Parameters

```python
from seismic_web3 import SeismicSecurityParams

# Use longer expiry window (200 blocks instead of 100)
security = SeismicSecurityParams(blocks_window=200)

tx_hash = contract.write.transfer(
    recipient,
    amount,
    security=security,
)
```

### Combining All Parameters

```python
from seismic_web3 import SeismicSecurityParams

security = SeismicSecurityParams(blocks_window=150)

tx_hash = contract.write.deposit(
    value=10**17,           # 0.1 ETH
    gas=200_000,
    gas_price=25 * 10**9,   # 25 gwei
    security=security,
)
```

***

## Return Value

Returns `HexBytes` containing the transaction hash.

```python
# Sync
tx_hash = contract.write.transfer(recipient, amount)
assert isinstance(tx_hash, HexBytes)

# Async
tx_hash = await contract.write.transfer(recipient, amount)
assert isinstance(tx_hash, HexBytes)
```

You can:
- Convert to hex string: `tx_hash.to_0x_hex()`
- Convert to bytes: `bytes(tx_hash)`
- Wait for receipt: `w3.eth.wait_for_transaction_receipt(tx_hash)`

***

## Privacy Guarantees

### What Gets Encrypted

- Function selector (4 bytes)
- All function arguments
- Encoding metadata

An observer watching the network can see:
- Your address (transaction sender)
- Contract address (transaction recipient)
- Value transferred (if non-zero)
- Gas used

But **cannot** see:
- Which function you called
- What arguments you passed
- Any data in the calldata

### What Remains Visible

These fields are **not** encrypted:
- `from` — Your wallet address
- `to` — Contract address
- `value` — ETH amount sent
- `gas` and `gas_price` — Gas parameters
- `nonce` — Transaction nonce
- Transaction metadata (block hash, expiry, encryption nonce)

***

## Security Considerations

### Block Hash Freshness

Every shielded transaction includes a recent block hash as a freshness proof. The node validates that:
1. The block hash corresponds to a real block
2. The block is recent (within the chain's freshness window)

This prevents:
- Replay attacks across chains
- Stale transaction submissions

### Transaction Expiry

Transactions include an expiry block number. After this block:
- The node will reject the transaction
- You must create a new transaction with updated parameters

Default expiry: **100 blocks** (~20 minutes on most chains)

### Nonce Uniqueness

Each transaction uses a cryptographically random 12-byte encryption nonce. **Never reuse nonces** — this breaks AES-GCM security.

The SDK generates random nonces automatically. Only override if you know what you're doing (e.g., testing).

***

## Error Handling

```python
from web3.exceptions import TransactionNotFound, TimeExhausted

try:
    tx_hash = contract.write.transfer(recipient, amount)
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

## Comparison with Other Namespaces

| Namespace | Encryption | Transaction Type | Gas Cost | Use Case |
|-----------|-----------|-----------------|----------|----------|
| `.write` | Encrypted calldata | `TxSeismic` (0x4a) | Standard + encryption overhead | Privacy-sensitive writes |
| `.twrite` | No encryption | `eth_sendTransaction` | Standard | Public writes, lower cost |
| `.dwrite` | Encrypted + debug info | `TxSeismic` (0x4a) | Same as `.write` | Development/debugging |

***

## Best Practices

### Use `.write` When

- Function arguments contain sensitive data (amounts, addresses, private state)
- Privacy is required (trading, voting, auctions)
- Compliance requires on-chain confidentiality

### Don't Use `.write` When

- Function is view-only (use `.read` instead)
- No privacy needed and gas optimization is priority (use `.twrite`)
- Data is already public (e.g., reading public constants)

### Production Checklist

- Use default security parameters (don't override unless necessary)
- Handle transaction failures gracefully
- Wait for transaction confirmation before assuming success
- Monitor gas prices and adjust if needed
- Test with `.dwrite` first to verify calldata encoding

***

## Low-Level Alternative

If you need more control (e.g., contract deployment, pre-encoded calldata):

```python
from hexbytes import HexBytes

tx_hash = w3.seismic.send_shielded_transaction(
    to="0x...",
    data=HexBytes("0x..."),
    value=0,
    gas=100_000,
    gas_price=10**9,
)
```

See [Shielded Write Guide](../../guides/shielded-write.md#low-level-api) for details.

***

## See Also

- [Shielded Write Guide](../../guides/shielded-write.md) — Full encryption workflow
- [SeismicSecurityParams](../../api-reference/transaction-types/seismic-security-params.md) — Security parameter reference
- [.read Namespace](read.md) — Encrypted signed reads
- [.dwrite Namespace](dwrite.md) — Debug writes with inspection
- [.twrite Namespace](twrite.md) — Transparent writes without encryption
