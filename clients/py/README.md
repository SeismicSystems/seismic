# seismic-web3

Python SDK for [Seismic](https://seismic.systems), built on [web3.py](https://github.com/ethereum/web3.py). Requires **Python 3.10+**.

```bash
pip install seismic-web3
```

## Quick start

```python
from seismic_web3 import create_shielded_web3, PrivateKey

w3 = create_shielded_web3(
    "http://127.0.0.1:8545",
    private_key=PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX")),
)

contract = w3.seismic.contract(address="0x...", abi=ABI)

# Shielded write — calldata is encrypted (TxSeismic type 0x4a)
tx_hash = contract.write.setNumber(42)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Shielded read — signed, encrypted eth_call
result = contract.read.getNumber()
```

`ShieldedContract` exposes five namespaces:

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
