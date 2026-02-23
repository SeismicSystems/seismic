---
description: Async wallet usage, concurrent reads, writes, and event watching
icon: rotate-cw
---

# Async Patterns

```python
import os
import asyncio
from eth_abi import decode
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI
from seismic_web3.src20 import async_watch_src20_events

async def main() -> None:
    pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
    w3 = await SEISMIC_TESTNET.async_wallet_client(pk)

    token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)

    # concurrent public reads
    chain_id_task = w3.eth.chain_id
    block_task = w3.eth.block_number
    tee_task = w3.seismic.get_tee_public_key()
    chain_id, block_number, tee_key = await asyncio.gather(
        chain_id_task,
        block_task,
        tee_task,
    )
    print(chain_id, block_number, tee_key.hex())

    # shielded write
    tx_hash = await token.write.transfer("0xRecipientAddress", 100)
    receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
    print(receipt.status)

    # signed read + decode
    raw = await token.read.balanceOf()
    balance = decode(["uint256"], bytes(raw))[0]
    print(balance)

    # async watcher
    watcher = await async_watch_src20_events(
        w3,
        encryption=w3.seismic.encryption,
        private_key=pk,
        token_address="0xYourTokenAddress",
        on_transfer=lambda log: print("transfer", log.decrypted_amount),
    )

    await asyncio.sleep(10)
    await watcher.stop()

asyncio.run(main())
```
