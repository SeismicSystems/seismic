---
description: Debug write namespace for encrypted transactions with inspection
icon: bug
---

# .dwrite Namespace

The `.dwrite` namespace provides encrypted contract write operations with debug inspection. It broadcasts a real transaction (like `.write`) but also returns both plaintext and encrypted views for debugging and testing.

***

## Overview

When you call `contract.dwrite.functionName(...)`, the SDK:

1. Encodes your function call using the contract ABI (plaintext)
2. Encrypts the calldata using AES-GCM with a shared key derived via ECDH
3. Constructs a `TxSeismic` with encryption metadata
4. Signs and broadcasts the encrypted transaction
5. Returns a `DebugWriteResult` containing:
   - **plaintext_tx** — Transaction before encryption
   - **shielded_tx** — Transaction with encrypted calldata
   - **tx_hash** — Transaction hash from broadcast

**Important**: The transaction is **actually broadcast** and consumes gas. This is not a dry run.

***

## Usage Pattern

```python
result = contract.dwrite.functionName(arg1, arg2, ...)
```

- **Sync**: Returns `DebugWriteResult` immediately
- **Async**: Returns `DebugWriteResult` (must `await`)

***

## Parameters

### Function Arguments

Pass function arguments as positional parameters:

```python
# Single argument
result = contract.dwrite.setNumber(42)

# Multiple arguments
result = contract.dwrite.transfer(recipient_address, 1000)

# Complex types
result = contract.dwrite.batchTransfer(
    ["0x123...", "0x456..."],
    [100, 200],
)
```

### Transaction Options (Keyword Arguments)

All transaction options are **optional** keyword arguments (same as `.write`):

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `value` | `int` | `0` | ETH value to send (in wei) |
| `gas` | `int \| None` | `None` | Gas limit (auto-estimated if `None`) |
| `gas_price` | `int \| None` | `None` | Gas price in wei (uses network default if `None`) |
| `security` | `SeismicSecurityParams \| None` | `None` | Custom security parameters (block expiry, nonce, etc.) |

***

## Return Value

Returns a `DebugWriteResult` with three fields:

```python
@dataclass(frozen=True)
class DebugWriteResult:
    plaintext_tx: PlaintextTx           # Transaction before encryption
    shielded_tx: UnsignedSeismicTx      # Transaction with encrypted calldata
    tx_hash: HexBytes                    # Transaction hash
```

### Field Details

| Field | Type | Description |
|-------|------|-------------|
| `plaintext_tx` | `PlaintextTx` | Transaction with unencrypted calldata (what you intended) |
| `shielded_tx` | `UnsignedSeismicTx` | Full `TxSeismic` structure with encrypted calldata |
| `tx_hash` | `HexBytes` | Transaction hash from `eth_sendRawTransaction` |

***

## Examples

### Sync Usage

```python
from seismic_web3 import create_wallet_client

# Create client and contract
w3 = create_wallet_client(...)
contract = w3.seismic.contract(address="0x...", abi=ABI)

# Debug write
result = contract.dwrite.setNumber(42)

# Access all three components
print(f"Transaction hash: {result.tx_hash.to_0x_hex()}")
print(f"Plaintext data: {result.plaintext_tx.data.to_0x_hex()}")
print(f"Encrypted data: {result.shielded_tx.data.to_0x_hex()}")

# Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash)
print(f"Status: {receipt['status']}")
```

### Async Usage

```python
from seismic_web3 import create_async_wallet_client

# Create async client and contract
w3 = await create_async_wallet_client(...)
contract = w3.seismic.contract(address="0x...", abi=ABI)

# Debug write
result = await contract.dwrite.setNumber(42)

# Access components
print(f"Transaction hash: {result.tx_hash.to_0x_hex()}")

# Wait for confirmation
receipt = await w3.eth.wait_for_transaction_receipt(result.tx_hash)
print(f"Status: {receipt['status']}")
```

### Inspect Plaintext Calldata

```python
result = contract.dwrite.transfer(recipient, 1000)

# Examine what was encoded before encryption
plaintext_data = result.plaintext_tx.data
print(f"Function selector: {plaintext_data[:4].to_0x_hex()}")
print(f"Full calldata: {plaintext_data.to_0x_hex()}")
```

### Verify Encryption

```python
result = contract.dwrite.transfer(recipient, 1000)

# Compare plaintext vs encrypted
plaintext_len = len(result.plaintext_tx.data)
encrypted_len = len(result.shielded_tx.data)

print(f"Plaintext length: {plaintext_len}")
print(f"Encrypted length: {encrypted_len}")
print(f"Difference: {encrypted_len - plaintext_len} bytes")

# AES-GCM adds 16-byte authentication tag
assert encrypted_len == plaintext_len + 16
```

### Inspect Transaction Parameters

```python
result = contract.dwrite.transfer(
    recipient,
    amount,
    value=10**17,
    gas=150_000,
)

# Access plaintext parameters
print(f"To: {result.plaintext_tx.to}")
print(f"Value: {result.plaintext_tx.value} wei")
print(f"Gas: {result.plaintext_tx.gas}")

# Access shielded parameters
shielded = result.shielded_tx
print(f"Nonce: {shielded.nonce}")
print(f"Gas price: {shielded.gas_price}")
print(f"Expires at block: {shielded.seismic.expires_at_block}")
```

### Analyze Gas Estimation

```python
result = contract.dwrite.complexMethod(args)

# Estimated gas
estimated = result.plaintext_tx.gas
print(f"Estimated gas: {estimated}")

# Wait for receipt
receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash)
actual = receipt['gasUsed']

print(f"Actual gas used: {actual}")
print(f"Difference: {actual - estimated}")
print(f"Estimation accuracy: {(estimated / actual) * 100:.2f}%")
```

### Compare with Production .write

```python
# Development: use .dwrite to inspect
result = contract.dwrite.transfer(recipient, amount)
print(f"Plaintext: {result.plaintext_tx.data.to_0x_hex()}")
tx_hash = result.tx_hash

# Production: use .write (same behavior, no debug info)
tx_hash = contract.write.transfer(recipient, amount)
```

***

## Use Cases

### Development and Testing

```python
# Verify calldata is correctly encoded
result = contract.dwrite.complexMethod(arg1, arg2, arg3)

# Inspect function selector
selector = result.plaintext_tx.data[:4]
print(f"Function selector: {selector.to_0x_hex()}")

# Verify it matches expected selector
from web3 import Web3
expected_selector = Web3.keccak(text="complexMethod(uint256,address,bool)")[:4]
assert selector == expected_selector
```

### Debugging Encryption

```python
result = contract.dwrite.problematicMethod(args)

# Compare plaintext vs encrypted
print("Plaintext calldata:")
print(result.plaintext_tx.data.to_0x_hex())

print("\nEncrypted calldata:")
print(result.shielded_tx.data.to_0x_hex())

# Verify lengths
print(f"\nPlaintext: {len(result.plaintext_tx.data)} bytes")
print(f"Encrypted: {len(result.shielded_tx.data)} bytes")
```

### Auditing Transaction Details

```python
result = contract.dwrite.highValueTransfer(recipient, large_amount)

# Audit all parameters
plaintext = result.plaintext_tx
shielded = result.shielded_tx

print(f"From: {w3.eth.default_account}")
print(f"To: {plaintext.to}")
print(f"Value: {plaintext.value} wei ({plaintext.value / 10**18} ETH)")
print(f"Gas limit: {plaintext.gas}")
print(f"Expires at block: {shielded.seismic.expires_at_block}")
print(f"Transaction hash: {result.tx_hash.to_0x_hex()}")
```

### Testing Security Parameters

```python
from seismic_web3 import SeismicSecurityParams, EncryptionNonce

# Use explicit nonce for testing
nonce = EncryptionNonce(b'\x00' * 12)
security = SeismicSecurityParams(
    blocks_window=200,
    encryption_nonce=nonce,
)

result = contract.dwrite.transfer(
    recipient,
    amount,
    security=security,
)

# Verify parameters were applied
assert result.shielded_tx.seismic.encryption_nonce == nonce
```

***

## Important Warnings

### Transaction is Actually Broadcast

**`.dwrite` is NOT a dry run**:
- The encrypted transaction is broadcast
- Gas is consumed
- State changes are applied
- You pay transaction fees

### Use in Production

**Don't use `.dwrite` in production**:
- Adds overhead (returns extra data)
- No benefit over `.write` for end users
- Only useful for development/debugging

**For production, use `.write`**:
```python
# Development/testing
result = contract.dwrite.transfer(recipient, amount)
tx_hash = result.tx_hash

# Production
tx_hash = contract.write.transfer(recipient, amount)
```

### Cost Implications

`.dwrite` has the **same gas cost** as `.write`:
- Same transaction is broadcast
- Same encryption overhead
- Only difference is SDK returns debug info (no on-chain difference)

***

## Comparison with Other Namespaces

| Namespace | Encryption | Returns | Transaction Broadcast | Use Case |
|-----------|-----------|---------|---------------------|----------|
| `.write` | Yes | `HexBytes` (tx hash) | Yes | Production shielded writes |
| `.dwrite` | Yes | `DebugWriteResult` | Yes | Development/debugging |
| `.twrite` | No | `HexBytes` (tx hash) | Yes | Transparent writes |

***

## Privacy Guarantees

### Identical to .write

`.dwrite` has the **same privacy guarantees** as `.write`:
- Calldata is encrypted on-chain
- Only you and the TEE can see plaintext
- Debug info is **only in the SDK return value** (not on-chain)

### Debug Info is Client-Side

The `plaintext_tx` and `shielded_tx` fields are **not** broadcast:
- Only the encrypted transaction is sent
- Debug info exists only in your client
- No privacy leak from using `.dwrite`

***

## Error Handling

```python
from web3.exceptions import TimeExhausted

try:
    result = contract.dwrite.transfer(recipient, amount)

    # Transaction broadcast succeeded
    print(f"Tx hash: {result.tx_hash.to_0x_hex()}")

    # Wait for confirmation
    receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash, timeout=120)

    if receipt['status'] == 0:
        print("Transaction reverted")
        # Inspect calldata to debug
        print(f"Plaintext: {result.plaintext_tx.data.to_0x_hex()}")
    else:
        print("Transaction succeeded")

except ValueError as e:
    print(f"Transaction failed: {e}")
except TimeExhausted:
    print("Transaction not mined within timeout")
```

***

## Best Practices

### When to Use .dwrite

- **Development** — Verify calldata encoding is correct
- **Testing** — Debug encryption/decryption issues
- **Auditing** — Inspect transaction parameters before production
- **Debugging** — Troubleshoot failed transactions

### When to Use .write Instead

- **Production** — No need for debug info
- **High-frequency calls** — Avoid overhead of extra return data
- **Simple operations** — Debug info not useful

### Testing Workflow

```python
# 1. Test with .dwrite
result = contract.dwrite.transfer(recipient, small_amount)

# 2. Inspect calldata
print(f"Plaintext: {result.plaintext_tx.data.to_0x_hex()}")

# 3. Verify encryption worked
assert result.plaintext_tx.data != result.shielded_tx.data

# 4. Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash)

# 5. If successful, switch to .write for production
if receipt['status'] == 1:
    # Use .write in production
    tx_hash = contract.write.transfer(recipient, large_amount)
```

***

## Low-Level Alternative

For manual control with debug info:

```python
from hexbytes import HexBytes

result = w3.seismic.debug_send_shielded_transaction(
    to="0x...",
    data=HexBytes("0x..."),
    value=0,
    gas=100_000,
    gas_price=10**9,
)

# Access debug fields
print(result.plaintext_tx.data.to_0x_hex())
print(result.shielded_tx.data.to_0x_hex())
print(result.tx_hash.to_0x_hex())
```

See [Shielded Write Guide](../../guides/shielded-write.md#low-level-api) for details.

***

## See Also

- [.write Namespace](write.md) — Production shielded writes (no debug info)
- [DebugWriteResult](../../api-reference/transaction-types/debug-write-result.md) — Return type reference
- [PlaintextTx](../../api-reference/transaction-types/plaintext-tx.md) — Plaintext transaction structure
- [UnsignedSeismicTx](../../api-reference/transaction-types/unsigned-seismic-tx.md) — Shielded transaction structure
- [Shielded Write Guide](../../guides/shielded-write.md) — Full encryption workflow
