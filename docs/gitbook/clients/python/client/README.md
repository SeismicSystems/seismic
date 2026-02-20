---
description: Creating sync and async Seismic clients
icon: plug
---

# Client

The Seismic Python SDK provides two client types for interacting with Seismic nodes:

- **[Wallet client](create-wallet-client.md)** — Full capabilities (shielded writes, signed reads, deposits). Requires a private key.
- **[Public client](create-public-client.md)** — Read-only (transparent reads, TEE public key, deposit queries). No private key needed.

Both clients are available in **sync** and **async** variants.

## Quick Start

### Installation

```bash
pip install seismic-web3
```

Or with [uv](https://docs.astral.sh/uv/):

```bash
uv add seismic-web3
```

### Wallet Client (Sync)

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))
w3 = SEISMIC_TESTNET.wallet_client(pk)
```

This gives you a standard `Web3` instance with an extra `w3.seismic` namespace. Everything from `web3.py` works as usual — `w3.eth.get_block(...)`, `w3.eth.wait_for_transaction_receipt(...)`, etc.

### Wallet Client (Async)

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))

# HTTP
w3 = await SEISMIC_TESTNET.async_wallet_client(pk)

# WebSocket (auto-selects ws_url from chain config)
w3 = await SEISMIC_TESTNET.async_wallet_client(pk, ws=True)
```

Every method on `w3.seismic` and on `ShieldedContract` is `await`-able when using the async client.

### Public Client

```python
from seismic_web3 import SEISMIC_TESTNET

# Sync
public = SEISMIC_TESTNET.public_client()

# Async
public = await SEISMIC_TESTNET.async_public_client()
```

The public client's `w3.seismic` namespace has limited methods: `get_tee_public_key()`, `get_deposit_root()`, `get_deposit_count()`, and `contract()` (with `.tread` only).

## Client Factory Functions

### Wallet Clients (Require Private Key)

| Function | Type | Description |
|----------|------|-------------|
| [create_wallet_client](create-wallet-client.md) | Sync | Create sync wallet client from RPC URL |
| [create_async_wallet_client](create-async-wallet-client.md) | Async | Create async wallet client from RPC URL |

### Public Clients (No Private Key)

| Function | Type | Description |
|----------|------|-------------|
| [create_public_client](create-public-client.md) | Sync | Create sync public client from RPC URL |
| [create_async_public_client](create-async-public-client.md) | Async | Create async public client from RPC URL |

## Chain-Based Creation

The recommended approach is to use chain configuration objects:

```python
from seismic_web3 import SEISMIC_TESTNET, SANVIL, PrivateKey

# Seismic testnet
pk = PrivateKey(...)
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Sanvil testnet
w3 = SANVIL.wallet_client(pk)
```

See [Chains Configuration](../chains/) for more details.

## Encryption

The wallet client automatically handles encryption setup:

1. Fetches the network's TEE public key
2. Derives a shared AES-GCM key via ECDH
3. Uses this key to encrypt calldata for all shielded transactions and signed reads

You don't need to manage this manually, but the encryption state is accessible at `w3.seismic.encryption` if needed.

### Encryption Components

| Component | Description |
|-----------|-------------|
| [EncryptionState](encryption-state.md) | Holds AES key and encryption keypair |
| [get_encryption](get-encryption.md) | Derives encryption state from TEE public key |

## Client Capabilities

### Wallet Client (`w3.seismic`)

- `send_shielded_transaction()` - Send shielded transactions
- `debug_send_shielded_transaction()` - Debug shielded transactions
- `signed_call()` - Execute signed reads
- `get_tee_public_key()` - Get TEE public key
- `deposit()` - Deposit ETH/tokens
- `get_deposit_root()` - Query deposit merkle root
- `get_deposit_count()` - Query deposit count
- `contract()` - Create contract wrappers

### Public Client (`w3.seismic`)

- `get_tee_public_key()` - Get TEE public key
- `get_deposit_root()` - Query deposit merkle root
- `get_deposit_count()` - Query deposit count
- `contract()` - Create contract wrappers (`.tread` only)

## See Also

- [Contract Instances](../contract/) - Working with shielded and public contracts
- [Namespaces](../namespaces/) - Detailed `w3.seismic` namespace documentation
- [Chains Configuration](../chains/) - Chain configs and constants
- [Shielded Write Guide](../guides/shielded-write.md) - Step-by-step shielded transaction guide
