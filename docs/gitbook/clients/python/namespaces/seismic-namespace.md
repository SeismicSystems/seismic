---
description: SeismicNamespace class - Full wallet capabilities (sync)
icon: wallet
---

# SeismicNamespace

The `SeismicNamespace` class provides full Seismic functionality for synchronous wallet clients. It extends `SeismicPublicNamespace` with write capabilities that require a private key.

## Overview

This namespace is automatically attached as `w3.seismic` when you create a wallet client with `create_wallet_client()`. It provides:

- Shielded transactions with encrypted calldata
- Signed reads (eth_call with encryption)
- Debug transaction introspection
- Validator deposits
- Contract wrappers with full read/write capabilities
- All read-only methods from `SeismicPublicNamespace`

## Access

```python
from seismic_web3 import create_wallet_client, PrivateKey

w3 = create_wallet_client(
    "https://rpc.example.com",
    private_key=PrivateKey(...),
)

# Access the namespace
w3.seismic.send_shielded_transaction(...)
w3.seismic.contract(address, abi)
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

The wallet namespace inherits all read-only capabilities from `SeismicPublicNamespace` and adds write operations that require cryptographic signing.

## Class Definition

```python
class SeismicNamespace(SeismicPublicNamespace):
    """Sync Seismic namespace -- attached as ``w3.seismic``.

    Args:
        w3: Sync Web3 instance.
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

Send a shielded transaction with encrypted calldata.

```python
tx_hash = w3.seismic.send_shielded_transaction(
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

Execute a signed read (eth_call with encrypted calldata).

```python
result = w3.seismic.signed_call(
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

Send a shielded transaction and return debug information.

```python
debug_result = w3.seismic.debug_send_shielded_transaction(
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

Create a `ShieldedContract` wrapper with full read/write capabilities.

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

**Returns:** `ShieldedContract` instance with namespaces:
- `.write` - Shielded writes (encrypted)
- `.read` - Signed reads (encrypted)
- `.twrite` - Transparent writes (unencrypted)
- `.tread` - Transparent reads (unencrypted)
- `.dwrite` - Debug writes (returns debug info)

[See contract documentation →](../contract/)

---

### Query Methods

These methods are inherited from `SeismicPublicNamespace`:

#### get_tee_public_key()

Fetch the TEE's compressed secp256k1 public key.

```python
tee_key = w3.seismic.get_tee_public_key()
```

**Returns:** 33-byte compressed public key

[See detailed documentation →](methods/get-tee-public-key.md)

---

#### get_deposit_root()

Read the current deposit Merkle root.

```python
root = w3.seismic.get_deposit_root(
    address="0x4242424242424242424242424242424242424242",
)
```

**Parameters:**
- `address` (str, optional): Deposit contract address (defaults to genesis contract)

**Returns:** 32-byte deposit root hash

[See detailed documentation →](methods/get-deposit-root.md)

---

#### get_deposit_count()

Read the current deposit count.

```python
count = w3.seismic.get_deposit_count(
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

Submit a validator deposit to the deposit contract.

```python
tx_hash = w3.seismic.deposit(
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
from seismic_web3 import create_wallet_client, PrivateKey
from hexbytes import HexBytes

# Create wallet client
w3 = create_wallet_client(
    "https://rpc.example.com",
    private_key=PrivateKey(b"...32 bytes..."),
)

# Send shielded transaction
tx_hash = w3.seismic.send_shielded_transaction(
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
    data=HexBytes("0x12345678..."),
    value=1000000000000000000,  # 1 ETH
)

print(f"Transaction sent: {tx_hash.hex()}")
```

### Signed Read

```python
# Perform signed read (encrypted eth_call)
result = w3.seismic.signed_call(
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
    data=HexBytes("0x12345678..."),
)

if result:
    print(f"Response: {result.hex()}")
```

### Contract Interaction

```python
# Create contract wrapper
contract = w3.seismic.contract(
    address="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
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
tx_hash = contract.write.transfer(
    "0x1234567890123456789012345678901234567890",
    1000,
)

# Perform signed read
balance = contract.read.balanceOf(
    "0x1234567890123456789012345678901234567890",
)
```

### Debug Transaction

```python
# Send transaction with debug info
debug_result = w3.seismic.debug_send_shielded_transaction(
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
    data=HexBytes("0x12345678..."),
)

print(f"Transaction hash: {debug_result.tx_hash.hex()}")
print(f"Plaintext calldata: {debug_result.plaintext_tx}")
print(f"Encrypted calldata: {debug_result.encrypted_tx}")
```

### Query TEE and Deposits

```python
# Get TEE public key
tee_key = w3.seismic.get_tee_public_key()
print(f"TEE public key: {tee_key.hex()}")

# Query deposit information
deposit_count = w3.seismic.get_deposit_count()
deposit_root = w3.seismic.get_deposit_root()
print(f"Total deposits: {deposit_count}")
print(f"Deposit root: {deposit_root.hex()}")
```

## See Also

- [AsyncSeismicNamespace](async-seismic-namespace.md) - Async version of this namespace
- [SeismicPublicNamespace](seismic-public-namespace.md) - Read-only public namespace
- [Contract Documentation](../contract/) - Contract interaction patterns
- [Shielded Write Guide](../guides/shielded-write.md) - Step-by-step transaction guide
- [Signed Reads Guide](../guides/signed-reads.md) - Signed read patterns
- [Client Documentation](../client/) - Client creation and configuration
