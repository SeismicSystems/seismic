---
description: Validator deposit contract ABI and address constants
icon: vault
---

# Deposit Contract

ABI and address for Seismic's Eth2-style validator deposit contract at `0x00000000219ab540356cBB839Cbe05303d7705Fa`. Validators deposit ETH along with their public keys, signatures, and withdrawal credentials.

## Constants

```python
from seismic_web3 import DEPOSIT_CONTRACT_ABI, DEPOSIT_CONTRACT_ADDRESS

DEPOSIT_CONTRACT_ADDRESS: str = "0x00000000219ab540356cBB839Cbe05303d7705Fa"
DEPOSIT_CONTRACT_ABI: list[dict[str, Any]]
```

## Functions

| Function | Parameters | Returns | Mutability | Description |
| --- | --- | --- | --- | --- |
| `deposit` | `node_pubkey`, `consensus_pubkey`, `withdrawal_credentials`, `node_signature`, `consensus_signature`, `deposit_data_root` | — | `payable` | Deposit ETH to become a validator |
| `get_deposit_count` | — | `bytes` | `view` | Total number of deposits |
| `get_deposit_root` | — | `bytes32` | `view` | Merkle root of all deposits |
| `supportsInterface` | `interfaceId: bytes4` | `bool` | `pure` | ERC-165 interface check |

## Events

| Event | Parameters | Description |
| --- | --- | --- |
| `DepositEvent` | `node_pubkey`, `consensus_pubkey`, `withdrawal_credentials`, `amount`, `node_signature`, `consensus_signature`, `index` | Emitted on successful deposit |

## Deposit Requirements

| Requirement | Details |
| --- | --- |
| Minimum deposit | 1 ETH (standard validator: 32 ETH) |
| `node_pubkey` | 32-byte ED25519 public key |
| `consensus_pubkey` | 48-byte BLS12-381 public key |
| `withdrawal_credentials` | 32 bytes — use [`make_withdrawal_credentials`](make-withdrawal-credentials.md) |
| `node_signature` | 64-byte ED25519 signature |
| `consensus_signature` | 96-byte BLS12-381 signature |
| `deposit_data_root` | 32 bytes — use [`compute_deposit_data_root`](compute-deposit-data-root.md) |
| Transaction value | Must match `amount_gwei * 10**9` in wei |

## Example

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Query deposit state via namespace methods
count = w3.seismic.get_deposit_count()
root = w3.seismic.get_deposit_root()
```

## See Also

- [compute\_deposit\_data\_root](compute-deposit-data-root.md) — Compute the SSZ hash tree root
- [make\_withdrawal\_credentials](make-withdrawal-credentials.md) — Build ETH1 withdrawal credentials
- [deposit](../namespaces/methods/deposit.md) — Namespace method for making deposits
