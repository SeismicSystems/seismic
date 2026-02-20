---
description: SeismicPublicNamespace class - Read-only public operations (sync)
icon: book-open
---

# SeismicPublicNamespace

The `SeismicPublicNamespace` class provides read-only Seismic functionality for synchronous public clients. It does not require a private key and is designed for querying blockchain state without performing transactions.

## Overview

This namespace is automatically attached as `w3.seismic` when you create a public client with `create_public_client()`. It provides:

- TEE public key retrieval
- Deposit contract queries (root, count)
- Public contract wrappers with transparent read capabilities
- No transaction signing or encryption capabilities

## Access

```python
from seismic_web3 import create_public_client

public = create_public_client("https://rpc.example.com")

# Access the namespace
public.seismic.get_tee_public_key()
public.seismic.contract(address, abi)
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

The public namespace is ideal for:
- Monitoring blockchain state
- Reading public contract data
- Querying deposit information
- Applications that don't need to send transactions

For write operations, use `SeismicNamespace` with a wallet client.

## Class Definition

```python
class SeismicPublicNamespace:
    """Sync public Seismic namespace -- attached as ``w3.seismic``.

    Provides read-only convenience methods that do not require
    a private key or encryption state.

    Args:
        w3: Sync Web3 instance.
    """
```

## Methods

### Query Methods

#### get_tee_public_key()

Fetch the TEE's compressed secp256k1 public key.

```python
tee_key = public.seismic.get_tee_public_key()
```

**Returns:** 33-byte compressed public key (bytes)

**Use case:** Required for client-side encryption before sending shielded transactions (though this namespace cannot send transactions itself).

[See detailed documentation →](methods/get-tee-public-key.md)

---

#### get_deposit_root()

Read the current deposit Merkle root from the deposit contract.

```python
root = public.seismic.get_deposit_root(
    address="0x4242424242424242424242424242424242424242",
)
```

**Parameters:**
- `address` (str, optional): Deposit contract address (defaults to genesis contract at `0x4242424242424242424242424242424242424242`)

**Returns:** 32-byte deposit root hash

**Use case:** Verify deposit inclusion or monitor deposit tree state.

[See detailed documentation →](methods/get-deposit-root.md)

---

#### get_deposit_count()

Read the current deposit count from the deposit contract.

```python
count = public.seismic.get_deposit_count(
    address="0x4242424242424242424242424242424242424242",
)
```

**Parameters:**
- `address` (str, optional): Deposit contract address (defaults to genesis contract at `0x4242424242424242424242424242424242424242`)

**Returns:** Number of deposits as a Python int

**Use case:** Monitor validator deposits or calculate deposit tree depth.

[See detailed documentation →](methods/get-deposit-count.md)

---

### Contract Factory

#### contract()

Create a `PublicContract` wrapper with transparent read-only capabilities.

```python
contract = public.seismic.contract(
    address=contract_address,
    abi=contract_abi,
)
```

**Parameters:**
- `address` (ChecksumAddress): Contract address
- `abi` (list[dict[str, Any]]): Contract ABI (list of function entries)

**Returns:** `PublicContract` instance with namespace:
- `.tread` - Transparent reads (unencrypted, read-only)

**Note:** This returns a `PublicContract`, not a `ShieldedContract`. It only supports transparent reads via the `.tread` namespace. For full contract capabilities including writes and encrypted operations, use a wallet client with `SeismicNamespace`.

[See contract documentation →](../contract/)

---

## Usage Examples

### Basic Query Operations

```python
from seismic_web3 import create_public_client

# Create public client (no private key required)
public = create_public_client("https://rpc.example.com")

# Get TEE public key
tee_key = public.seismic.get_tee_public_key()
print(f"TEE public key: {tee_key.hex()}")

# Query deposit information
deposit_count = public.seismic.get_deposit_count()
deposit_root = public.seismic.get_deposit_root()
print(f"Total deposits: {deposit_count}")
print(f"Deposit root: {deposit_root.hex()}")
```

### Reading Contract Data

```python
# Create contract wrapper
contract = public.seismic.contract(
    address="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
    abi=[
        {
            "name": "balanceOf",
            "type": "function",
            "inputs": [{"name": "account", "type": "address"}],
            "outputs": [{"name": "", "type": "uint256"}],
            "stateMutability": "view",
        },
        {
            "name": "totalSupply",
            "type": "function",
            "inputs": [],
            "outputs": [{"name": "", "type": "uint256"}],
            "stateMutability": "view",
        },
    ],
)

# Read contract state (transparent, unencrypted)
balance = contract.tread.balanceOf(
    "0x1234567890123456789012345678901234567890",
)
total_supply = contract.tread.totalSupply()

print(f"Balance: {balance}")
print(f"Total supply: {total_supply}")
```

### Monitoring Deposits

```python
import time

# Monitor deposit activity
last_count = 0

while True:
    current_count = public.seismic.get_deposit_count()

    if current_count > last_count:
        print(f"New deposits detected: {current_count - last_count}")
        print(f"Total deposits: {current_count}")

        # Get updated root
        root = public.seismic.get_deposit_root()
        print(f"Current root: {root.hex()}")

        last_count = current_count

    time.sleep(12)  # Check every block (~12 seconds)
```

### Multiple Clients Pattern

```python
from seismic_web3 import create_public_client, create_wallet_client, PrivateKey

# Use public client for queries
public = create_public_client("https://rpc.example.com")

# Use wallet client for transactions
wallet = create_wallet_client(
    "https://rpc.example.com",
    private_key=PrivateKey(b"...32 bytes..."),
)

# Query with public client (read-only)
tee_key = public.seismic.get_tee_public_key()
deposit_count = public.seismic.get_deposit_count()

# Transact with wallet client (requires private key)
tx_hash = wallet.seismic.send_shielded_transaction(
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0",
    data=encoded_data,
)

print(f"Query results: {deposit_count} deposits")
print(f"Transaction sent: {tx_hash.hex()}")
```

### Custom Deposit Contract

```python
# Query custom deposit contract (non-default address)
custom_deposit_address = "0x1234567890123456789012345678901234567890"

count = public.seismic.get_deposit_count(
    address=custom_deposit_address,
)
root = public.seismic.get_deposit_root(
    address=custom_deposit_address,
)

print(f"Custom contract deposits: {count}")
print(f"Custom contract root: {root.hex()}")
```

### Data Aggregation

```python
def get_chain_summary(rpc_url: str) -> dict:
    """Aggregate public chain data."""
    public = create_public_client(rpc_url)

    return {
        "tee_public_key": public.seismic.get_tee_public_key().hex(),
        "deposit_count": public.seismic.get_deposit_count(),
        "deposit_root": public.seismic.get_deposit_root().hex(),
        "block_number": public.eth.block_number,
        "chain_id": public.eth.chain_id,
    }

summary = get_chain_summary("https://rpc.example.com")
print(summary)
```

## When to Use Public vs Wallet Client

### Use Public Client When:

1. **Read-only operations**: You only need to query blockchain state
2. **No private key**: You don't have or want to handle private keys
3. **Monitoring**: Building monitoring or analytics tools
4. **Public data**: Accessing transparent contract data
5. **Security**: Minimizing attack surface by not handling keys

```python
# Good use case for public client
public = create_public_client("https://rpc.example.com")
deposit_stats = {
    "count": public.seismic.get_deposit_count(),
    "root": public.seismic.get_deposit_root(),
}
```

### Use Wallet Client When:

1. **Transactions**: You need to send transactions
2. **Shielded operations**: Working with encrypted contract calls
3. **Signed reads**: Performing authenticated queries
4. **Deposits**: Submitting validator deposits
5. **Full contract interaction**: Using write operations

```python
# Requires wallet client
wallet = create_wallet_client(rpc_url, private_key=key)
tx_hash = wallet.seismic.send_shielded_transaction(
    to=address,
    data=data,
)
```

## Limitations

The public namespace does **not** support:

- Sending transactions (use `SeismicNamespace.send_shielded_transaction()`)
- Signed reads (use `SeismicNamespace.signed_call()`)
- Shielded contract writes (use `ShieldedContract.write`)
- Debug transactions (use `SeismicNamespace.debug_send_shielded_transaction()`)
- Validator deposits (use `SeismicNamespace.deposit()`)

Attempting to perform these operations requires upgrading to a wallet client:

```python
# Upgrade to wallet client for write operations
from seismic_web3 import create_wallet_client, PrivateKey

wallet = create_wallet_client(
    "https://rpc.example.com",
    private_key=PrivateKey(b"...32 bytes..."),
)

# Now you can perform write operations
tx_hash = wallet.seismic.send_shielded_transaction(...)
```

## See Also

- [AsyncSeismicPublicNamespace](async-seismic-public-namespace.md) - Async version of this namespace
- [SeismicNamespace](seismic-namespace.md) - Full wallet namespace with write capabilities
- [Contract Documentation](../contract/) - Contract interaction patterns
- [Client Documentation](../client/) - Client creation and configuration
- [Guides](../guides/) - Step-by-step tutorials
