---
description: Complete unsigned Seismic transaction
icon: file-contract
---

# UnsignedSeismicTx

All fields of a `TxSeismic` (type `0x4a`) transaction before signing.

## Definition

```python
@dataclass(frozen=True)
class UnsignedSeismicTx:
    chain_id: int
    nonce: int
    gas_price: int
    gas: int
    to: ChecksumAddress | None
    value: int
    data: HexBytes
    seismic: SeismicElements
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `chain_id` | `int` | Numeric chain identifier |
| `nonce` | `int` | Sender's transaction count |
| `gas_price` | `int` | Gas price in wei |
| `gas` | `int` | Gas limit |
| `to` | `ChecksumAddress \| None` | Recipient address, or `None` for contract creation |
| `value` | `int` | Amount of wei to transfer |
| `data` | `HexBytes` | **Encrypted** calldata (ciphertext) |
| `seismic` | [`SeismicElements`](seismic-elements.md) | Seismic-specific encryption and expiry fields |

## Example

```python
from seismic_web3 import sign_seismic_tx_eip712, PrivateKey

unsigned_tx = UnsignedSeismicTx(...)

signed_tx = sign_seismic_tx_eip712(unsigned_tx, PrivateKey(...))
tx_hash = w3.eth.send_raw_transaction(signed_tx)
```

## Notes

- The `data` field contains encrypted calldata — plaintext is not recoverable without the TEE's private key
- Created automatically by the SDK's write methods; visible in [`DebugWriteResult`](debug-write-result.md) for inspection
- Compatible with both raw signing (`message_version=0`) and EIP-712 (`message_version=2`)

## See Also

- [SeismicElements](seismic-elements.md) — Seismic-specific fields
- [sign\_seismic\_tx\_eip712](../eip712/sign-seismic-tx-eip712.md) — sign this transaction
- [DebugWriteResult](debug-write-result.md) — contains UnsignedSeismicTx
- [build\_seismic\_typed\_data](../eip712/build-seismic-typed-data.md) — convert to EIP-712 dict
