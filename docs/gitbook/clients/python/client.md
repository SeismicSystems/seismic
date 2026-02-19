---
description: Creating sync and async Seismic clients
icon: plug
---

# Client

***

### Sync

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))

w3 = SEISMIC_TESTNET.create_client(pk)
```

This gives you a standard `Web3` instance with an extra `w3.seismic` namespace bolted on. Everything from `web3.py` works as usual — `w3.eth.get_block(...)`, `w3.eth.wait_for_transaction_receipt(...)`, etc. The `w3.seismic` namespace is where the Seismic-specific stuff lives.

***

### Async

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))

# HTTP
w3 = await SEISMIC_TESTNET.create_async_client(pk)

# WebSocket (auto-selects ws_url from chain config)
w3 = await SEISMIC_TESTNET.create_async_client(pk, ws=True)
```

Every method on `w3.seismic` and on `ShieldedContract` is `await`-able when using the async client.

***

### Using a URL

You can also pass a raw URL to the factory functions directly:

```python
from seismic_web3 import create_shielded_web3, create_async_shielded_web3, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))

# Sync
w3 = create_shielded_web3("http://127.0.0.1:8545", private_key=pk)

# Async HTTP
w3 = await create_async_shielded_web3("http://127.0.0.1:8545", private_key=pk)

# Async WebSocket
w3 = await create_async_shielded_web3("ws://127.0.0.1:8545", private_key=pk, ws=True)
```

***

### Encryption

The client automatically handles encryption setup. On creation, it fetches the node's TEE public key and derives a shared AES-GCM key via ECDH. This key is used to encrypt calldata for every shielded transaction and signed read.

You don't need to manage this yourself — but the encryption state is accessible at `w3.seismic.encryption` if you need it.
