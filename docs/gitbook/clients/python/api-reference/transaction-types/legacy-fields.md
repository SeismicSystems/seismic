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
- Does not include `gas`, `gasPrice`, or `data`. These are the standard EVM fields that go into the AAD — `gas`/`gasPrice` are excluded because they aren't part of the authenticated context, and `data` is the plaintext being encrypted (so it can't also be AAD input)
- The AAD also includes [`SeismicElements`](seismic-elements.md) (encryption params, block hash, expiry). The node validates these fields before the transaction enters the mempool

## See Also

- [TxSeismicMetadata](tx-seismic-metadata.md) — uses LegacyFields
- [SeismicElements](seismic-elements.md) — Seismic-specific fields (contrast)
- [UnsignedSeismicTx](unsigned-seismic-tx.md) — full transaction structure
