---
description: The `w3.seismic` namespace classes
icon: brackets-curly
---

# Namespaces

The SDK attaches a `seismic` object to each client.

## Public namespaces

- [SeismicPublicNamespace](seismic-public-namespace.md) (sync)
- [AsyncSeismicPublicNamespace](async-seismic-public-namespace.md) (async)

Public methods:

- `get_tee_public_key()`
- `get_deposit_root()`
- `get_deposit_count()`
- `contract(address, abi)` for read-only contract wrappers

## Wallet namespaces

- [SeismicNamespace](seismic-namespace.md) (sync)
- [AsyncSeismicNamespace](async-seismic-namespace.md) (async)

Wallet methods add:

- `send_shielded_transaction()`
- `debug_send_shielded_transaction()`
- `signed_call()`
- `deposit()`
- `contract(address, abi, eip712=False)` for shielded contract wrappers

## Quick example

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey.from_hex_str("0x...")
w3 = SEISMIC_TESTNET.wallet_client(pk)

tee_key = w3.seismic.get_tee_public_key()
print(tee_key.hex())
```
