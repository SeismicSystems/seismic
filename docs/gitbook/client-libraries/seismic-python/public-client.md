---
icon: globe
---

# Public Client

The public client provides read-only operations. No private key is needed.

## Sync

```python
from seismic_web3 import create_public_client
from seismic_web3.chains import SEISMIC_TESTNET

w3 = create_public_client(chain=SEISMIC_TESTNET)
```

## Async

```python
from seismic_web3 import create_async_public_client
from seismic_web3.chains import SEISMIC_TESTNET

w3 = await create_async_public_client(chain=SEISMIC_TESTNET)
```

## Available methods

The `w3.seismic` namespace on a public client is limited to:

| Method                 | Purpose                                   |
| ---------------------- | ----------------------------------------- |
| `get_tee_public_key()` | Fetch the node's TEE public key           |
| `get_deposit_root()`   | Query the deposit contract merkle root    |
| `get_deposit_count()`  | Query the total deposit count             |
| `contract()`           | Create a contract wrapper (`.tread` only) |

Public clients cannot perform shielded writes, signed reads, or deposits.
