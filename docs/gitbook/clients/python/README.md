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
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))

# Wallet client — full capabilities (requires private key)
w3 = SEISMIC_TESTNET.wallet_client(pk)

contract = w3.seismic.contract(address="0x...", abi=ABI)

# Shielded write — calldata is encrypted
tx_hash = contract.write.setNumber(42)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Signed read — encrypted eth_call, proves your identity
result = contract.read.getNumber()
```

```python
# Public client — read-only (no private key needed)
public = SEISMIC_TESTNET.public_client()

contract = public.seismic.contract(address="0x...", abi=ABI)
result = contract.tread.getNumber()
```
