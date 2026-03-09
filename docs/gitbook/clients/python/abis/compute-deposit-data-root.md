---
description: Compute SSZ-style deposit data root for validator deposits
icon: calculator
---

# compute\_deposit\_data\_root

Compute the 32-byte deposit data root hash (SHA-256 SSZ hash tree root) for validator deposits. This value must be passed as the `deposit_data_root` parameter when calling `deposit()`.

## Signature

```python
def compute_deposit_data_root(
    *,
    node_pubkey: bytes,
    consensus_pubkey: bytes,
    withdrawal_credentials: bytes,
    node_signature: bytes,
    consensus_signature: bytes,
    amount_gwei: int,
) -> bytes
```

All parameters are keyword-only.

## Parameters

| Parameter | Type | Bytes | Description |
| --- | --- | --- | --- |
| `node_pubkey` | `bytes` | 32 | ED25519 public key for node identity |
| `consensus_pubkey` | `bytes` | 48 | BLS12-381 public key for consensus |
| `withdrawal_credentials` | `bytes` | 32 | Use [`make_withdrawal_credentials`](make-withdrawal-credentials.md) |
| `node_signature` | `bytes` | 64 | ED25519 signature over deposit data |
| `consensus_signature` | `bytes` | 96 | BLS12-381 signature over deposit data |
| `amount_gwei` | `int` | — | Deposit amount in gwei (e.g. `32 * 1000000000`) |

## Returns

`bytes` — 32-byte deposit data root hash.

Raises `ValueError` if any byte-length parameter has the wrong size.

## How It Works

The function computes the SSZ hash tree root, mirroring the on-chain verification in `DepositContract.sol`:

```
pubkey_root         = SHA256(node_pubkey + SHA256(consensus_pubkey + pad_16))
signature_root      = SHA256(SHA256(node_sig) + SHA256(SHA256(con_sig[:64]) + SHA256(con_sig[64:] + pad_32)))
deposit_data_root   = SHA256(SHA256(pubkey_root + withdrawal_credentials) + SHA256(amount_le8 + pad_24 + signature_root))
```

Where `amount_le8` is the gwei value encoded as 8-byte little-endian, and `pad_N` extends data to chunk boundaries.

## Example

```python
from seismic_web3 import compute_deposit_data_root, make_withdrawal_credentials

withdrawal_credentials = make_withdrawal_credentials(
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
)

root = compute_deposit_data_root(
    node_pubkey=bytes(32),
    consensus_pubkey=bytes(48),
    withdrawal_credentials=withdrawal_credentials,
    node_signature=bytes(64),
    consensus_signature=bytes(96),
    amount_gwei=32 * 1000000000,
)

print(root.hex())  # 32-byte hex string
```

## See Also

- [make\_withdrawal\_credentials](make-withdrawal-credentials.md) — Build the `withdrawal_credentials` input
- [Deposit Contract](deposit-contract.md) — ABI, address, and deposit requirements
- [deposit](../namespaces/methods/deposit.md) — Namespace method for making deposits
