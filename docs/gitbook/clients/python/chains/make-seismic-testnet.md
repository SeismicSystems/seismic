---
description: Factory function for testnet chain configs
icon: network-wired
---

# make_seismic_testnet

Factory function that creates a [`ChainConfig`](chain-config.md) for a Seismic testnet node.

## Overview

`make_seismic_testnet` generates a chain configuration for a GCP testnet node or a custom host. This is useful when you need to connect to alternate testnet nodes beyond the default [`SEISMIC_TESTNET`](seismic-testnet.md) (which is GCP node 1).

## Definition

```python
def make_seismic_testnet(
    n: int = 1,
    *,
    host: str | None = None,
) -> ChainConfig:
```

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `n` | `int` | No | `1` | GCP node number |
| `host` | `str \| None` | No | `None` | Custom hostname. Mutually exclusive with `n` |

Raises `ValueError` if both `n` and `host` are provided.

## Returns

- `ChainConfig` - A chain configuration for the specified testnet

## Examples

### Connect to Alternate GCP Nodes

```python
import os
from seismic_web3 import make_seismic_testnet, PrivateKey

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

# GCP node 1 (same as SEISMIC_TESTNET)
testnet_1 = make_seismic_testnet(1)
w3_1 = testnet_1.wallet_client(pk)

# GCP node 2
testnet_2 = make_seismic_testnet(2)
w3_2 = testnet_2.wallet_client(pk)
```

### Custom Host

```python
from seismic_web3 import make_seismic_testnet

testnet = make_seismic_testnet(host="my-testnet.example.com")
print(testnet.rpc_url)  # "https://my-testnet.example.com/rpc"
```

### Default Parameter

```python
from seismic_web3 import make_seismic_testnet

# No argument defaults to GCP node 1
testnet = make_seismic_testnet()

print(testnet.rpc_url)  # "https://gcp-1.seismictest.net/rpc"
```

## Relationship to SEISMIC_TESTNET

The default [`SEISMIC_TESTNET`](seismic-testnet.md) constant is defined as:

```python
SEISMIC_TESTNET: ChainConfig = make_seismic_testnet(1)
```

## See Also

- [ChainConfig](chain-config.md) - Chain configuration dataclass
- [SEISMIC_TESTNET](seismic-testnet.md) - Pre-defined testnet (GCP node 1)
- [PrivateKey](../api-reference/types/private-key.md) - Private key type
