---
description: Python SDK for Seismic, built on web3.py
icon: snake
---

# seismic-web3

Python SDK for [Seismic](https://seismic.systems), built on [web3.py](https://github.com/ethereum/web3.py). Requires **Python 3.10+**.

```bash
pip install seismic-web3
```

Or with [uv](https://docs.astral.sh/uv/):

```bash
uv add seismic-web3
```

***

### Quick example

```python
from seismic_web3 import create_shielded_web3, PrivateKey

w3 = create_shielded_web3(
    "http://127.0.0.1:8545",
    private_key=PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX")),
)

contract = w3.seismic.contract(address="0x...", abi=ABI)

# Shielded write — calldata is encrypted
tx_hash = contract.write.setNumber(42)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Signed read — encrypted eth_call, proves your identity
result = contract.read.getNumber()
```
