---
description: Async contract wrapper with shielded and transparent namespaces
icon: file-lock
---

# AsyncShieldedContract

Asynchronous contract wrapper providing encrypted and transparent interaction with Seismic contracts.

## Overview

`AsyncShieldedContract` is the async version of `ShieldedContract`, providing the same five namespaces (`.write`, `.read`, `.twrite`, `.tread`, `.dwrite`) but with coroutine-based methods. All namespace methods return coroutines that must be awaited. Use this class in async/await applications for non-blocking contract interactions.

## Definition

```python
class AsyncShieldedContract:
    def __init__(
        self,
        w3: AsyncWeb3,
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
| `w3` | `AsyncWeb3` | Yes | Asynchronous AsyncWeb3 instance connected to Seismic RPC |
| `encryption` | [`EncryptionState`](../client/encryption-state.md) | Yes | Encryption state for shielded operations |
| `private_key` | [`PrivateKey`](../api-reference/types/private-key.md) | Yes | 32-byte secp256k1 private key for signing transactions |
| `address` | `ChecksumAddress` | Yes | Contract address (checksummed Ethereum address) |
| `abi` | `list[dict[str, Any]]` | Yes | Contract ABI (list of function entries) |
| `eip712` | `bool` | No | Use EIP-712 typed data signing (default: `False`) |

## Namespaces

### `.write` - Encrypted Write

Sends encrypted transactions using `TxSeismic` (type `0x4a`). Calldata is encrypted before broadcast.

**Returns**: `Coroutine[HexBytes]` (transaction hash)

**Optional Parameters**:
- `value: int` - Wei to send (default: `0`)
- `gas: int | None` - Gas limit (default: `30_000_000` when omitted)
- `gas_price: int | None` - Gas price in wei (default: network suggested)
- `security: [`SeismicSecurityParams`](../api-reference/transaction-types/seismic-security-params.md) | None` - Security parameters for expiry

### `.read` - Encrypted Read

Executes encrypted signed `eth_call` with encrypted calldata. Result is decrypted and ABI-decoded by the SDK. Single-output functions return the value directly (e.g. `int`, `bool`); multi-output functions return a `tuple`.

**Returns**: `Coroutine[Any]` (ABI-decoded Python value)

**Optional Parameters**:
- `value: int` - Wei for call context (default: `0`)
- `gas: int` - Gas limit (default: `30_000_000`)
- `security: [`SeismicSecurityParams`](../api-reference/transaction-types/seismic-security-params.md) | None` - Security parameters for expiry

### `.twrite` - Transparent Write

Sends standard async `eth_sendTransaction` with unencrypted calldata.

**Returns**: `Coroutine[HexBytes]` (transaction hash)

**Optional Parameters**:
- `value: int` - Wei to send (default: `0`)
- `**tx_params: Any` - Additional transaction parameters (gas, gasPrice, etc.)

### `.tread` - Transparent Read

Executes standard async `eth_call` with unencrypted calldata. Result is ABI-decoded by the SDK. Single-output functions return the value directly; multi-output functions return a `tuple`.

**Returns**: `Coroutine[Any]` (ABI-decoded Python value)

**Optional Parameters**: None (pass positional arguments only)

### `.dwrite` - Debug Write

Like `.write` but returns debug information including plaintext and encrypted views. **Transaction is actually broadcast**.

**Returns**: `Coroutine[DebugWriteResult]` ([`DebugWriteResult`](../api-reference/transaction-types/debug-write-result.md))

**Optional Parameters**: Same as `.write`

## Examples

### Basic Encrypted Write

```python
import asyncio
from seismic_web3 import create_async_wallet_client, AsyncShieldedContract

async def main():
    w3 = await create_async_wallet_client(
        provider_url="https://gcp-1.seismictest.net/rpc",
        private_key=private_key,
    )

    contract = AsyncShieldedContract(
        w3=w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
        address="0x1234567890123456789012345678901234567890",
        abi=CONTRACT_ABI,
    )

    # Encrypted write - must await
    tx_hash = await contract.write.setNumber(42)
    print(f"Transaction: {tx_hash.to_0x_hex()}")

    # Wait for confirmation
    receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
    print(f"Status: {receipt['status']}")

asyncio.run(main())
```

### Encrypted Read

```python
async def read_example(contract: AsyncShieldedContract):
    # Encrypted read — auto-decoded, must await
    number = await contract.read.getNumber()  # int
    print(f"Number: {number}")
```

### Transparent Operations

```python
async def transparent_example(contract: AsyncShieldedContract, w3: AsyncWeb3):
    # Transparent write - calldata visible on-chain
    tx_hash = await contract.twrite.setNumber(42)
    receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
    print(f"Status: {receipt['status']}")

    # Transparent read — standard eth_call, auto-decoded
    number = await contract.tread.getNumber()
    print(f"Result: {number}")
```

### Debug Write

```python
async def debug_example(contract: AsyncShieldedContract, w3: AsyncWeb3):
    # Debug write - returns plaintext and encrypted views
    debug_result = await contract.dwrite.transfer("0xRecipient...", 1000)

    print(f"Transaction hash: {debug_result.tx_hash.to_0x_hex()}")
    print(f"Plaintext data: {debug_result.plaintext_tx.data.to_0x_hex()}")
    print(f"Encrypted data: {debug_result.shielded_tx.data.to_0x_hex()}")

    # Transaction is actually broadcast
    receipt = await w3.eth.wait_for_transaction_receipt(debug_result.tx_hash)
    print(f"Confirmed in block: {receipt['blockNumber']}")
```

### Concurrent Operations

```python
async def concurrent_example(contract: AsyncShieldedContract):
    # Execute multiple reads concurrently — each is auto-decoded
    balances = await asyncio.gather(
        contract.tread.balanceOf("0xAddress1..."),
        contract.tread.balanceOf("0xAddress2..."),
        contract.tread.balanceOf("0xAddress3..."),
    )

    for i, balance in enumerate(balances):
        print(f"Balance {i}: {balance}")
```

### With Transaction Parameters

```python
async def advanced_write(contract: AsyncShieldedContract):
    # Custom gas and value
    tx_hash = await contract.write.deposit(
        value=10**18,  # 1 ETH
        gas=200_000,
        gas_price=20 * 10**9,  # 20 gwei
    )

    # With security parameters
    from seismic_web3.transaction_types import SeismicSecurityParams

    security = SeismicSecurityParams(blocks_window=100)
    tx_hash = await contract.write.withdraw(
        amount,
        security=security,
    )
```

### Batch Processing

```python
async def batch_writes(contract: AsyncShieldedContract, recipients: list[str]):
    # Send multiple transactions concurrently
    tx_hashes = await asyncio.gather(
        *[contract.write.transfer(recipient, 100) for recipient in recipients]
    )

    print(f"Sent {len(tx_hashes)} transactions")

    # Wait for all confirmations
    receipts = await asyncio.gather(
        *[w3.eth.wait_for_transaction_receipt(tx_hash) for tx_hash in tx_hashes]
    )

    successful = sum(1 for r in receipts if r['status'] == 1)
    print(f"{successful}/{len(receipts)} successful")
```

### Using EIP-712 Signing

```python
async def eip712_example():
    w3 = await create_async_wallet_client(...)

    # Enable EIP-712 for typed data signing
    contract = AsyncShieldedContract(
        w3=w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
        address=contract_address,
        abi=CONTRACT_ABI,
        eip712=True,  # Use EIP-712 instead of raw signing
    )

    tx_hash = await contract.write.setNumber(123)
```

### Instantiation via Async Client

```python
async def client_pattern():
    # Most common pattern - let the client create the contract
    w3 = await create_async_wallet_client(
        provider_url="https://gcp-1.seismictest.net/rpc",
        private_key=private_key,
    )

    # Client's contract() method creates AsyncShieldedContract
    contract = w3.seismic.contract(address=contract_address, abi=CONTRACT_ABI)

    # Now use any namespace (must await)
    tx_hash = await contract.write.setNumber(42)
```

### Error Handling

```python
async def error_handling(contract: AsyncShieldedContract):
    try:
        tx_hash = await contract.write.withdraw(123)
        receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)

        if receipt['status'] != 1:
            print("Transaction failed on-chain")
    except ValueError as e:
        print(f"RPC error: {e}")
    except Exception as e:
        print(f"Unexpected error: {e}")
```

### Context Manager Pattern

```python
async def context_pattern():
    async with create_async_wallet_client(...) as w3:
        contract = AsyncShieldedContract(...)

        tx_hash = await contract.write.setNumber(42)
        receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
    # Connection automatically closed
```

## Notes

- **All methods return coroutines**: Must use `await` with every namespace call
- **Concurrent operations**: Use `asyncio.gather()` for parallel reads/writes
- **Dynamic method access**: Contract methods resolved via `__getattr__` at runtime
- **ABI remapping**: Shielded types automatically remapped (same as sync version)
- **Connection pooling**: AsyncWeb3 can reuse connections for better performance
- **Gas defaults**: `.write` uses `30_000_000` when `gas` is omitted; `.twrite` follows normal `web3.py`/provider transaction behavior
- **EIP-712 vs raw**: Same signing options as sync version
- **Error handling**: Use try/except around await calls for RPC errors

## See Also

- [ShieldedContract](shielded-contract.md) - Synchronous version of this class
- [AsyncPublicContract](async-public-contract.md) - Async read-only contract wrapper
- [create_async_wallet_client](../client/create-async-wallet-client.md) - Create async client
- [EncryptionState](../client/encryption-state.md) - Manages encryption keys
- [DebugWriteResult](../api-reference/transaction-types/debug-write-result.md) - Debug write return type
- [SeismicSecurityParams](../api-reference/transaction-types/seismic-security-params.md) - Transaction expiry parameters
- [Contract Namespaces](namespaces/) - Detailed namespace documentation
