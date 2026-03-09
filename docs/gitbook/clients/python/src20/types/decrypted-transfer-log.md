---
description: Decoded SRC20 Transfer event with decrypted amount
---

# DecryptedTransferLog

Frozen dataclass representing a decoded SRC20 Transfer event with decrypted amount.

## Solidity Event

```solidity
event Transfer(
    address indexed from,
    address indexed to,
    bytes32 indexed encryptKeyHash,
    bytes encryptedAmount
)
```

## Definition

```python
@dataclass(frozen=True)
class DecryptedTransferLog:
    from_address: ChecksumAddress
    to_address: ChecksumAddress
    encrypt_key_hash: bytes
    encrypted_amount: bytes
    decrypted_amount: int
    transaction_hash: HexBytes
    block_number: int
```

## Fields

| Field | Type | Description |
| --- | --- | --- |
| `from_address` | `ChecksumAddress` | Sender address |
| `to_address` | `ChecksumAddress` | Recipient address |
| `encrypt_key_hash` | `bytes` | 32-byte `keccak256` of viewing key |
| `encrypted_amount` | `bytes` | Raw AES-GCM ciphertext |
| `decrypted_amount` | `int` | Decrypted token amount |
| `transaction_hash` | `HexBytes` | Transaction hash |
| `block_number` | `int` | Block number |

## Example

```python
from seismic_web3.src20 import DecryptedTransferLog

def on_transfer(log: DecryptedTransferLog):
    print(f"Transfer from {log.from_address} to {log.to_address}: {log.decrypted_amount}")
    print(f"Block {log.block_number}, tx {log.transaction_hash.hex()}")
```

## See Also

- [DecryptedApprovalLog](decrypted-approval-log.md) — Approval event data
- [Event Watching](../event-watching/) — Watch and decrypt events
