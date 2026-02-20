---
description: w3.seismic namespace reference
icon: cube
---

# Namespaces

The `w3.seismic` namespace provides Seismic-specific functionality for wallet and public clients. The available methods depend on the client type.

## Namespace Types

| Namespace | Client Type | Sync/Async | Description |
|-----------|-------------|------------|-------------|
| `SeismicNamespace` | Wallet | Sync | Full capabilities (writes, reads, deposits) |
| `AsyncSeismicNamespace` | Wallet | Async | Full capabilities (async) |
| `SeismicPublicNamespace` | Public | Sync | Read-only operations |
| `AsyncSeismicPublicNamespace` | Public | Async | Read-only operations (async) |

## Wallet Namespace Methods

Available on wallet clients created with `create_wallet_client()` or `create_async_wallet_client()`:

### Transaction Methods
- `send_shielded_transaction()` - Send a shielded transaction with encrypted calldata
- `debug_send_shielded_transaction()` - Send shielded transaction with debug info
- `signed_call()` - Execute a signed read (eth_call with encrypted calldata)

### Query Methods
- `get_tee_public_key()` - Retrieve the TEE's public key for encryption
- `get_deposit_root()` - Query the deposit contract merkle root
- `get_deposit_count()` - Query the total number of deposits

### Deposit Methods
- `deposit()` - Deposit ETH or tokens into the Seismic network

### Contract Factory
- `contract()` - Create `ShieldedContract` or `PublicContract` wrappers

## Public Namespace Methods

Available on public clients created with `create_public_client()` or `create_async_public_client()`:

### Query Methods
- `get_tee_public_key()` - Retrieve the TEE's public key
- `get_deposit_root()` - Query the deposit contract merkle root
- `get_deposit_count()` - Query the total number of deposits

### Contract Factory
- `contract()` - Create `PublicContract` wrappers (transparent reads only)

## Quick Examples

### Wallet Client

```python
from seismic_web3 import create_wallet_client, PrivateKey

w3 = create_wallet_client("https://rpc.example.com", private_key=PrivateKey(...))

# Get TEE public key
tee_key = w3.seismic.get_tee_public_key()

# Execute signed read
result = w3.seismic.signed_call(
    to=contract_address,
    data=encoded_call,
)

# Create contract wrapper
contract = w3.seismic.contract(contract_address, abi)
```

### Public Client

```python
from seismic_web3 import create_public_client

public = create_public_client("https://rpc.example.com")

# Get TEE public key
tee_key = public.seismic.get_tee_public_key()

# Query deposits
deposit_count = public.seismic.get_deposit_count()
deposit_root = public.seismic.get_deposit_root()

# Create contract wrapper (read-only)
contract = public.seismic.contract(contract_address, abi)
```

## See Also

- [Client Documentation](../client/) - Client creation and configuration
- [Contract Documentation](../contract/) - Contract interaction patterns
- [Shielded Write Guide](../guides/shielded-write.md) - Step-by-step transaction guide
- [Signed Reads Guide](../guides/signed-reads.md) - Signed read patterns
