---
description: Practical workflows for shielded writes and signed reads
icon: book
---

# Guides

These guides cover the two core private interaction patterns in the Python SDK.

- [Shielded Write](shielded-write.md)
- [Signed Reads](signed-reads.md)

## Before You Start

You need a wallet client (`w3 = SEISMIC_TESTNET.wallet_client(pk)` or async equivalent), because both flows require signing and encryption state.

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)
```

For runnable end-to-end snippets, see [Examples](../examples/).
