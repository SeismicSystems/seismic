---
description: Deposit contract address and ABI constant
icon: file-code
---

# Deposit Contract

## Constants

```python
DEPOSIT_CONTRACT_ADDRESS: str = "0x00000000219ab540356cBB839Cbe05303d7705Fa"
DEPOSIT_CONTRACT_ABI: list[dict[str, Any]]
```

## ABI entries

Functions:

- `deposit(bytes,bytes,bytes,bytes,bytes,bytes32)` (`payable`)
- `get_deposit_count()` (`view`)
- `get_deposit_root()` (`view`)
- `supportsInterface(bytes4)` (`pure`)

Events:

- `DepositEvent`

## Namespace usage example

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey.from_hex_str("0x...")
w3 = SEISMIC_TESTNET.wallet_client(pk)

count = w3.seismic.get_deposit_count()
root = w3.seismic.get_deposit_root()
```

## Related docs

- [make_withdrawal_credentials](make-withdrawal-credentials.md)
- [compute_deposit_data_root](compute-deposit-data-root.md)
