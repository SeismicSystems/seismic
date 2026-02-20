---
description: Unencrypted transaction view for debugging
icon: eye
---

# PlaintextTx

Unencrypted transaction view returned by debug writes.

## Overview

`PlaintextTx` contains the same fields as a shielded transaction but with **plaintext** calldata (before AES-GCM encryption). It's returned by `.dwrite` methods for debugging and inspection.

## Definition

```python
@dataclass(frozen=True)
class PlaintextTx:
    """Unencrypted transaction view returned by debug writes.

    Contains the same fields as the shielded transaction but with
    plaintext calldata (before AES-GCM encryption).
    """
    to: ChecksumAddress | None
    data: HexBytes
    nonce: int
    gas: int
    gas_price: int
    value: int
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `to` | `ChecksumAddress \| None` | Recipient address, or `None` for contract creation |
| `data` | `HexBytes` | **Plaintext** calldata (before encryption) |
| `nonce` | `int` | Sender's transaction count |
| `gas` | `int` | Gas limit |
| `gas_price` | `int` | Gas price in wei |
| `value` | `int` | Amount of wei to transfer |

## Examples

### Get from Debug Write

```python
from seismic_web3 import create_wallet_client, ShieldedContract

w3 = create_wallet_client(...)
contract = ShieldedContract(w3, contract_address, abi)

# Debug write returns plaintext view
result = await contract.dwrite.transfer(recipient, 1000)

# Access plaintext transaction
plaintext = result.plaintext_tx
print(f"To: {plaintext.to}")
print(f"Plaintext data: {plaintext.data.to_0x_hex()}")
print(f"Gas: {plaintext.gas}")
print(f"Value: {plaintext.value} wei")
```

### Compare with Shielded Transaction

```python
result = await contract.dwrite.transfer(recipient, 1000)

# Plaintext calldata (readable)
plaintext_data = result.plaintext_tx.data
print(f"Plaintext: {plaintext_data.to_0x_hex()}")

# Encrypted calldata (ciphertext)
encrypted_data = result.shielded_tx.data
print(f"Encrypted: {encrypted_data.to_0x_hex()}")

# They have different values!
assert plaintext_data != encrypted_data
```

### Inspect Function Call

```python
from eth_abi import decode

result = await contract.dwrite.transfer(recipient, 1000)

# Decode plaintext calldata
plaintext_data = result.plaintext_tx.data

# Extract function selector (first 4 bytes)
selector = plaintext_data[:4]
print(f"Function selector: {selector.to_0x_hex()}")

# Decode parameters (after selector)
params = plaintext_data[4:]
decoded = decode(['address', 'uint256'], params)
print(f"Decoded params: {decoded}")
```

### Gas Estimation

```python
result = await contract.dwrite.transfer(recipient, 1000)

# Check estimated gas
plaintext = result.plaintext_tx
print(f"Gas estimate: {plaintext.gas}")

# Compare with actual gas used after execution
receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash)
print(f"Gas used: {receipt['gasUsed']}")
```

## Field Details

### data (Plaintext Calldata)

The `data` field contains **plaintext** calldata:
- Function selector (4 bytes) + encoded parameters
- **Not encrypted** - readable and inspectable
- Matches what would be sent in a standard Ethereum transaction

This contrasts with [`UnsignedSeismicTx.data`](unsigned-seismic-tx.md#data-encrypted-calldata), which contains the **encrypted** version.

### to

- Standard checksummed Ethereum address
- `None` for contract creation

### gas

- Estimated gas limit for the transaction
- Fetched from `eth_estimateGas` with plaintext calldata

### gas_price

- Gas price in wei
- Fetched from `eth_gasPrice` or configured value

### value

- Amount of native currency to transfer in wei
- Can be `0` for pure function calls

## Properties

- **Immutable** - Cannot be modified after construction (`frozen=True`)
- **Plaintext** - Data is not encrypted
- **Debug only** - Used for inspection, not for actual execution

## Use Cases

### Debugging Encryption

```python
result = await contract.dwrite.someMethod(args)

# Verify plaintext is correct before encryption
plaintext = result.plaintext_tx
print(f"Plaintext calldata: {plaintext.data.to_0x_hex()}")
```

### Testing Function Encoding

```python
result = await contract.dwrite.transfer(recipient, 1000)

# Verify function selector and parameter encoding
plaintext_data = result.plaintext_tx.data
expected_selector = w3.keccak(text="transfer(address,uint256)")[:4]

assert plaintext_data[:4] == expected_selector
```

### Gas Optimization

```python
# Compare gas estimates for different approaches
result1 = await contract.dwrite.method1(args)
result2 = await contract.dwrite.method2(args)

print(f"Method 1 gas: {result1.plaintext_tx.gas}")
print(f"Method 2 gas: {result2.plaintext_tx.gas}")
```

## Warnings

- **Never broadcast PlaintextTx** - It's unencrypted and invalid for Seismic
- **Debug only** - Only use `.dwrite` for testing, not production
- **Already broadcast** - `.dwrite` sends the **shielded** transaction, not plaintext

## Notes

- Part of [`DebugWriteResult`](debug-write-result.md)
- Only available with `.dwrite` namespace, not `.write`
- The actual broadcast transaction uses encrypted data from `shielded_tx`
- Useful for debugging calldata encoding issues

## See Also

- [DebugWriteResult](debug-write-result.md) - Contains PlaintextTx
- [UnsignedSeismicTx](unsigned-seismic-tx.md) - Shielded version with encrypted data
- [ShieldedContract](../../contract/shielded-contract.md) - Provides `.dwrite` namespace
