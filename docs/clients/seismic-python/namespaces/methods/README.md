---
description: Reference documentation for Seismic namespace methods
icon: brackets-curly
---

# Namespace Methods

This section provides detailed documentation for all methods available on the Seismic namespace classes. Each method is documented with signatures, parameters, return values, examples, and implementation details.

***

## Method Categories

### Query Methods (Public)

These methods are available on both public and wallet clients. They query read-only data from the network.

| Method | Description | Available On |
|--------|-------------|--------------|
| [get_tee_public_key()](get-tee-public-key.md) | Fetch the TEE's public key for encryption | Public + Wallet |
| [get_deposit_root()](get-deposit-root.md) | Read the deposit Merkle root | Public + Wallet |
| [get_deposit_count()](get-deposit-count.md) | Read the total number of deposits | Public + Wallet |

### Transaction Methods (Wallet Only)

These methods require a private key and are only available on wallet clients.

| Method | Description | Available On |
|--------|-------------|--------------|
| [send_shielded_transaction()](send-shielded-transaction.md) | Send a shielded transaction with encrypted calldata | Wallet only |
| [debug_send_shielded_transaction()](debug-send-shielded-transaction.md) | Send a shielded transaction and return debug info | Wallet only |
| [signed_call()](signed-call.md) | Execute a signed read with encryption | Wallet only |
| [deposit()](deposit.md) | Submit a validator deposit | Wallet only |

***

## Quick Reference

### Shielded Transactions

```python
# Send encrypted transaction
tx_hash = w3.seismic.send_shielded_transaction(
    to=contract_address,
    data=encoded_calldata,
    value=0,
)

# Send with debug info
result = w3.seismic.debug_send_shielded_transaction(
    to=contract_address,
    data=encoded_calldata,
)
```

### Signed Reads

```python
# Execute encrypted read
response = w3.seismic.signed_call(
    to=contract_address,
    data=encoded_calldata,
)
```

### Deposit Queries

```python
# Query deposit information
tee_key = w3.seismic.get_tee_public_key()
count = w3.seismic.get_deposit_count()
root = w3.seismic.get_deposit_root()
```

### Validator Deposits

```python
# Submit validator deposit
tx_hash = w3.seismic.deposit(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    deposit_data_root=deposit_data_root,
    value=32 * 10**18,
)
```

***

## Method Comparison

### Write Operations

| Feature | send_shielded_transaction | debug_send_shielded_transaction | deposit |
|---------|--------------------------|--------------------------------|---------|
| **Encrypts calldata** | ✅ Yes | ✅ Yes | ❌ No (transparent) |
| **Returns transaction hash** | ✅ Yes | ✅ Yes | ✅ Yes |
| **Returns debug info** | ❌ No | ✅ Yes | ❌ No |
| **Modifies state** | ✅ Yes | ✅ Yes | ✅ Yes |
| **Use case** | Production writes | Development/debugging | Validator deposits |

### Read Operations

| Feature | signed_call | get_tee_public_key | get_deposit_root | get_deposit_count |
|---------|------------|-------------------|-----------------|-------------------|
| **Requires private key** | ✅ Yes | ❌ No | ❌ No | ❌ No |
| **Encrypts calldata** | ✅ Yes | N/A | N/A | N/A |
| **Modifies state** | ❌ No | ❌ No | ❌ No | ❌ No |
| **Use case** | Private reads | Get encryption key | Query deposit tree | Count validators |

***

## Client Availability

### Public Clients

Created with `create_public_client()` or `create_async_public_client()`:

```python
w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

# Available methods
w3.seismic.get_tee_public_key()
w3.seismic.get_deposit_root()
w3.seismic.get_deposit_count()

# Not available (require private key)
w3.seismic.send_shielded_transaction()  # ❌ AttributeError
w3.seismic.signed_call()                # ❌ AttributeError
w3.seismic.deposit()                    # ❌ AttributeError
```

### Wallet Clients

Created with `create_wallet_client()` or `create_async_wallet_client()`:

```python
w3 = create_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=PrivateKey(b"..."),
)

# All methods available
w3.seismic.get_tee_public_key()
w3.seismic.get_deposit_root()
w3.seismic.get_deposit_count()
w3.seismic.send_shielded_transaction(...)
w3.seismic.signed_call(...)
w3.seismic.deposit(...)
```

***

## Sync vs Async

All methods are available in both synchronous and asynchronous variants:

### Sync Example

```python
from seismic_web3 import create_wallet_client

w3 = create_wallet_client(...)

# Synchronous calls
tee_key = w3.seismic.get_tee_public_key()
tx_hash = w3.seismic.send_shielded_transaction(...)
result = w3.seismic.signed_call(...)
```

### Async Example

```python
from seismic_web3 import create_async_wallet_client

w3 = await create_async_wallet_client(...)

# Asynchronous calls (must await)
tee_key = await w3.seismic.get_tee_public_key()
tx_hash = await w3.seismic.send_shielded_transaction(...)
result = await w3.seismic.signed_call(...)
```

***

## Common Patterns

### Error Handling

All methods can raise exceptions. Wrap calls in try-except blocks:

```python
try:
    tx_hash = w3.seismic.send_shielded_transaction(
        to=contract_address,
        data=calldata,
    )
    receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
    print(f"Success: {receipt['status'] == 1}")
except ValueError as e:
    print(f"Transaction failed: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

### Transaction Confirmation

Always wait for transaction confirmation before assuming success:

```python
# Send transaction
tx_hash = w3.seismic.send_shielded_transaction(...)

# Wait for confirmation (with timeout)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash, timeout=120)

# Check status
if receipt['status'] == 1:
    print("Transaction succeeded")
else:
    print("Transaction reverted")
```

### Security Parameters

Customize security parameters when needed:

```python
from seismic_web3 import SeismicSecurityParams

# Use longer expiry window
security = SeismicSecurityParams(blocks_window=200)

tx_hash = w3.seismic.send_shielded_transaction(
    to=contract_address,
    data=calldata,
    security=security,
)
```

***

## Best Practices

### Use High-Level APIs When Possible

```python
# ✅ Recommended: Use contract methods
contract = w3.seismic.contract(address, abi)
tx_hash = contract.write.transfer(recipient, amount)

# ⚠️ Lower-level: Manual encoding
from seismic_web3.contract.abi import encode_shielded_calldata
calldata = encode_shielded_calldata(abi, "transfer", [recipient, amount])
tx_hash = w3.seismic.send_shielded_transaction(to=address, data=calldata)
```

### Debug During Development

```python
# Development: Use debug methods
result = w3.seismic.debug_send_shielded_transaction(...)
print(f"Plaintext: {result.plaintext_tx.data.hex()}")
print(f"Encrypted: {result.shielded_tx.data.hex()}")

# Production: Use regular methods
tx_hash = w3.seismic.send_shielded_transaction(...)
```

### Validate Inputs

```python
# Validate parameter lengths for deposits
assert len(node_pubkey) == 32, "node_pubkey must be 32 bytes"
assert len(consensus_pubkey) == 48, "consensus_pubkey must be 48 bytes"

# Then submit
tx_hash = w3.seismic.deposit(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    # ... other parameters
)
```

***

## Method Documentation

### Query Methods

- [get_tee_public_key()](get-tee-public-key.md) — Fetch TEE's encryption public key
- [get_deposit_root()](get-deposit-root.md) — Read deposit Merkle root
- [get_deposit_count()](get-deposit-count.md) — Read total validator deposits

### Transaction Methods

- [send_shielded_transaction()](send-shielded-transaction.md) — Send encrypted transaction
- [debug_send_shielded_transaction()](debug-send-shielded-transaction.md) — Send with debug info
- [signed_call()](signed-call.md) — Execute signed read

### Deposit Methods

- [deposit()](deposit.md) — Submit validator deposit

***

## See Also

- [SeismicNamespace](../seismic-namespace.md) — Full namespace class (sync)
- [AsyncSeismicNamespace](../async-seismic-namespace.md) — Full namespace class (async)
- [SeismicPublicNamespace](../seismic-public-namespace.md) — Public (read-only) namespace (sync)
- [AsyncSeismicPublicNamespace](../async-seismic-public-namespace.md) — Public (read-only) namespace (async)
- [Contract Namespaces](../../contract/namespaces/) — Contract method documentation
- [Client Documentation](../../client/) — Client creation and configuration
