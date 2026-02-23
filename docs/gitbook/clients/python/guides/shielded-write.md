---
description: Send encrypted contract transactions
icon: send
---

# Shielded Write

Shielded writes encrypt calldata, build `TxSeismic` (`0x4a`), sign it, and broadcast it.

## Preferred flow: `contract.write`

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)

tx_hash = token.write.transfer("0xRecipient", 1_000)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
print(receipt.status)
```

## Transaction controls

All shielded write methods support:

- `value`
- `gas`
- `gas_price`
- `security` (`SeismicSecurityParams`)
- `eip712` (chosen when creating contract wrapper or calling namespace methods)

```python
from seismic_web3 import SeismicSecurityParams

params = SeismicSecurityParams(blocks_window=150)

tx_hash = token.write.transfer(
    "0xRecipient",
    1_000,
    gas=250_000,
    security=params,
)
```

## Low-level flow: `w3.seismic.send_shielded_transaction`

Use this when you already have calldata.

```python
from seismic_web3.contract.abi import encode_shielded_calldata

data = encode_shielded_calldata(SRC20_ABI, "transfer", ["0xRecipient", 1_000])

tx_hash = w3.seismic.send_shielded_transaction(
    to="0xYourTokenAddress",
    data=data,
)
```

## Debug write flow

`.dwrite` sends a real transaction and also returns plaintext/encrypted views.

```python
result = token.dwrite.transfer("0xRecipient", 1_000)

print(result.tx_hash.hex())
print(result.plaintext_tx.data.hex())
print(result.shielded_tx.data.hex())
```

## Async variant

```python
w3 = await SEISMIC_TESTNET.async_wallet_client(pk)
token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)

tx_hash = await token.write.transfer("0xRecipient", 1_000)
receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
```
