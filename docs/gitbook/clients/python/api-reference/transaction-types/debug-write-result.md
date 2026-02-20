---
description: Result from debug shielded write
icon: bug
---

# DebugWriteResult

Result from a debug shielded write (`.dwrite` namespace).

## Overview

`DebugWriteResult` is returned by `.dwrite` methods, providing both plaintext and encrypted views of the transaction along with the transaction hash. The transaction **is** broadcast (like `.write`), but you also get inspection data for debugging.

## Definition

```python
@dataclass(frozen=True)
class DebugWriteResult:
    """Result from a debug shielded write (dwrite).

    The transaction is broadcast (like .write), but the caller
    also gets both the plaintext and encrypted views for inspection.
    """
    plaintext_tx: PlaintextTx
    shielded_tx: UnsignedSeismicTx
    tx_hash: HexBytes
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `plaintext_tx` | [`PlaintextTx`](plaintext-tx.md) | Transaction parameters with unencrypted calldata |
| `shielded_tx` | [`UnsignedSeismicTx`](unsigned-seismic-tx.md) | Full unsigned TxSeismic with encrypted calldata |
| `tx_hash` | `HexBytes` | Transaction hash from `eth_sendRawTransaction` |

## Examples

### Basic Usage

```python
from seismic_web3 import create_wallet_client, ShieldedContract

w3 = create_wallet_client(...)
contract = ShieldedContract(w3, contract_address, abi)

# Debug write broadcasts and returns debug info
result = await contract.dwrite.transfer(recipient, 1000)

# Access all three components
print(f"Transaction hash: {result.tx_hash.to_0x_hex()}")
print(f"Plaintext data: {result.plaintext_tx.data.to_0x_hex()}")
print(f"Encrypted data: {result.shielded_tx.data.to_0x_hex()}")
```

### Wait for Receipt

```python
result = await contract.dwrite.transfer(recipient, 1000)

# Wait for transaction to be mined
receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash)

print(f"Status: {receipt['status']}")
print(f"Gas used: {receipt['gasUsed']}")
print(f"Block number: {receipt['blockNumber']}")
```

### Compare Plaintext vs Encrypted

```python
result = await contract.dwrite.transfer(recipient, 1000)

# Inspect plaintext
plaintext_data = result.plaintext_tx.data
print(f"Plaintext calldata: {plaintext_data.to_0x_hex()}")

# Inspect encrypted version
encrypted_data = result.shielded_tx.data
print(f"Encrypted calldata: {encrypted_data.to_0x_hex()}")

# Verify lengths differ (AES-GCM adds authentication tag)
assert len(encrypted_data) == len(plaintext_data) + 16  # 16-byte auth tag
```

### Inspect Transaction Fields

```python
result = await contract.dwrite.transfer(recipient, 1000)

# Access shielded transaction details
shielded = result.shielded_tx
print(f"Nonce: {shielded.nonce}")
print(f"Gas: {shielded.gas}")
print(f"Gas price: {shielded.gas_price}")
print(f"Value: {shielded.value}")

# Access Seismic elements
elements = shielded.seismic
print(f"Expires at block: {elements.expires_at_block}")
print(f"Message version: {elements.message_version}")
print(f"Encryption nonce: {elements.encryption_nonce.to_0x_hex()}")
```

### Debug Encryption

```python
result = await contract.dwrite.someMethod(complex_args)

# Verify plaintext was correctly encoded
plaintext = result.plaintext_tx
print(f"Function selector: {plaintext.data[:4].to_0x_hex()}")

# Check encryption was applied
shielded = result.shielded_tx
assert plaintext.data != shielded.data  # Must differ

# Verify transaction was broadcast
print(f"Tx hash: {result.tx_hash.to_0x_hex()}")
```

### Gas Analysis

```python
result = await contract.dwrite.transfer(recipient, 1000)

# Estimated gas (from plaintext gas estimation)
estimated = result.plaintext_tx.gas
print(f"Estimated gas: {estimated}")

# Wait for receipt and compare
receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash)
actual = receipt['gasUsed']
print(f"Actual gas used: {actual}")
print(f"Difference: {actual - estimated}")
```

## Behavior

### Transaction is Broadcast

Unlike a dry run, `.dwrite` **actually broadcasts** the transaction:
- The **shielded** transaction (with encrypted data) is sent
- The transaction is included in a block
- State changes are applied
- Gas is consumed

### Both Views Provided

You get both perspectives:
- **Plaintext** - What the function call looks like before encryption
- **Shielded** - What actually got broadcast (with encrypted calldata)

This is useful for:
- Debugging calldata encoding
- Verifying encryption is working
- Inspecting transaction parameters
- Testing before production

## Use Cases

### Development and Testing

```python
# Use .dwrite during development
result = await contract.dwrite.transfer(recipient, 1000)

# Inspect plaintext to verify correctness
assert result.plaintext_tx.data[:4] == transfer_selector
```

### Debugging Encryption Issues

```python
result = await contract.dwrite.problematicMethod(args)

# Compare plaintext vs encrypted
print("Plaintext:", result.plaintext_tx.data.to_0x_hex())
print("Encrypted:", result.shielded_tx.data.to_0x_hex())
```

### Auditing Transaction Parameters

```python
result = await contract.dwrite.highValueTransfer(recipient, large_amount)

# Audit all parameters before considering it successful
plaintext = result.plaintext_tx
shielded = result.shielded_tx

print(f"To: {plaintext.to}")
print(f"Value: {plaintext.value} wei")
print(f"Expires: {shielded.seismic.expires_at_block}")
```

## Properties

- **Immutable** - Cannot be modified after construction (`frozen=True`)
- **Transaction broadcast** - The shielded tx is actually sent
- **Debug only** - Use `.write` in production

## Warnings

- **Real transactions** - `.dwrite` consumes gas and changes state
- **Not a dry run** - The transaction is actually broadcast
- **Use `.write` in production** - `.dwrite` is for debugging only
- **Cost** - Costs gas just like `.write`

## Production Alternative

For production code, use `.write` instead:

```python
# Development/testing
result = await contract.dwrite.transfer(recipient, 1000)
tx_hash = result.tx_hash

# Production
tx_hash = await contract.write.transfer(recipient, 1000)
```

The `.write` method returns just the transaction hash, without debug information.

## Notes

- Only available on shielded contracts with `.dwrite` namespace
- The `shielded_tx` field is **unsigned** (signature not included)
- The broadcast transaction includes the signature (added by SDK)
- Useful for debugging but adds overhead

## See Also

- [PlaintextTx](plaintext-tx.md) - Plaintext transaction view
- [UnsignedSeismicTx](unsigned-seismic-tx.md) - Shielded transaction structure
- [ShieldedContract](../../contract/shielded-contract.md) - Provides `.dwrite` namespace
- [Shielded Write Guide](../../guides/shielded-write.md) - Full workflow examples
