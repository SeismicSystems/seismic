---
description: Factory function for alternate testnet instances
icon: network-wired
---

# make\_seismic\_testnet

Factory function that creates a [`ChainConfig`](chain-config.md) for a specific GCP testnet instance.

## Overview

`make_seismic_testnet(n)` generates a chain configuration for GCP testnet instance _n_. This is useful when you need to connect to alternate testnet instances beyond the default [`SEISMIC_TESTNET`](seismic-testnet.md) (which is GCP-1).

## Definition

```python
def make_seismic_testnet(n: int = 1) -> ChainConfig:
    """Create a ``ChainConfig`` for GCP testnet instance *n*.

    Args:
        n: GCP instance number (default ``1``).

    Returns:
        A ``ChainConfig`` pointing at ``gcp-{n}.seismictest.net``.
    """
    host = f"gcp-{n}.seismictest.net"
    return ChainConfig(
        chain_id=SEISMIC_TESTNET_CHAIN_ID,
        rpc_url=f"https://{host}/rpc",
        ws_url=f"wss://{host}/ws",
        name=f"Seismic Testnet (GCP-{n})",
    )
```

## Parameters

| Parameter | Type  | Required | Default | Description         |
| --------- | ----- | -------- | ------- | ------------------- |
| `n`       | `int` | No       | `1`     | GCP instance number |

## Returns

* `ChainConfig` - A chain configuration pointing to `gcp-{n}.seismictest.net`

## Configuration

The returned `ChainConfig` has:

| Property   | Value                                    |
| ---------- | ---------------------------------------- |
| `chain_id` | `5124` (same for all testnet instances)  |
| `rpc_url`  | `f"https://gcp-{n}.seismictest.net/rpc"` |
| `ws_url`   | `f"wss://gcp-{n}.seismictest.net/ws"`    |
| `name`     | `f"Seismic Testnet (GCP-{n})"`           |

## Examples

### Connect to Alternate Testnet Instances

```python
from seismic_web3 import make_seismic_testnet, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))

# GCP-1 (same as SEISMIC_TESTNET)
testnet_1 = make_seismic_testnet(1)
w3_1 = testnet_1.wallet_client(pk)

# GCP-2
testnet_2 = make_seismic_testnet(2)
w3_2 = testnet_2.wallet_client(pk)

# GCP-3
testnet_3 = make_seismic_testnet(3)
w3_3 = testnet_3.wallet_client(pk)
```

### Default Parameter

```python
from seismic_web3 import make_seismic_testnet

# No argument defaults to instance 1
testnet = make_seismic_testnet()  # Same as make_seismic_testnet(1)

print(testnet.rpc_url)  # "https://gcp-1.seismictest.net/rpc"
print(testnet.name)     # "Seismic Testnet (GCP-1)"
```

### Multi-Instance Testing

```python
from seismic_web3 import make_seismic_testnet, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))

# Test against multiple instances
instances = [1, 2, 3]

for i in instances:
    testnet = make_seismic_testnet(i)
    w3 = testnet.wallet_client(pk)

    try:
        block = w3.eth.get_block("latest")
        print(f"GCP-{i}: Connected at block {block.number}")
    except Exception as e:
        print(f"GCP-{i}: Connection failed: {e}")
```

### Environment-Based Instance Selection

```python
import os
from seismic_web3 import make_seismic_testnet, PrivateKey

# Select instance from environment variable
instance_num = int(os.getenv("TESTNET_INSTANCE", "1"))
testnet = make_seismic_testnet(instance_num)

pk = PrivateKey(os.environ["PRIVATE_KEY"])
w3 = testnet.wallet_client(pk)

print(f"Connected to {testnet.name}")
```

### Comparing Instance States

```python
from seismic_web3 import make_seismic_testnet

async def compare_instances():
    """Compare block heights across testnet instances."""
    instances = [1, 2, 3]

    for i in instances:
        testnet = make_seismic_testnet(i)
        public = await testnet.async_public_client()

        block_number = await public.eth.block_number
        print(f"GCP-{i}: block {block_number}")

# Run comparison
import asyncio
asyncio.run(compare_instances())
```

## Use Cases

### Load Balancing

Distribute requests across multiple testnet instances:

```python
from seismic_web3 import make_seismic_testnet, PrivateKey
import random

def get_balanced_client(private_key: PrivateKey):
    """Get a client connected to a random testnet instance."""
    instance = random.choice([1, 2, 3])
    testnet = make_seismic_testnet(instance)
    return testnet.wallet_client(private_key)
```

### Failover Strategy

Automatically switch to backup instance on failure:

```python
from seismic_web3 import make_seismic_testnet, PrivateKey

async def get_client_with_failover(private_key: PrivateKey):
    """Try connecting to testnet instances in order."""
    for i in [1, 2, 3]:
        testnet = make_seismic_testnet(i)
        w3 = await testnet.async_wallet_client(private_key)

        try:
            # Test connection
            await w3.eth.block_number
            print(f"Connected to GCP-{i}")
            return w3
        except Exception as e:
            print(f"GCP-{i} failed: {e}, trying next...")

    raise ConnectionError("All testnet instances unavailable")
```

## Notes

* All testnet instances share the same chain ID (`5124`)
* Instances may have different states and block heights
* Useful for load distribution and redundancy
* Instance 1 is the primary/default instance (same as [`SEISMIC_TESTNET`](seismic-testnet.md))
* All instances follow the same protocol and API

## Relationship to SEISMIC\_TESTNET

The default [`SEISMIC_TESTNET`](seismic-testnet.md) constant is defined as:

```python
SEISMIC_TESTNET: ChainConfig = make_seismic_testnet(1)
```

So these are equivalent:

```python
from seismic_web3 import SEISMIC_TESTNET, make_seismic_testnet

testnet_a = SEISMIC_TESTNET
testnet_b = make_seismic_testnet(1)

# Both have the same configuration
assert testnet_a.rpc_url == testnet_b.rpc_url
assert testnet_a.chain_id == testnet_b.chain_id
```

## See Also

* [ChainConfig](chain-config.md) - Chain configuration dataclass
* [SEISMIC\_TESTNET](seismic-testnet.md) - Pre-defined testnet (GCP-1)
* [create\_wallet\_client](../client/create-wallet-client.md) - Direct client creation
* [PrivateKey](../api-reference/bytes32/private-key.md) - Private key type
