---
icon: link
---

# Chains

The SDK provides pre-configured chain objects that bundle RPC endpoints, WebSocket URLs, and chain IDs.

## Built-in chains

| Chain             | Chain ID | RPC endpoint                    |
| ----------------- | -------- | ------------------------------- |
| `SEISMIC_TESTNET` | 5124     | `https://gcp-1.seismictest.net` |
| `SANVIL`          | 31337    | `http://127.0.0.1:8545`         |

## Usage

```python
from seismic_web3.chains import SEISMIC_TESTNET, SANVIL

# Access properties
print(SEISMIC_TESTNET.rpc_url)
print(SEISMIC_TESTNET.ws_url)
print(SEISMIC_TESTNET.chain_id)
print(SEISMIC_TESTNET.name)
```

## Creating clients from chains

```python
from seismic_web3.chains import SEISMIC_TESTNET
from eth_account import Account

key = Account.from_key("0x...").key

# Sync wallet client
w3 = SEISMIC_TESTNET.wallet_client(key)

# Async wallet client (HTTP)
w3 = await SEISMIC_TESTNET.async_wallet_client(key)

# Async wallet client (WebSocket)
w3 = await SEISMIC_TESTNET.async_wallet_client(key, ws=True)

# Public client (no key needed)
w3 = SEISMIC_TESTNET.public_client()
```

## Custom chains

Use `make_seismic_testnet()` to create a chain config pointing at a different testnet instance:

```python
from seismic_web3.chains import make_seismic_testnet

custom = make_seismic_testnet(rpc_url="https://custom-node.example.com")
w3 = custom.wallet_client(key)
```

Chain configurations are immutable (frozen dataclasses). WebSocket is used automatically when `ws=True` and the chain config includes a `ws_url`. HTTP is the fallback.
