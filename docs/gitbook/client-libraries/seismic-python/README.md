---
description: >-
  Python client library for Seismic, extending web3.py with shielded
  transactions, signed reads, and Mercury EVM precompiles.
icon: snake
---

# Seismic Python

The Seismic Python SDK (`seismic-web3`) extends [web3.py](https://web3py.readthedocs.io/) with a `w3.seismic` namespace that handles encrypted transactions, signed reads, and access to Mercury EVM precompiles. It supports both synchronous and asynchronous usage.

## Features

- **Shielded transactions** -- Encrypt calldata with the TEE public key via `TxSeismic` (type `0x4A`)
- **Signed reads** -- Prove identity in `eth_call` so contracts can gate reads on `msg.sender`
- **Contract wrappers** -- Five namespaces (`.write`, `.read`, `.twrite`, `.tread`, `.dwrite`) for shielded and transparent operations
- **Precompiles** -- Access RNG, ECDH, AES-GCM, HKDF, and secp256k1 signing on-chain
- **SRC20 support** -- Built-in ABI for the shielded token standard
- **Async/await** -- Every operation has an async variant

## Quick start

```bash
pip install seismic-web3
```

```python
from seismic_web3 import create_wallet_client
from seismic_web3.chains import SEISMIC_TESTNET
from eth_account import Account

w3 = create_wallet_client(
    chain=SEISMIC_TESTNET,
    private_key=Account.from_key("0x...").key,
)

# Shielded write
tx_hash = contract.write.transfer(to, amount)

# Signed read (proves msg.sender)
balance = contract.read.balanceOf()

# Transparent read (no identity)
name = contract.tread.name()
```

The wallet client automatically fetches the TEE public key, derives an AES-GCM key via ECDH, and encrypts calldata for all shielded operations.
