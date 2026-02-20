---
description: Create async Web3 instance with full Seismic wallet capabilities
icon: arrows-spin
---

# create_async_wallet_client

Create an asynchronous `AsyncWeb3` instance with full Seismic wallet capabilities.

## Overview

`create_async_wallet_client()` is the async factory function for creating a client that can perform shielded writes, signed reads, and deposits. It supports both HTTP and WebSocket connections, fetches the TEE public key asynchronously, derives encryption state via [ECDH](https://en.wikipedia.org/wiki/Elliptic-curve_Diffie%E2%80%93Hellman), and attaches a fully-configured [`w3.seismic`](../namespaces/async-seismic-namespace.md) namespace.

The returned client works with all standard async `web3.py` APIs (`await w3.eth.get_block()`, etc.) plus the additional `w3.seismic` namespace for Seismic-specific operations.

## Signature

```python
async def create_async_wallet_client(
    provider_url: str,
    private_key: PrivateKey,
    *,
    encryption_sk: PrivateKey | None = None,
    ws: bool = False,
) -> AsyncWeb3
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `provider_url` | `str` | Yes | HTTP(S) or WS(S) URL of the Seismic node |
| `private_key` | [`PrivateKey`](../api-reference/types/private-key.md) | Yes | 32-byte secp256k1 private key for signing transactions |
| `encryption_sk` | [`PrivateKey`](../api-reference/types/private-key.md) | No | Optional 32-byte key for ECDH. If `None`, a random ephemeral key is generated |
| `ws` | `bool` | No | If `True`, uses `WebSocketProvider` (persistent connection, supports subscriptions). Otherwise uses `AsyncHTTPProvider`. Default: `False`. WebSocket is only available on async clients — sync clients are HTTP-only |

## Returns

| Type | Description |
|------|-------------|
| `AsyncWeb3` | An `AsyncWeb3` instance with `w3.seismic` namespace attached ([`AsyncSeismicNamespace`](../namespaces/async-seismic-namespace.md)) |

## Examples

### Basic Usage (HTTP)

```python
import os
from seismic_web3 import create_async_wallet_client, PrivateKey

# Load private key
private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

# Create async wallet client
w3 = await create_async_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=private_key,
)

# Now use w3.seismic for Seismic operations
contract = w3.seismic.contract(address, abi)
tx_hash = await contract.swrite.transfer(recipient, 1000)
receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
```

### WebSocket Connection

```python
import os
from seismic_web3 import create_async_wallet_client, PrivateKey

private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

# WebSocket provider for persistent connection
w3 = await create_async_wallet_client(
    "wss://gcp-1.seismictest.net/ws",
    private_key=private_key,
    ws=True,
)

# Subscribe to new blocks
async for block in w3.eth.subscribe("newHeads"):
    print(f"New block: {block['number']}")
```

### Using Chain Configuration

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

# Recommended: use chain config with HTTP
w3 = await SEISMIC_TESTNET.async_wallet_client(private_key)

# Or with WebSocket (uses ws_url from chain config)
w3 = await SEISMIC_TESTNET.async_wallet_client(private_key, ws=True)
```

### Context Manager Pattern

```python
import os
from seismic_web3 import create_async_wallet_client, PrivateKey

private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

# Use context manager to ensure cleanup
async with create_async_wallet_client(
    "wss://gcp-1.seismictest.net/ws",
    private_key=private_key,
    ws=True,
) as w3:
    contract = w3.seismic.contract(address, abi)
    tx_hash = await contract.swrite.transfer(recipient, 1000)
    receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
```

### Async Application

```python
import asyncio
import os
from seismic_web3 import create_async_wallet_client, PrivateKey

async def main():
    private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

    w3 = await create_async_wallet_client(
        "https://gcp-1.seismictest.net/rpc",
        private_key=private_key,
    )

    # Get current block
    block = await w3.eth.get_block("latest")
    print(f"Latest block: {block['number']}")

    # Get balance
    address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    balance = await w3.eth.get_balance(address)
    print(f"Balance: {w3.from_wei(balance, 'ether')} ETH")

asyncio.run(main())
```

### With Custom Encryption Key

```python
import os
from seismic_web3 import create_async_wallet_client, PrivateKey

async def setup_client():
    signing_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
    encryption_key = PrivateKey(os.urandom(32))  # Custom encryption keypair

    w3 = await create_async_wallet_client(
        "https://gcp-1.seismictest.net/rpc",
        private_key=signing_key,
        encryption_sk=encryption_key,
    )

    return w3
```

## How It Works

The function performs six steps:

1. **Create provider**
   ```python
   if ws:
       provider = WebSocketProvider(provider_url)
   else:
       provider = AsyncHTTPProvider(provider_url)
   ```

2. **Create AsyncWeb3 instance**
   ```python
   w3 = AsyncWeb3(provider)
   ```

3. **Fetch TEE public key** (async RPC call)
   ```python
   network_pk = await async_get_tee_public_key(w3)
   ```

4. **Generate encryption keypair** (if `encryption_sk` is `None`, a random ephemeral key is created)
   ```python
   encryption_sk = encryption_sk or PrivateKey(os.urandom(32))
   ```

5. **Derive encryption state** (ECDH + [HKDF](https://en.wikipedia.org/wiki/HKDF))
   ```python
   encryption = get_encryption(network_pk, encryption_sk)
   ```

6. **Attach Seismic namespace**
   ```python
   w3.seismic = AsyncSeismicNamespace(w3, encryption, private_key)
   ```

## Client Capabilities

The returned client provides:

### Standard AsyncWeb3 Methods (e.g. `w3.eth`, `w3.net`)
- `await get_block()`, `await get_transaction()`, `await get_balance()`
- `await send_raw_transaction()`, `await wait_for_transaction_receipt()`
- All other standard async `web3.py` functionality

### Async Seismic Methods (`w3.seismic`)
- [`await send_shielded_transaction()`](../namespaces/methods/send-shielded-transaction.md) - Send shielded transactions
- [`await debug_send_shielded_transaction()`](../namespaces/methods/debug-send-shielded-transaction.md) - Debug shielded transactions
- [`await signed_call()`](../namespaces/methods/signed-call.md) - Execute signed reads
- [`await deposit()`](../namespaces/methods/deposit.md) - Deposit ETH/tokens
- [`await get_tee_public_key()`](../namespaces/methods/get-tee-public-key.md) - Get TEE public key
- [`await get_deposit_root()`](../namespaces/methods/get-deposit-root.md) - Query deposit merkle root
- [`await get_deposit_count()`](../namespaces/methods/get-deposit-count.md) - Query deposit count
- [`contract()`](../contract/) - Create contract wrappers (methods are async)

## HTTP vs WebSocket

| Aspect | AsyncHTTPProvider (`ws=False`) | WebSocketProvider (`ws=True`) |
|--------|-------------------------------|-------------------------------|
| **Connection** | New connection per request | Persistent connection |
| **Latency** | Higher per-request overhead | Lower latency |
| **Subscriptions** | Not supported | Supported (`eth.subscribe`) |
| **Resource usage** | Lower idle usage | Keeps connection open |
| **Use case** | One-off transactions | Real-time monitoring, subscriptions |

## Encryption

The client automatically:
- Fetches the network's TEE public key asynchronously
- Performs ECDH key exchange using `encryption_sk` (or generates a random one)
- Derives a shared AES-GCM key via HKDF
- Uses this key to encrypt all shielded transaction calldata and signed reads

Access the encryption state at `w3.seismic.encryption` if needed for advanced use cases.

## Notes

- The function is `async` and must be `await`-ed
- Makes one asynchronous RPC call to fetch the TEE public key
- If `encryption_sk` is `None`, a random ephemeral key is generated
- The encryption key is separate from the transaction signing key
- WebSocket connections should be properly closed when done
- All `w3.seismic` methods are async and must be `await`-ed
- For sync operations, use [`create_wallet_client()`](create-wallet-client.md)

## Warnings

- **Private key security** - Never log or expose private keys. Use environment variables or secure key management
- **Connection cleanup** - Close WebSocket connections properly to avoid resource leaks
- **Error handling** - WebSocket connections can drop; implement reconnection logic for production
- **HTTPS/WSS recommended** - Use secure protocols in production to prevent MITM attacks

## See Also

- [create_wallet_client](create-wallet-client.md) - Sync variant (HTTP only)
- [create_async_public_client](create-async-public-client.md) - Async read-only client
- [EncryptionState](encryption-state.md) - Encryption state class
- [get_encryption](get-encryption.md) - Encryption derivation function
- [AsyncSeismicNamespace](../namespaces/async-seismic-namespace.md) - The async `w3.seismic` namespace
- [Chains Configuration](../chains.md) - Pre-configured chain constants
