---
description: AsyncSeismicNamespace class - Full wallet capabilities (async)
icon: wallet
---

# AsyncSeismicNamespace

The `AsyncSeismicNamespace` class provides full Seismic functionality for asynchronous wallet clients. It extends `AsyncSeismicPublicNamespace` with write capabilities that require a private key.

## Overview

This namespace is automatically attached as `w3.seismic` when you create an async wallet client with `create_async_wallet_client()`. It provides:

- Shielded transactions with encrypted calldata (async)
- Signed reads (eth_call with encryption, async)
- Debug transaction introspection (async)
- Validator deposits (async)
- Contract wrappers with full read/write capabilities
- All read-only methods from `AsyncSeismicPublicNamespace`

## Access

```python
from seismic_web3 import create_async_wallet_client, PrivateKey

w3 = await create_async_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=PrivateKey(...),
)

# Access the namespace (all methods are async)
await w3.seismic.send_shielded_transaction(...)
contract = w3.seismic.contract(address, abi)
```

## Public vs Wallet Namespaces

| Feature | Public Namespace | Wallet Namespace |
|---------|------------------|------------------|
| **Read-only queries** | ✅ Yes | ✅ Yes (inherited) |
| **Shielded writes** | ❌ No | ✅ Yes |
| **Signed reads** | ❌ No | ✅ Yes |
| **Validator deposits** | ❌ No | ✅ Yes |
| **Contract writes** | ❌ No | ✅ Yes |
| **Requires private key** | ❌ No | ✅ Yes |

The wallet namespace inherits all read-only capabilities from `AsyncSeismicPublicNamespace` and adds write operations that require cryptographic signing.

## Class Definition

```python
class AsyncSeismicNamespace(AsyncSeismicPublicNamespace):
    """Async Seismic namespace -- attached as ``w3.seismic``.

    Args:
        w3: Async AsyncWeb3 instance.
        encryption: Encryption state with ECDH-derived AES key.
        private_key: 32-byte signing key for transactions.

    Attributes:
        encryption: The EncryptionState holding the ECDH-derived
            AES key and keypair.
    """
```

## Methods

### Transaction Methods

#### send_shielded_transaction()

Send a shielded transaction with encrypted calldata (async).

```python
tx_hash = await w3.seismic.send_shielded_transaction(
    to=contract_address,
    data=encoded_calldata,
    value=0,
    gas=None,
    gas_price=None,
    security=None,
    eip712=False,
)
```

**Parameters:**
- `to` (ChecksumAddress): Recipient contract address
- `data` (HexBytes): Plaintext calldata (will be encrypted)
- `value` (int, optional): Wei to transfer (default: 0)
- `gas` (int | None, optional): Gas limit (auto-estimated if None)
- `gas_price` (int | None, optional): Gas price in wei
- `security` (SeismicSecurityParams | None, optional): Security parameter overrides
- `eip712` (bool, optional): Use EIP-712 typed data signing (default: False)

**Returns:** Transaction hash (HexBytes)

[See detailed documentation →](methods/send-shielded-transaction.md)

---

#### signed_call()

Execute a signed read (eth_call with encrypted calldata, async).

```python
result = await w3.seismic.signed_call(
    to=contract_address,
    data=encoded_calldata,
    value=0,
    gas=30_000_000,
    security=None,
    eip712=False,
)
```

**Parameters:**
- `to` (ChecksumAddress): Contract address to call
- `data` (HexBytes): Plaintext calldata (will be encrypted)
- `value` (int, optional): Wei to include (default: 0)
- `gas` (int, optional): Gas limit (default: 30,000,000)
- `security` (SeismicSecurityParams | None, optional): Security parameter overrides
- `eip712` (bool, optional): Use EIP-712 typed data signing (default: False)

**Returns:** Decrypted response bytes, or None if empty

[See detailed documentation →](methods/signed-call.md)

---

#### debug_send_shielded_transaction()

Send a shielded transaction and return debug information (async).

```python
debug_result = await w3.seismic.debug_send_shielded_transaction(
    to=contract_address,
    data=encoded_calldata,
    value=0,
    gas=None,
    gas_price=None,
    security=None,
    eip712=False,
)
```

**Parameters:** Same as `send_shielded_transaction()`

**Returns:** `DebugWriteResult` containing:
- Transaction hash
- Plaintext transaction view
- Encrypted transaction view

[See detailed documentation →](methods/debug-send-shielded-transaction.md)

---

### Contract Factory

#### contract()

Create an `AsyncShieldedContract` wrapper with full read/write capabilities.

```python
contract = w3.seismic.contract(
    address=contract_address,
    abi=contract_abi,
    eip712=False,
)
```

**Parameters:**
- `address` (ChecksumAddress): Contract address
- `abi` (list[dict[str, Any]]): Contract ABI (list of function entries)
- `eip712` (bool, optional): Use EIP-712 typed data signing (default: False)

**Returns:** `AsyncShieldedContract` instance with namespaces:
- `.write` - Shielded writes (encrypted, async)
- `.read` - Signed reads (encrypted, async)
- `.twrite` - Transparent writes (unencrypted, async)
- `.tread` - Transparent reads (unencrypted, async)
- `.dwrite` - Debug writes (returns debug info, async)

[See contract documentation →](../contract/)

---

### Query Methods

These methods are inherited from `AsyncSeismicPublicNamespace`:

#### get_tee_public_key()

Fetch the TEE's compressed secp256k1 public key (async).

```python
tee_key = await w3.seismic.get_tee_public_key()
```

**Returns:** 33-byte compressed public key

[See detailed documentation →](methods/get-tee-public-key.md)

---

#### get_deposit_root()

Read the current deposit Merkle root (async).

```python
root = await w3.seismic.get_deposit_root(
    address="0x4242424242424242424242424242424242424242",
)
```

**Parameters:**
- `address` (str, optional): Deposit contract address (defaults to genesis contract)

**Returns:** 32-byte deposit root hash

[See detailed documentation →](methods/get-deposit-root.md)

---

#### get_deposit_count()

Read the current deposit count (async).

```python
count = await w3.seismic.get_deposit_count(
    address="0x4242424242424242424242424242424242424242",
)
```

**Parameters:**
- `address` (str, optional): Deposit contract address (defaults to genesis contract)

**Returns:** Number of deposits as a Python int

[See detailed documentation →](methods/get-deposit-count.md)

---

### Deposit Methods

#### deposit()

Submit a validator deposit to the deposit contract (async).

```python
tx_hash = await w3.seismic.deposit(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    deposit_data_root=deposit_data_root,
    value=32 * 10**18,  # 32 ETH in wei
    address="0x4242424242424242424242424242424242424242",
)
```

**Parameters:** (all keyword-only)
- `node_pubkey` (bytes): 32-byte ED25519 public key
- `consensus_pubkey` (bytes): 48-byte BLS12-381 public key
- `withdrawal_credentials` (bytes): 32-byte withdrawal credentials
- `node_signature` (bytes): 64-byte ED25519 signature
- `consensus_signature` (bytes): 96-byte BLS12-381 signature
- `deposit_data_root` (bytes): 32-byte deposit data root hash
- `value` (int): Deposit amount in wei (e.g., 32 * 10**18)
- `address` (str, optional): Deposit contract address (defaults to genesis contract)

**Returns:** Transaction hash

**Raises:** ValueError if any argument has the wrong byte length

[See detailed documentation →](methods/deposit.md)

---

## Usage Examples

### Basic Shielded Transaction

```python
import asyncio
from seismic_web3 import create_async_wallet_client, PrivateKey
from hexbytes import HexBytes

async def main():
    # Create async wallet client
    w3 = await create_async_wallet_client(
        "https://gcp-1.seismictest.net/rpc",
        private_key=PrivateKey(b"...32 bytes..."),
    )

    # Send shielded transaction
    tx_hash = await w3.seismic.send_shielded_transaction(
        to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        data=HexBytes("0x12345678..."),
        value=1000000000000000000,  # 1 ETH
    )

    print(f"Transaction sent: {tx_hash.hex()}")

asyncio.run(main())
```

### Signed Read

```python
async def signed_read_example():
    w3 = await create_async_wallet_client(
        "https://gcp-1.seismictest.net/rpc",
        private_key=PrivateKey(b"...32 bytes..."),
    )

    # Perform signed read (encrypted eth_call)
    result = await w3.seismic.signed_call(
        to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        data=HexBytes("0x12345678..."),
    )

    if result:
        print(f"Response: {result.hex()}")
```

### Contract Interaction

```python
async def contract_example():
    w3 = await create_async_wallet_client(
        "https://gcp-1.seismictest.net/rpc",
        private_key=PrivateKey(b"...32 bytes..."),
    )

    # Create contract wrapper
    contract = w3.seismic.contract(
        address="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        abi=[
            {
                "name": "transfer",
                "type": "function",
                "inputs": [
                    {"name": "to", "type": "address"},
                    {"name": "amount", "type": "uint256"},
                ],
                "outputs": [{"name": "", "type": "bool"}],
            },
        ],
    )

    # Execute shielded write
    tx_hash = await contract.write.transfer(
        "0x1234567890123456789012345678901234567890",
        1000,
    )

    # Perform signed read
    balance = await contract.read.balanceOf(
        "0x1234567890123456789012345678901234567890",
    )
```

### Concurrent Operations

```python
async def concurrent_example():
    w3 = await create_async_wallet_client(
        "https://gcp-1.seismictest.net/rpc",
        private_key=PrivateKey(b"...32 bytes..."),
    )

    # Execute multiple operations concurrently
    results = await asyncio.gather(
        w3.seismic.get_tee_public_key(),
        w3.seismic.get_deposit_count(),
        w3.seismic.get_deposit_root(),
    )

    tee_key, deposit_count, deposit_root = results
    print(f"TEE key: {tee_key.hex()}")
    print(f"Deposits: {deposit_count}")
    print(f"Root: {deposit_root.hex()}")
```

### Debug Transaction

```python
async def debug_example():
    w3 = await create_async_wallet_client(
        "https://gcp-1.seismictest.net/rpc",
        private_key=PrivateKey(b"...32 bytes..."),
    )

    # Send transaction with debug info
    debug_result = await w3.seismic.debug_send_shielded_transaction(
        to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        data=HexBytes("0x12345678..."),
    )

    print(f"Transaction hash: {debug_result.tx_hash.hex()}")
    print(f"Plaintext calldata: {debug_result.plaintext_tx}")
    print(f"Encrypted calldata: {debug_result.encrypted_tx}")
```

### Context Manager Pattern

```python
async def context_manager_example():
    async with await create_async_wallet_client(
        "https://gcp-1.seismictest.net/rpc",
        private_key=PrivateKey(b"...32 bytes..."),
    ) as w3:
        # Use the client
        tx_hash = await w3.seismic.send_shielded_transaction(
            to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            data=HexBytes("0x12345678..."),
        )
        print(f"Transaction: {tx_hash.hex()}")
    # Client automatically cleaned up
```

## Async Best Practices

### Use asyncio.gather() for Parallel Operations

```python
# Good: Execute operations in parallel
tee_key, count = await asyncio.gather(
    w3.seismic.get_tee_public_key(),
    w3.seismic.get_deposit_count(),
)

# Bad: Execute operations sequentially
tee_key = await w3.seismic.get_tee_public_key()
count = await w3.seismic.get_deposit_count()
```

### Handle Connection Cleanup

```python
# Use context manager or explicitly close connections
async with await create_async_wallet_client(...) as w3:
    # Perform operations
    pass
# Connection automatically closed
```

### Error Handling with Async

```python
try:
    tx_hash = await w3.seismic.send_shielded_transaction(
        to=address,
        data=data,
    )
except Exception as e:
    print(f"Transaction failed: {e}")
```

## See Also

- [SeismicNamespace](seismic-namespace.md) - Sync version of this namespace
- [AsyncSeismicPublicNamespace](async-seismic-public-namespace.md) - Read-only async namespace
- [Contract Documentation](../contract/) - Contract interaction patterns
- [Shielded Write Guide](../guides/shielded-write.md) - Step-by-step transaction guide
- [Signed Reads Guide](../guides/signed-reads.md) - Signed read patterns
- [Client Documentation](../client/) - Client creation and configuration
