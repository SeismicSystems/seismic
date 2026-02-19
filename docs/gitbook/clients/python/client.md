---
description: Creating sync and async Seismic clients
icon: plug
---

# Client

***

### Sync

```python
from seismic_web3 import create_shielded_web3, PrivateKey

w3 = create_shielded_web3(
    "http://127.0.0.1:8545",
    private_key=PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX")),
)
```

This gives you a standard `Web3` instance with an extra `w3.seismic` namespace bolted on. Everything from `web3.py` works as usual — `w3.eth.get_block(...)`, `w3.eth.wait_for_transaction_receipt(...)`, etc. The `w3.seismic` namespace is where the Seismic-specific stuff lives.

***

### Async

```python
from seismic_web3 import create_async_shielded_web3, PrivateKey

# HTTP
w3 = await create_async_shielded_web3(
    "http://127.0.0.1:8545",
    private_key=PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX")),
)

# WebSocket (persistent connection, supports subscriptions)
w3 = await create_async_shielded_web3(
    "ws://127.0.0.1:8545",
    private_key=PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX")),
    use_websocket=True,
)
```

Every method on `w3.seismic` and on `ShieldedContract` is `await`-able when using the async client.

***

### Encryption

The client automatically handles encryption setup. On creation, it fetches the node's TEE public key and derives a shared AES-GCM key via ECDH. This key is used to encrypt calldata for every shielded transaction and signed read.

You don't need to manage this yourself — but the encryption state is accessible at `w3.seismic.encryption` if you need it.
