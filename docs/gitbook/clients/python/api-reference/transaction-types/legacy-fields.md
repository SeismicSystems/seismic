---
description: Standard EVM transaction fields
icon: list
---

# LegacyFields

Standard EVM transaction fields used in [`TxSeismicMetadata`](tx-seismic-metadata.md) for AAD (Additional Authenticated Data) construction.

## Definition

```python
@dataclass(frozen=True)
class LegacyFields:
    chain_id: int
    nonce: int
    to: ChecksumAddress | None
    value: int
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `chain_id` | `int` | Numeric chain identifier (e.g., `5124` for testnet) |
| `nonce` | `int` | Sender's transaction count |
| `to` | `ChecksumAddress \| None` | Recipient address, or `None` for contract creation |
| `value` | `int` | Amount of wei to transfer |

## Example

```python
from seismic_web3 import LegacyFields

legacy = LegacyFields(
    chain_id=5124,
    nonce=42,
    to="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    value=1 * 10**18,  # 1 ETH in wei
)
```

## Notes

- Called "legacy" because these fields are unchanged from standard Ethereum transactions — present in every tx type since pre-EIP-2718
- Does not include `gas`, `gasPrice`, or `data` — those are in [`UnsignedSeismicTx`](unsigned-seismic-tx.md)
- Part of the AAD for AES-GCM encryption, ensuring ciphertext is bound to transaction context

## See Also

- [TxSeismicMetadata](tx-seismic-metadata.md) — uses LegacyFields
- [SeismicElements](seismic-elements.md) — Seismic-specific fields (contrast)
- [UnsignedSeismicTx](unsigned-seismic-tx.md) — full transaction structure
