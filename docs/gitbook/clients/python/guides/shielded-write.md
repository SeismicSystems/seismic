---
description: Send encrypted contract transactions
icon: send
---

# Shielded Write

Shielded writes encrypt calldata before broadcasting a `TxSeismic` transaction.

## Recommended path: contract namespace

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract(
    "0xYourTokenAddress",
    SRC20_ABI,
)

tx_hash = token.write.transfer("0xRecipient", 1_000)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
```

## Optional controls

All shielded write calls support:

- `value`
- `gas`
- `gas_price`
- `security` (`SeismicSecurityParams`)

Example:

```python
tx_hash = token.write.transfer(
    "0xRecipient",
    1_000,
    gas=250_000,
)
```

## EIP-712 signing mode

Enable typed-data signing by creating the contract with `eip712=True`:

```python
token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI, eip712=True)
```

## Debug write during development

Use `.dwrite` to broadcast and inspect plaintext/encrypted views:

```python
result = token.dwrite.transfer("0xRecipient", 1_000)
print(result.plaintext_tx)
print(result.shielded_tx)
print(result.tx_hash.hex())
```
