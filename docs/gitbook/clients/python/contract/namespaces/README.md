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

| Namespace | Operation | Encryption | `msg.sender` | Broadcasts | Use Case |
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

***

## See Also

- [Contract Instance](../README.md) — Contract wrapper reference
- [Shielded Write Guide](../../guides/shielded-write.md) — Full encryption workflow
- [Signed Read Guide](../../guides/signed-reads.md) — Authentication and privacy for reads
- [SeismicSecurityParams](../../api-reference/transaction-types/seismic-security-params.md) — Security parameters
- [DebugWriteResult](../../api-reference/transaction-types/debug-write-result.md) — Debug write return type
