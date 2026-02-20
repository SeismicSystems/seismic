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

pk = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"].removeprefix("0x")))

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

## Documentation

Full docs are hosted on GitBook: **[docs.seismic.systems/clients/python](https://docs.seismic.systems/clients/python)**

## Contributing

See [DEVELOPMENT.md](DEVELOPMENT.md) for local setup, running tests, and publishing.

---

> This SDK was entirely vibecoded.
