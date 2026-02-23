---
description: Create wallet/public clients with chain configs
icon: wallet
---

# Basic Wallet Setup

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

# Full wallet client (sync)
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Public read-only client (sync)
public = SEISMIC_TESTNET.public_client()

# Access Seismic namespace
tee_key = public.seismic.get_tee_public_key()
print(tee_key.hex())
```

Async variants:

```python
# await SEISMIC_TESTNET.async_wallet_client(pk)
# await SEISMIC_TESTNET.async_public_client()
```
