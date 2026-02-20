---
description: Send a shielded transaction and return debug information for inspection
icon: bug
---

# debug_send_shielded_transaction()

Send a shielded transaction with encrypted calldata and return debug information including both the plaintext and encrypted views of the transaction. This is useful for development, testing, and debugging encryption logic.

***

## Overview

This method works exactly like `send_shielded_transaction()` — it broadcasts a real transaction to the network — but also returns detailed debug information:

- **Plaintext transaction**: The transaction before encryption (readable calldata)
- **Encrypted transaction**: The full `TxSeismic` with encrypted calldata
- **Transaction hash**: The hash returned by the network

Use this during development to verify:
- Calldata is encoded correctly
- Encryption parameters are set as expected
- Transaction structure matches your expectations

***

## Signatures

<table>
<thead>
<tr>
<th width="400">Sync</th>
<th>Async</th>
</tr>
</thead>
<tbody>
<tr>
<td>

```python
def debug_send_shielded_transaction(
    *,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
    eip712: bool = False,
) -> DebugWriteResult
```

</td>
<td>

```python
async def debug_send_shielded_transaction(
    *,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
    eip712: bool = False,
) -> DebugWriteResult
```

</td>
</tr>
</tbody>
</table>

***

## Parameters

All parameters are **keyword-only** and identical to `send_shielded_transaction()`.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `to` | `ChecksumAddress` | Required | Recipient contract address |
| `data` | `HexBytes` | Required | Plaintext calldata (will be encrypted) |
| `value` | `int` | `0` | Wei to transfer with the transaction |
| `gas` | `int \| None` | `None` | Gas limit (auto-estimated if `None`) |
| `gas_price` | `int \| None` | `None` | Gas price in wei (uses network default if `None`) |
| `security` | [`SeismicSecurityParams`](../../api-reference/transaction-types/seismic-security-params.md) \| `None` | `None` | Custom security parameters (block hash, nonce, expiry) |
| `eip712` | `bool` | `False` | Use EIP-712 typed data signing instead of raw signing |

***

## Returns

**Type:** [`DebugWriteResult`](../../api-reference/transaction-types/debug-write-result.md)

A dataclass containing three fields:

| Field | Type | Description |
|-------|------|-------------|
| `tx_hash` | `HexBytes` | Transaction hash from the network |
| `plaintext_tx` | `PlaintextTx` | Transaction parameters with **unencrypted** calldata |
| `shielded_tx` | `UnsignedSeismicTx` | Full `TxSeismic` with **encrypted** calldata |

### PlaintextTx Structure

```python
@dataclass
class PlaintextTx:
    to: ChecksumAddress | None
    data: HexBytes              # Plaintext calldata
    nonce: int
    gas: int
    gas_price: int
    value: int
```

### UnsignedSeismicTx Structure

```python
@dataclass
class UnsignedSeismicTx:
    chain_id: int
    nonce: int
    gas_price: int
    gas: int
    to: ChecksumAddress | None
    value: int
    data: HexBytes              # Encrypted calldata
    seismic: SeismicElements    # Encryption metadata
```

***

## Examples

### Sync Usage

```python
from seismic_web3 import create_wallet_client, PrivateKey
from hexbytes import HexBytes

# Create wallet client
w3 = create_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=PrivateKey(b"...32 bytes..."),
)

# Send transaction with debug info
result = w3.seismic.debug_send_shielded_transaction(
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    data=HexBytes("0xa9059cbb000000000000000000000000..."),
    value=0,
)

# Inspect results
print(f"Transaction hash: {result.tx_hash.hex()}")
print(f"Plaintext calldata: {result.plaintext_tx.data.hex()}")
print(f"Encrypted calldata: {result.shielded_tx.data.hex()}")
print(f"Encryption nonce: {result.shielded_tx.seismic.encryption_nonce.hex()}")
print(f"Block hash: {result.shielded_tx.seismic.recent_block_hash.hex()}")
print(f"Expires at block: {result.shielded_tx.seismic.expires_at_block}")
```

### Async Usage

```python
from seismic_web3 import create_async_wallet_client, PrivateKey
from hexbytes import HexBytes

# Create async wallet client
w3 = await create_async_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=PrivateKey(b"...32 bytes..."),
)

# Send transaction with debug info
result = await w3.seismic.debug_send_shielded_transaction(
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    data=HexBytes("0xa9059cbb000000000000000000000000..."),
)

print(f"Transaction hash: {result.tx_hash.hex()}")
print(f"Plaintext: {result.plaintext_tx.data.hex()}")
print(f"Encrypted: {result.shielded_tx.data.hex()}")
```

### Verifying Encryption

```python
# Verify calldata was actually encrypted
result = w3.seismic.debug_send_shielded_transaction(
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    data=HexBytes("0x12345678abcdef"),
)

plaintext = result.plaintext_tx.data
encrypted = result.shielded_tx.data

print(f"Original: {plaintext.hex()}")
print(f"Encrypted: {encrypted.hex()}")
print(f"Same? {plaintext == encrypted}")  # Should be False
print(f"Length changed? {len(plaintext)} -> {len(encrypted)}")
```

### Inspecting Security Parameters

```python
from seismic_web3 import SeismicSecurityParams

# Send with custom security parameters
security = SeismicSecurityParams(blocks_window=200)

result = w3.seismic.debug_send_shielded_transaction(
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    data=HexBytes("0x..."),
    security=security,
)

# Check actual security parameters used
print(f"Block hash: {result.shielded_tx.seismic.recent_block_hash.hex()}")
print(f"Expires at: {result.shielded_tx.seismic.expires_at_block}")
print(f"Encryption nonce: {result.shielded_tx.seismic.encryption_nonce.hex()}")
print(f"Message version: {result.shielded_tx.seismic.message_version}")
print(f"Signed read: {result.shielded_tx.seismic.signed_read}")
```

### Comparing Gas Estimates

```python
# See actual gas used vs estimated
result = w3.seismic.debug_send_shielded_transaction(
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    data=HexBytes("0x..."),
)

print(f"Gas limit set: {result.shielded_tx.gas}")

# Wait for receipt to see actual gas used
receipt = w3.eth.wait_for_transaction_receipt(result.tx_hash)
print(f"Gas used: {receipt['gasUsed']}")
print(f"Difference: {result.shielded_tx.gas - receipt['gasUsed']}")
```

### With Contract Method

```python
# Use with contract.dwrite namespace
contract = w3.seismic.contract(
    address="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    abi=ABI,
)

# Call debug write on contract
result = contract.dwrite.transfer(
    "0x1234567890123456789012345678901234567890",
    1000,
)

print(f"Transaction hash: {result.tx_hash.hex()}")
print(f"Function call: {result.plaintext_tx.data.hex()[:10]}")  # Function selector
```

***

## Implementation Details

### Transaction is Broadcast

**Important:** This method **does broadcast** a real transaction to the network. It is not a dry-run or simulation.

The debug information is returned **after** the transaction is sent. Use this only when you want to actually execute the transaction.

### Encryption Process

The method performs the exact same encryption as `send_shielded_transaction()`:

1. Fetch security parameters (block hash, nonce, expiry)
2. Encrypt calldata using AES-GCM with ECDH-derived key
3. Construct `TxSeismic` transaction
4. Sign and broadcast

The only difference is that it captures and returns the intermediate states.

### Performance

Debug writes have the same performance as regular shielded writes. The only overhead is:
- Constructing the `PlaintextTx` and `UnsignedSeismicTx` objects
- Storing them in memory

This overhead is negligible for most use cases.

***

## Use Cases

### Development

Use during development to:
- Verify ABI encoding is correct
- Check encryption parameters match expectations
- Debug transaction construction issues
- Inspect nonces, gas prices, and other fields

### Testing

Use in tests to:
- Assert calldata is encoded correctly before encryption
- Verify security parameters are set as expected
- Check transaction structure without manual inspection

### Debugging Production Issues

Use temporarily in production to:
- Diagnose encryption-related issues
- Verify transactions are constructed correctly
- Compare plaintext vs encrypted data

***

## Security Considerations

### Plaintext Exposure

The `PlaintextTx` contains **unencrypted** calldata. Be careful when logging or displaying this information:

```python
# ⚠️ DON'T log plaintext in production
print(f"Plaintext: {result.plaintext_tx.data.hex()}")

# ✅ Log only transaction hash
print(f"Transaction: {result.tx_hash.hex()}")
```

### Production Use

In production:
- Use `send_shielded_transaction()` or `contract.write` for better performance
- Only use debug writes when actively debugging an issue
- Remove debug logging before deploying

***

## When to Use

### Use `debug_send_shielded_transaction()` When

- Developing new contract integrations
- Writing tests for shielded transactions
- Debugging encryption or encoding issues
- Verifying transaction parameters

### Use `send_shielded_transaction()` When

- In production code
- When you don't need debug information
- For better code clarity (less verbose return type)

### Use `contract.dwrite` When

- Working with contract methods
- Want automatic ABI encoding + debug info

***

## Notes

### Requires Wallet Client

This method is only available on wallet clients created with `create_wallet_client()`. It requires:
- A private key for signing
- Encryption state (derived from TEE public key)

### Return Type

The return type is `DebugWriteResult`, not `HexBytes`. Access the transaction hash via `result.tx_hash`.

### Contract Namespace

For contract interactions, prefer using `contract.dwrite.functionName(...)` instead of manually encoding calldata.

***

## See Also

- [send_shielded_transaction()](send-shielded-transaction.md) — Send without debug info
- [contract.dwrite](../contract/namespaces/dwrite.md) — Debug writes with ABI encoding
- [DebugWriteResult](../../api-reference/transaction-types/debug-write-result.md) — Return type reference
- [UnsignedSeismicTx](../../api-reference/transaction-types/unsigned-seismic-tx.md) — Transaction structure
- [Shielded Write Guide](../../guides/shielded-write.md) — Full encryption workflow
