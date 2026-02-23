---
description: Create wallet/public clients (sync and async)
icon: wallet
---

# Basic Wallet Setup

## Sync clients

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

# Full wallet client
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Read-only public client
public = SEISMIC_TESTNET.public_client()

print(w3.eth.chain_id)
print(public.eth.block_number)
print(public.seismic.get_tee_public_key().hex())
```

## Async clients

```python
import os
import asyncio
from seismic_web3 import PrivateKey, SEISMIC_TESTNET

async def main() -> None:
    pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

    w3 = await SEISMIC_TESTNET.async_wallet_client(pk)
    public = await SEISMIC_TESTNET.async_public_client()

    print(await w3.eth.chain_id)
    print(await public.eth.block_number)
    print((await public.seismic.get_tee_public_key()).hex())

asyncio.run(main())
```
