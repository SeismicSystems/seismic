# seismic-web3

Python SDK for [Seismic](https://seismic.systems), built on [web3.py](https://github.com/ethereum/web3.py). Requires **Python 3.10+**.

```bash
pip install seismic-web3
```

## Client types

The SDK provides two client types:

- **Wallet client** — you provide a private key. Gives you full capabilities: shielded reads/writes, signed calls, deposits.
- **Public client** — no private key needed. Read-only access via transparent `eth_call`.

## Quick start

```python
import os
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

# Wallet client — full capabilities (requires private key)
w3 = SEISMIC_TESTNET.wallet_client(pk)

contract = w3.seismic.contract(address="0x...", abi=ABI)

# Shielded write — calldata is encrypted (TxSeismic type 0x4a)
tx_hash = contract.write.setNumber(42)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Shielded read — signed, encrypted eth_call
result = contract.read.getNumber()
```

```python
# Public client — read-only (no private key needed)
public = SEISMIC_TESTNET.public_client()

contract = public.seismic.contract(address="0x...", abi=ABI)
result = contract.tread.getNumber()
```

`ShieldedContract` (from the wallet client) exposes five namespaces:

| Namespace | What it does | On-chain visibility |
|-----------|-------------|-------------------|
| `.write` | Encrypted transaction (`TxSeismic` type `0x4a`) | Calldata hidden |
| `.read` | Encrypted signed `eth_call` | Calldata + result hidden |
| `.twrite` | Standard `eth_sendTransaction` | Calldata visible |
| `.tread` | Standard `eth_call` | Calldata visible |
| `.dwrite` | Debug write — returns plaintext + encrypted views | Calldata hidden |

Both sync and async clients are supported. See the full documentation for details.

## Transaction lookup responses

Keep using the existing `w3.eth.get_transaction`, `get_transaction_by_block`, and `get_block(..., full_transactions=True)` methods. The SDK exports `SeismicTransactionResponse`, `GasPaymentResponse`, and a narrowing helper:

```python
from seismic_web3 import is_seismic_transaction

transaction = w3.eth.get_transaction(tx_hash)
if is_seismic_transaction(transaction):
    payment = transaction["gasPayment"]
    if payment["type"] == "token":
        print(payment["token"])
```

The response and its payment remain web3.py `AttributeDict` mappings, **not** signing dataclasses. Standard Ethereum fields keep web3.py formatting; additional Seismic quantity fields keep their RPC hex strings. The helper does not mutate responses, checksum token addresses, or default missing selectors to Auto. It also works after awaiting async lookups and on full-block entries; ordinary Ethereum transactions and hash-only entries do not narrow.

The selector describes the signed preference, not which asset Auto ultimately used. See the [GasPayment guide](https://docs.seismic.systems/clients/python/api-reference/transaction-types/gas-payment) for response typing details.

## Documentation

Full docs are hosted on GitBook: **[docs.seismic.systems/clients/python](https://docs.seismic.systems/clients/python)**

## Contributing

See [DEVELOPMENT.md](DEVELOPMENT.md) for local setup, running tests, and publishing.

---

> This SDK was entirely vibecoded.
