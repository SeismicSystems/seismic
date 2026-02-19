---
description: Pre-defined and custom chain configurations
icon: link
---

# Chains

The SDK ships with pre-defined configs for common networks:

```python
from seismic_web3 import SEISMIC_TESTNET, SANVIL, make_seismic_testnet, ChainConfig

SEISMIC_TESTNET.rpc_url  # "https://gcp-1.seismictest.net/rpc"
SEISMIC_TESTNET.ws_url   # "wss://gcp-1.seismictest.net/ws"
SEISMIC_TESTNET.chain_id # 5124

SANVIL.rpc_url   # "http://127.0.0.1:8545"
SANVIL.chain_id  # 31337
```

***

### Creating clients from a chain config

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))

# Sync
w3 = SEISMIC_TESTNET.create_client(pk)

# Async (auto-selects ws_url when ws=True)
w3 = await SEISMIC_TESTNET.create_async_client(pk, ws=True)
```

***

### Other testnet instances

```python
testnet_2 = make_seismic_testnet(2)  # gcp-2.seismictest.net
testnet_3 = make_seismic_testnet(3)  # gcp-3.seismictest.net
```

***

### Custom chains

```python
custom = ChainConfig(
    chain_id=1234,
    rpc_url="https://my-node.example.com/rpc",
    ws_url="wss://my-node.example.com/ws",
    name="My Network",
)

w3 = custom.create_client(pk)
```
