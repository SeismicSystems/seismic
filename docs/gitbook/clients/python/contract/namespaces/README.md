---
description: Contract interaction namespaces for shielded and transparent operations
icon: code-branch
---

# Contract Namespaces

Seismic contracts expose five namespaces for different types of operations. Each namespace provides a different combination of encryption, authentication, and transaction behavior.

***

## Overview

When you instantiate a contract:

```python
contract = w3.seismic.contract(address="0x...", abi=ABI)
```

You get access to five namespaces:

| Namespace | Operation | Encryption | Identity (`msg.sender`) | Broadcasts | Use Case |
|-----------|-----------|-----------|----------------------|-----------|----------|
| [`.write`](write.md) | Write | Yes | Your address | Yes | Privacy-sensitive writes |
| [`.read`](read.md) | Read | Yes | Your address | No | Access-controlled reads |
| [`.twrite`](twrite.md) | Write | No | Your address | Yes | Public writes |
| [`.tread`](tread.md) | Read | No | `0x0` | No | Public reads |
| [`.dwrite`](dwrite.md) | Write | Yes | Your address | Yes | Debug/testing |

***

## Quick Comparison

### Encrypted vs Transparent

**Encrypted** (`.write`, `.read`, `.dwrite`):
- Calldata is encrypted using AES-GCM
- Only you and the TEE can see plaintext
- Requires ECDH key exchange with node
- Includes security metadata (nonce, block hash, expiry)
- Higher gas cost (encryption overhead)

**Transparent** (`.twrite`, `.tread`):
- Calldata is visible on-chain
- Standard Ethereum transactions/calls
- No encryption or security metadata
- Lower gas cost (no overhead)
- Faster (no encryption computation)

### Write vs Read

**Write** (`.write`, `.twrite`, `.dwrite`):
- Broadcasts a transaction
- Consumes gas
- Modifies contract state
- Returns transaction hash
- Must wait for confirmation

**Read** (`.read`, `.tread`):
- Executes an `eth_call`
- Free (no gas cost)
- Does not modify state
- Returns result immediately
- No confirmation needed

***

## Usage Patterns

### Encrypted Write (.write)

```python
# Privacy-sensitive transaction
tx_hash = contract.write.transfer(recipient, 1000)

# Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
```

**When to use**:
- Amounts, addresses, or arguments should be private
- Privacy compliance required
- Trading, voting, auctions, confidential transfers

### Signed Read (.read)

```python
# Encrypted read that proves your identity — auto-decoded
balance = contract.read.balanceOf()  # int
```

**When to use**:
- Contract checks `msg.sender` for access control
- Caller-specific data (e.g., "my balance")
- Privacy required for queries
- Result should be encrypted

### Transparent Write (.twrite)

```python
# Public transaction
tx_hash = contract.twrite.approve(spender, amount)

# Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
```

**When to use**:
- Data is public anyway
- Lower gas cost matters
- No privacy requirements
- Standard Ethereum behavior

### Transparent Read (.tread)

```python
# Public read — auto-decoded
total_supply = contract.tread.totalSupply()  # int
```

**When to use**:
- Function is public (doesn't check `msg.sender`)
- No authentication needed
- Data is public
- Fast queries for analytics/dashboards

### Debug Write (.dwrite)

```python
# Debug transaction with inspection
result = contract.dwrite.transfer(recipient, 1000)

# Inspect plaintext calldata
print(f"Plaintext: {result.plaintext_tx.data.to_0x_hex()}")
print(f"Encrypted: {result.shielded_tx.data.to_0x_hex()}")
print(f"Tx hash: {result.tx_hash.to_0x_hex()}")

# Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash)
```

**When to use**:
- Development and testing
- Debugging encryption issues
- Verifying calldata encoding
- Auditing transaction parameters

***

## Decision Tree

### Choosing the Right Namespace

```
Is it a write operation (modifies state)?
├─ Yes: Write namespace
│  ├─ Is privacy required?
│  │  ├─ Yes: Use .write
│  │  └─ No: Use .twrite
│  └─ Are you debugging?
│     └─ Yes: Use .dwrite (development only)
│
└─ No: Read namespace
   ├─ Does function check msg.sender?
   │  ├─ Yes: Use .read (signed read)
   │  └─ No: Use .tread (standard call)
   └─ Is privacy required?
      ├─ Yes: Use .read
      └─ No: Use .tread
```

***

## Common Examples

### Token Transfer (Private)

```python
# Private transfer — amount and recipient hidden
tx_hash = contract.write.transfer(recipient, amount)
```

### Token Transfer (Public)

```python
# Public transfer — standard ERC20
tx_hash = contract.twrite.transfer(recipient, amount)
```

### Check Your Balance (Access-Controlled)

```python
# SRC20 balanceOf uses msg.sender — auto-decoded
balance = contract.read.balanceOf()  # int
```

### Check Any Balance (Public)

```python
# Function takes address argument — auto-decoded
balance = contract.tread.balanceOf(address)  # int
```

### Token Metadata (Public)

```python
# Public getters — auto-decoded
name = contract.tread.name()            # str
symbol = contract.tread.symbol()        # str
decimals = contract.tread.decimals()    # int
```

***

## Namespace Details

### .write

**Encrypted contract writes with privacy.**

- **Encryption**: Yes (AES-GCM)
- **Transaction type**: `TxSeismic` (type `0x4a`)
- **Returns**: `HexBytes` (transaction hash)
- **Proves identity**: Yes
- **Gas cost**: Standard + encryption overhead

[See full documentation →](write.md)

### .read

**Encrypted contract reads with authentication.**

- **Encryption**: Yes (AES-GCM)
- **Call type**: Signed `eth_call`
- **Returns**: `Any` (ABI-decoded Python value)
- **Proves identity**: Yes (`msg.sender` is your address)
- **Gas cost**: None (doesn't broadcast)

[See full documentation →](read.md)

### .twrite

**Transparent contract writes (standard Ethereum).**

- **Encryption**: No
- **Transaction type**: Standard `eth_sendTransaction`
- **Returns**: `HexBytes` (transaction hash)
- **Proves identity**: Yes
- **Gas cost**: Standard (no overhead)

[See full documentation →](twrite.md)

### .tread

**Transparent contract reads (standard Ethereum).**

- **Encryption**: No
- **Call type**: Standard `eth_call`
- **Returns**: `Any` (ABI-decoded Python value)
- **Proves identity**: No (`msg.sender` is `0x0`)
- **Gas cost**: None (doesn't broadcast)

[See full documentation →](tread.md)

### .dwrite

**Debug writes with inspection (development only).**

- **Encryption**: Yes (AES-GCM)
- **Transaction type**: `TxSeismic` (type `0x4a`)
- **Returns**: `DebugWriteResult` (plaintext + encrypted + hash)
- **Proves identity**: Yes
- **Gas cost**: Standard + encryption overhead

[See full documentation →](dwrite.md)

***

## Privacy Considerations

### What Gets Encrypted (Shielded Namespaces)

**Encrypted** (`.write`, `.read`, `.dwrite`):
- Function selector (4 bytes)
- All function arguments
- Return values (for `.read`)

**Not encrypted**:
- `from` address (sender)
- `to` address (contract)
- `value` (ETH sent)
- Gas parameters
- Transaction metadata

### What's Visible (Transparent Namespaces)

**Completely visible** (`.twrite`, `.tread`):
- Everything: function selector, arguments, results
- Standard Ethereum transparency
- Anyone can decode calldata

***

## Security Parameters

Encrypted namespaces (`.write`, `.read`, `.dwrite`) accept optional security parameters:

```python
from seismic_web3 import SeismicSecurityParams

security = SeismicSecurityParams(
    blocks_window=200,          # Expiry window (default: 100 blocks)
    encryption_nonce=None,      # Random nonce (default: random)
    recent_block_hash=None,     # Recent block (default: latest)
    expires_at_block=None,      # Explicit expiry (default: computed)
)

tx_hash = contract.write.transfer(
    recipient,
    amount,
    security=security,
)
```

See [SeismicSecurityParams](../../api-reference/transaction-types/seismic-security-params.md) for details.

***

## Transaction Options

### Write Namespaces

All write namespaces (`.write`, `.twrite`, `.dwrite`) accept transaction options:

```python
tx_hash = contract.write.transfer(
    recipient,
    amount,
    value=10**18,           # ETH to send (wei)
    gas=100_000,            # Gas limit
    gas_price=20 * 10**9,   # Gas price (wei)
    security=params,        # Security params (.write/.dwrite only)
)
```

`.twrite` also accepts any standard `eth_sendTransaction` parameter:
```python
tx_hash = contract.twrite.transfer(
    recipient,
    amount,
    maxFeePerGas=50 * 10**9,         # EIP-1559
    maxPriorityFeePerGas=2 * 10**9,  # EIP-1559
    nonce=42,                         # Explicit nonce
)
```

### Read Namespaces

`.read` accepts call options:
```python
result = contract.read.balanceOf(
    value=0,           # ETH for simulation
    gas=30_000_000,    # Gas limit
    security=params,   # Security params
)
```

`.tread` accepts no options (uses defaults).

***

## Async Support

All namespaces work identically with async clients:

```python
from seismic_web3 import create_async_wallet_client

w3 = await create_async_wallet_client(...)
contract = w3.seismic.contract(address="0x...", abi=ABI)

# All namespaces require await
tx_hash = await contract.write.transfer(recipient, amount)
result = await contract.read.balanceOf()
tx_hash = await contract.twrite.approve(spender, amount)
result = await contract.tread.totalSupply()
debug = await contract.dwrite.transfer(recipient, amount)
```

***

## Error Handling

### Write Operations

```python
from web3.exceptions import TimeExhausted

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

### Read Operations

```python
try:
    balance = contract.read.balanceOf()
    print(f"Balance: {balance}")

except ValueError as e:
    print(f"Call failed: {e}")
```

***

## Best Practices

### Privacy Guidelines

- **Sensitive data** → Use `.write` or `.read`
- **Public data** → Use `.twrite` or `.tread`
- **Access control** → Use `.read` (proves identity)
- **No access control** → Use `.tread` (faster)

### Cost Optimization

- **Privacy required** → Accept encryption overhead (`.write`, `.read`)
- **No privacy needed** → Use transparent namespaces (`.twrite`, `.tread`)
- **Frequent reads** → Use `.tread` (free, fast, cacheable)

***

## Common Pitfalls

### Using .tread for Access-Controlled Functions

**Problem**:
```python
# BAD: SRC20 balanceOf uses msg.sender — returns 0x0's balance
balance = contract.tread.balanceOf()  # 0
```

**Solution**:
```python
# GOOD: Proves your identity
balance = contract.read.balanceOf()  # Your actual balance
```

### Using .write for Public Data

**Unnecessary overhead**:
```python
# Wasteful: Encrypts already-public data
tx_hash = contract.write.approve(spender, amount)
```

**Better**:
```python
# More efficient: Use transparent write
tx_hash = contract.twrite.approve(spender, amount)
```

### Using .dwrite in Production

**Development only**:
```python
# BAD: .dwrite in production (unnecessary overhead)
result = contract.dwrite.transfer(recipient, amount)
tx_hash = result.tx_hash
```

**Production**:
```python
# GOOD: Use .write in production
tx_hash = contract.write.transfer(recipient, amount)
```

***

## See Also

- [Contract Instance](../README.md) — Contract wrapper reference
- [Shielded Write Guide](../../guides/shielded-write.md) — Full encryption workflow
- [Signed Read Guide](../../guides/signed-reads.md) — Authentication and privacy for reads
- [SeismicSecurityParams](../../api-reference/transaction-types/seismic-security-params.md) — Security parameters
- [DebugWriteResult](../../api-reference/transaction-types/debug-write-result.md) — Debug write return type
