---
description: Sync contract wrapper with shielded and transparent namespaces
icon: file-lock
---

# ShieldedContract

Synchronous contract wrapper providing encrypted and transparent interaction with Seismic contracts.

## Overview

`ShieldedContract` is the primary sync interface for interacting with Seismic smart contracts. It provides five namespaces for different interaction modes: encrypted writes ([`.write`](namespaces/write.md)), encrypted reads ([`.read`](namespaces/read.md)), transparent writes ([`.twrite`](namespaces/twrite.md)), transparent reads ([`.tread`](namespaces/tread.md)), and debug writes ([`.dwrite`](namespaces/dwrite.md)). Each namespace dynamically exposes contract methods based on the provided ABI.

## Definition

```python
class ShieldedContract:
    def __init__(
        self,
        w3: Web3,
        encryption: EncryptionState,
        private_key: PrivateKey,
        address: ChecksumAddress,
        abi: list[dict[str, Any]],
        eip712: bool = False,
    ) -> None:
        ...
```

## Constructor Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `w3` | `Web3` | Yes | Synchronous Web3 instance connected to Seismic RPC |
| `encryption` | [`EncryptionState`](../client/encryption-state.md) | Yes | Encryption state for shielded operations |
| `private_key` | [`PrivateKey`](../api-reference/types/private-key.md) | Yes | 32-byte secp256k1 private key for signing transactions |
| `address` | `ChecksumAddress` | Yes | Contract address (checksummed Ethereum address) |
| `abi` | `list[dict[str, Any]]` | Yes | Contract ABI (list of function entries) |
| `eip712` | `bool` | No | Use EIP-712 typed data signing (default: `False`) |

## Namespaces

### `.write` - Encrypted Write

Sends encrypted transactions using `TxSeismic` (type `0x4a`). Calldata is encrypted before broadcast.

**Returns**: `HexBytes` (transaction hash)

**Positional Arguments**: `*args` - ABI function arguments (e.g. `contract.write.transfer(to, amount)`)

**Optional Parameters**:
- `value: int` - Wei to send (default: `0`)
- `gas: int | None` - Gas limit (default: `30_000_000` when omitted)
- `gas_price: int | None` - Gas price in wei (default: network suggested)
- `security: [`SeismicSecurityParams`](../api-reference/transaction-types/seismic-security-params.md) | None` - Security parameters for expiry

### `.read` - Encrypted Read

Executes encrypted signed `eth_call` with encrypted calldata. Result is decrypted and ABI-decoded by the SDK. Single-output functions return the value directly (e.g. `int`, `bool`); multi-output functions return a `tuple`.

**Returns**: `Any` (ABI-decoded Python value)

**Positional Arguments**: `*args` - ABI function arguments (e.g. `contract.read.balanceOf(owner)`)

**Optional Parameters**:
- `value: int` - Wei for call context (default: `0`)
- `gas: int` - Gas limit (default: `30_000_000`)
- `security: [`SeismicSecurityParams`](../api-reference/transaction-types/seismic-security-params.md) | None` - Security parameters for expiry

### `.twrite` - Transparent Write

Sends standard `eth_sendTransaction` with unencrypted calldata.

**Returns**: `HexBytes` (transaction hash)

**Positional Arguments**: `*args` - ABI function arguments

**Optional Parameters**:
- `value: int` - Wei to send (default: `0`)
- `**tx_params: Any` - Additional transaction parameters (gas, gasPrice, etc.)

### `.tread` - Transparent Read

Executes standard `eth_call` with unencrypted calldata. Result is ABI-decoded by the SDK. Single-output functions return the value directly; multi-output functions return a `tuple`.

**Returns**: `Any` (ABI-decoded Python value)

**Positional Arguments**: `*args` - ABI function arguments

### `.dwrite` - Debug Write

Like `.write` but returns debug information including plaintext and encrypted views. **Transaction is actually broadcast**.

**Returns**: [`DebugWriteResult`](../api-reference/transaction-types/debug-write-result.md)

**Positional Arguments**: `*args` - ABI function arguments

**Optional Parameters**: Same as `.write`

## Examples

### Basic Encrypted Write

```python
from seismic_web3 import create_wallet_client, ShieldedContract

w3 = create_wallet_client(
    rpc_url="https://gcp-1.seismictest.net/rpc",
    private_key=private_key,
)

contract = ShieldedContract(
    w3=w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    address="0x1234567890123456789012345678901234567890",
    abi=CONTRACT_ABI,
)

# Encrypted write - calldata hidden on-chain
tx_hash = contract.write.setNumber(42)
print(f"Transaction: {tx_hash.to_0x_hex()}")

# Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
print(f"Status: {receipt['status']}")
```

### Encrypted Read

```python
# Encrypted read — calldata and result hidden, auto-decoded
number = contract.read.getNumber()  # int
print(f"Number: {number}")
```

### Transparent Operations

```python
# Transparent write - calldata visible on-chain
tx_hash = contract.twrite.setNumber(42)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Transparent read — standard eth_call, auto-decoded
number = contract.tread.getNumber()
print(f"Result: {number}")
```

### Debug Write

```python
# Debug write - returns plaintext and encrypted views
debug_result = contract.dwrite.transfer("0xRecipient...", 1000)

print(f"Transaction hash: {debug_result.tx_hash.to_0x_hex()}")
print(f"Plaintext data: {debug_result.plaintext_tx.data.to_0x_hex()}")
print(f"Encrypted data: {debug_result.shielded_tx.data.to_0x_hex()}")

# Transaction is actually broadcast
receipt = w3.eth.wait_for_transaction_receipt(debug_result.tx_hash)
```

### With Transaction Parameters

```python
# Custom gas and value
tx_hash = contract.write.deposit(
    value=10**18,  # 1 ETH
    gas=200_000,
    gas_price=20 * 10**9,  # 20 gwei
)

# With security parameters
from seismic_web3.transaction_types import SeismicSecurityParams

security = SeismicSecurityParams(blocks_window=100)
tx_hash = contract.write.withdraw(
    amount,
    security=security,
)
```

### Using EIP-712 Signing

```python
# Enable EIP-712 for typed data signing
contract = ShieldedContract(
    w3=w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
    address=contract_address,
    abi=CONTRACT_ABI,
    eip712=True,  # Use EIP-712 instead of raw signing
)

tx_hash = contract.write.setNumber(123)
```

### Instantiation via Client

```python
# Most common pattern - let the client create the contract
from seismic_web3 import create_wallet_client

w3 = create_wallet_client(
    rpc_url="https://gcp-1.seismictest.net/rpc",
    private_key=private_key,
)

# Client's contract() method creates ShieldedContract
contract = w3.seismic.contract(address=contract_address, abi=CONTRACT_ABI)

# Now use any namespace
tx_hash = contract.write.setNumber(42)
```

## Notes

- **Dynamic method access**: Contract methods are accessed via `__getattr__`, so `contract.write.setNumber()` dynamically resolves to the ABI function
- **ABI remapping**: Shielded types (`suint256`, `sbool`, `saddress`) are remapped to standard types for encoding while preserving original names for selector computation
- **Gas defaults**: `.write` uses `30_000_000` when `gas` is omitted; `.twrite` follows normal `web3.py`/provider transaction behavior
- **Encryption overhead**: Encrypted operations add ~16 bytes (AES-GCM auth tag) to calldata
- **EIP-712 vs raw**: EIP-712 signing enables integration with browser extension wallets like Metamask; raw signing is faster for automation and likely what you want to use in this Python SDK
- **Use `.write` in production**: `.dwrite` is for debugging only

## See Also

- [AsyncShieldedContract](async-shielded-contract.md) - Async version of this class
- [PublicContract](public-contract.md) - Read-only contract wrapper (no encryption)
- [EncryptionState](../client/encryption-state.md) - Manages encryption keys
- [DebugWriteResult](../api-reference/transaction-types/debug-write-result.md) - Debug write return type
- [SeismicSecurityParams](../api-reference/transaction-types/seismic-security-params.md) - Transaction expiry parameters
- [Contract Namespaces](namespaces/) - Detailed namespace documentation
- [Shielded Write Guide](../guides/shielded-write.md) - Complete workflow guide
