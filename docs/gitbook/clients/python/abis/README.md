---
description: Built-in ABI constants and deposit helper functions
icon: file-code
---

# ABIs

The SDK exports protocol ABIs, genesis contract addresses, and deposit helper utilities.

## Constants

| Constant | Type | Description |
| --- | --- | --- |
| [`SRC20_ABI`](src20-abi.md) | `list[dict]` | SRC20 token interface (7 functions, 2 events) |
| [`DEPOSIT_CONTRACT_ABI`](deposit-contract.md) | `list[dict]` | Validator deposit contract (4 functions, 1 event) |
| [`DEPOSIT_CONTRACT_ADDRESS`](deposit-contract.md) | `str` | `0x00000000219ab540356cBB839Cbe05303d7705Fa` |
| [`DIRECTORY_ABI`](directory.md) | `list[dict]` | Viewing key directory (4 functions) |
| [`DIRECTORY_ADDRESS`](directory.md) | `str` | `0x1000000000000000000000000000000000000004` |

## Helper Functions

| Function | Returns | Description |
| --- | --- | --- |
| [`compute_deposit_data_root`](compute-deposit-data-root.md) | `bytes` | SHA-256 SSZ hash tree root for deposit data |
| [`make_withdrawal_credentials`](make-withdrawal-credentials.md) | `bytes` | 32-byte ETH1 withdrawal credentials from address |

## Example

```python
import os
from eth_abi import decode
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)
balance = decode(["uint256"], bytes(token.read.balanceOf()))[0]
```

## See Also

- [SRC20](../src20/) — SRC20 token usage guide
- [Contract](../contract/) — Contract interaction patterns
- [Namespaces](../namespaces/) — `w3.seismic` methods including `deposit`, `get_deposit_count`, `get_deposit_root`
