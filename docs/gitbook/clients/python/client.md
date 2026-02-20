---
description: Creating sync and async Seismic clients
icon: plug
---

# Client

### Install

```bash
pip install seismic-web3
```

Or with [uv](https://docs.astral.sh/uv/):

```bash
uv add seismic-web3
```

***

The SDK provides two client types:

- **Wallet client** — full capabilities (shielded writes, signed reads, deposits). Requires a private key.
- **Public client** — read-only (transparent reads, TEE public key, deposit queries). No private key needed.

***

### Wallet client (sync)

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))

w3 = SEISMIC_TESTNET.wallet_client(pk)
```

This gives you a standard `Web3` instance with an extra `w3.seismic` namespace bolted on. Everything from `web3.py` works as usual — `w3.eth.get_block(...)`, `w3.eth.wait_for_transaction_receipt(...)`, etc. The `w3.seismic` namespace is where the Seismic-specific stuff lives.

***

### Wallet client (async)

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))

# HTTP
w3 = await SEISMIC_TESTNET.async_wallet_client(pk)

# WebSocket (auto-selects ws_url from chain config)
w3 = await SEISMIC_TESTNET.async_wallet_client(pk, ws=True)
```

Every method on `w3.seismic` and on `ShieldedContract` is `await`-able when using the async client.

***

### Public client

```python
from seismic_web3 import SEISMIC_TESTNET

# Sync
public = SEISMIC_TESTNET.public_client()

# Async
public = SEISMIC_TESTNET.async_public_client()
```

The public client's `w3.seismic` namespace has `get_tee_public_key()`, `get_deposit_root()`, `get_deposit_count()`, and `contract()` (with `.tread` only).

***

### Using a URL

You can also pass a raw URL to the factory functions directly:

```python
import os
from seismic_web3 import create_wallet_client, create_public_client, PrivateKey

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))

# Wallet (sync)
w3 = create_wallet_client("http://127.0.0.1:8545", private_key=pk)

# Public (sync)
public = create_public_client("http://127.0.0.1:8545")
```

Async variants:

```python
from seismic_web3 import create_async_wallet_client, create_async_public_client

# Async wallet (HTTP)
w3 = await create_async_wallet_client("http://127.0.0.1:8545", private_key=pk)

# Async wallet (WebSocket)
w3 = await create_async_wallet_client("ws://127.0.0.1:8545", private_key=pk, ws=True)

# Async public
public = create_async_public_client("http://127.0.0.1:8545")
```

***

### Encryption

The wallet client automatically handles encryption setup. On creation, it fetches the network's TEE public key and derives a shared AES-GCM key via ECDH. This key is used to encrypt calldata for every shielded transaction and signed read.

You don't need to manage this yourself — but the encryption state is accessible at `w3.seismic.encryption` if you need it.
