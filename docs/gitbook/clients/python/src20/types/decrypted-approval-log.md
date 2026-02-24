---
description: Decoded SRC20 Approval event with decrypted amount
icon: check-circle
---

# DecryptedApprovalLog

Frozen dataclass representing a decoded SRC20 Approval event with decrypted amount.

## Solidity Event

```solidity
event Approval(
    address indexed owner,
    address indexed spender,
    bytes32 indexed encryptKeyHash,
    bytes encryptedAmount
)
```

## Definition

```python
@dataclass(frozen=True)
class DecryptedApprovalLog:
    owner: ChecksumAddress
    spender: ChecksumAddress
    encrypt_key_hash: bytes
    encrypted_amount: bytes
    decrypted_amount: int
    transaction_hash: HexBytes
    block_number: int
```

## Fields

| Field | Type | Description |
| --- | --- | --- |
| `owner` | `ChecksumAddress` | Address granting the approval |
| `spender` | `ChecksumAddress` | Address receiving the approval |
| `encrypt_key_hash` | `bytes` | 32-byte `keccak256` of viewing key |
| `encrypted_amount` | `bytes` | Raw AES-GCM ciphertext |
| `decrypted_amount` | `int` | Decrypted approval amount |
| `transaction_hash` | `HexBytes` | Transaction hash |
| `block_number` | `int` | Block number |

## Example

```python
from seismic_web3.src20 import DecryptedApprovalLog

def on_approval(log: DecryptedApprovalLog):
    print(f"Approval from {log.owner} to {log.spender}: {log.decrypted_amount}")
    print(f"Block {log.block_number}, tx {log.transaction_hash.hex()}")
```

## See Also

- [DecryptedTransferLog](decrypted-transfer-log.md) — Transfer event data
- [Event Watching](../event-watching/) — Watch and decrypt events
