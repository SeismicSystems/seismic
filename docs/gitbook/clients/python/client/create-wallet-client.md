---
description: Create sync Web3 instance with full Seismic wallet capabilities
icon: wallet
---

# create_wallet_client

Create a synchronous `Web3` instance with full Seismic wallet capabilities.

## Overview

`create_wallet_client()` is the primary factory function for creating a sync client that can perform shielded writes, signed reads, and deposits. It fetches the TEE public key, derives encryption state via [ECDH](https://en.wikipedia.org/wiki/Elliptic-curve_Diffie%E2%80%93Hellman), and attaches a fully-configured [`w3.seismic`](../namespaces/seismic-namespace.md) namespace to a standard `Web3` instance.

The returned client works with all standard `web3.py` APIs (`w3.eth.get_block()`, `w3.eth.send_raw_transaction()`, etc.) plus the additional `w3.seismic` namespace for Seismic-specific operations.

## Signature

```python
def create_wallet_client(
    rpc_url: str,
    private_key: PrivateKey,
    *,
    encryption_sk: PrivateKey | None = None,
) -> Web3
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `rpc_url` | `str` | Yes | HTTP(S) URL of the Seismic node (e.g., `"https://gcp-1.seismictest.net/rpc"`). WebSocket URLs are not supported — see note below |
| `private_key` | [`PrivateKey`](../api-reference/types/private-key.md) | Yes | 32-byte secp256k1 private key for signing transactions |
| `encryption_sk` | [`PrivateKey`](../api-reference/types/private-key.md) | No | Optional 32-byte key for ECDH. If `None`, a random ephemeral key is generated |

## Returns

| Type | Description |
|------|-------------|
| `Web3` | A `Web3` instance with `w3.seismic` namespace attached ([`SeismicNamespace`](../namespaces/seismic-namespace.md)) |

## Examples

### Basic Usage

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey

# Load private key
private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))

# Create wallet client
w3 = create_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=private_key,
)

# Now use w3.seismic for Seismic operations
contract = w3.seismic.contract(address, abi)
tx_hash = contract.swrite.transfer(recipient, 1000)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
```

### Using Chain Configuration

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))

# Recommended: use chain config instead of raw URL
w3 = SEISMIC_TESTNET.wallet_client(private_key)

# Equivalent to:
# w3 = create_wallet_client(SEISMIC_TESTNET.rpc_url, private_key=private_key)
```

### With Custom Encryption Key

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey

signing_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
encryption_key = PrivateKey(os.urandom(32))  # Custom encryption keypair

w3 = create_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=signing_key,
    encryption_sk=encryption_key,
)
```

### Standard Web3 Operations

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# All standard web3.py operations work
block = w3.eth.get_block("latest")
balance = w3.eth.get_balance("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb")
chain_id = w3.eth.chain_id
```

## How It Works

The function performs four steps:

1. **Create Web3 instance**
   ```python
   w3 = Web3(Web3.HTTPProvider(rpc_url))
   ```

2. **Fetch TEE public key** (synchronous RPC call)
   ```python
   network_pk = get_tee_public_key(w3)
   ```

3. **Derive encryption state** (ECDH + HKDF)
   ```python
   encryption = get_encryption(network_pk, encryption_sk)
   ```

4. **Attach Seismic namespace**
   ```python
   w3.seismic = SeismicNamespace(w3, encryption, private_key)
   ```

## Client Capabilities

The returned client provides:

### Standard Web3 Methods (`w3.eth`)
- `get_block()`, `get_transaction()`, `get_balance()`
- `send_raw_transaction()`, `wait_for_transaction_receipt()`
- All other standard `web3.py` functionality

### Seismic Methods (`w3.seismic`)
- `send_shielded_transaction()` - Send shielded transactions
- `debug_send_shielded_transaction()` - Debug shielded transactions
- `signed_call()` - Execute signed reads
- `deposit()` - Deposit ETH/tokens
- `get_tee_public_key()` - Get TEE public key
- `get_deposit_root()` - Query deposit merkle root
- `get_deposit_count()` - Query deposit count
- `contract()` - Create contract wrappers

## Encryption

The client automatically:
- Fetches the network's TEE public key
- Performs ECDH key exchange using `encryption_sk` (or generates a random one)
- Derives a shared AES-GCM key via HKDF
- Uses this key to encrypt all shielded transaction calldata and signed reads

Access the encryption state at `w3.seismic.encryption` if needed for advanced use cases.

## Notes

- **HTTP only** — Sync clients use `Web3` with `HTTPProvider`, which does not support WebSocket connections. This is a limitation of the underlying `web3.py` library (`WebSocketProvider` is async-only). If you need WebSocket support (persistent connections, subscriptions), use [`create_async_wallet_client()`](create-async-wallet-client.md) with `ws=True`
- The function makes one synchronous RPC call to fetch the TEE public key
- If `encryption_sk` is `None`, a random ephemeral key is generated
- The encryption key is separate from the transaction signing key
- The returned `Web3` instance is fully compatible with all `web3.py` APIs
- For async operations, use [`create_async_wallet_client()`](create-async-wallet-client.md)

## Warnings

- **Private key security** - Never log or expose private keys. Use environment variables or secure key management
- **RPC URL validation** - Ensure the RPC URL is correct and accessible
- **Network connectivity** - The function will fail if it cannot reach the RPC endpoint
- **HTTPS recommended** - Use HTTPS URLs in production to prevent MITM attacks

## See Also

- [create_async_wallet_client](create-async-wallet-client.md) - Async variant with WebSocket support
- [create_public_client](create-public-client.md) - Read-only client without private key
- [EncryptionState](encryption-state.md) - Encryption state class
- [get_encryption](get-encryption.md) - Encryption derivation function
- [SeismicNamespace](../namespaces/seismic-namespace.md) - The `w3.seismic` namespace
- [Chains Configuration](../chains.md) - Pre-configured chain constants
