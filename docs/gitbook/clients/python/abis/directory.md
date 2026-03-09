---
description: Directory contract ABI and address for viewing key management
icon: address-book
---

# Directory

ABI and address for Seismic's Directory genesis contract at `0x1000000000000000000000000000000000000004`. Stores per-user AES-256 viewing keys using shielded storage (`suint256`).

## Constants

```python
from seismic_web3 import DIRECTORY_ABI, DIRECTORY_ADDRESS

DIRECTORY_ADDRESS: str = "0x1000000000000000000000000000000000000004"
DIRECTORY_ABI: list[dict[str, Any]]
```

## Functions

| Function | Parameters | Returns | Mutability | Description |
| --- | --- | --- | --- | --- |
| `checkHasKey` | `_addr: address` | `bool` | `view` | Check if address has a viewing key |
| `keyHash` | `to: address` | `bytes32` | `view` | Get keccak256 hash of address's key |
| `getKey` | — | `uint256` | `view` | Get caller's viewing key (shielded read) |
| `setKey` | `_key: suint256` | — | `nonpayable` | Register or update viewing key (shielded write) |

## Notes

- `checkHasKey` and `keyHash` are public reads — no authentication needed
- `getKey` returns the caller's own key and requires a signed read (uses `msg.sender`)
- `setKey` uses a shielded type (`suint256`) so the key value is encrypted in the transaction
- Only the 4 functions needed by the Python SDK are included; the full contract may have additional functions

## See Also

- [Intelligence Providers](../src20/intelligence-providers/) — Higher-level viewing key management (register, fetch, query)
- [SRC20\_ABI](src20-abi.md) — Token standard that uses Directory keys
- [Event Watching](../src20/event-watching/) — Decrypt events using viewing keys
