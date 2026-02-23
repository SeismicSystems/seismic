---
description: Async client, writes, reads, and watcher usage
icon: rotate-cw
---

# Async Patterns

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI
from seismic_web3.src20 import async_watch_src20_events

async def main() -> None:
    pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
    w3 = await SEISMIC_TESTNET.async_wallet_client(pk)

    token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)

    tx_hash = await token.write.transfer("0xRecipient", 1_000)
    print(tx_hash.hex())

    raw = await token.read.balanceOf()
    print(raw.hex())

    watcher = await async_watch_src20_events(
        w3,
        encryption=w3.seismic.encryption,
        private_key=pk,
        token_address="0xYourTokenAddress",
        on_transfer=lambda log: print(log.decrypted_amount),
    )

    # ... later
    # await watcher.stop()
```
