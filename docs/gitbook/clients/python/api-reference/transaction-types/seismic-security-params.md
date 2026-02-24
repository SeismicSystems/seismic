---
description: Optional security parameters for shielded transactions
icon: shield-halved
---

# SeismicSecurityParams

Optional overrides for the SDK's default security parameters when creating shielded transactions. All fields default to `None`.

## Definition

```python
@dataclass(frozen=True)
class SeismicSecurityParams:
    blocks_window: int | None = None
    encryption_nonce: EncryptionNonce | None = None
    recent_block_hash: Bytes32 | None = None
    expires_at_block: int | None = None
```

## Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `blocks_window` | `int \| None` | `None` → 100 | Number of blocks before the tx expires |
| `encryption_nonce` | [`EncryptionNonce`](../types/encryption-nonce.md) `\| None` | `None` → random | Explicit AES-GCM nonce |
| `recent_block_hash` | [`Bytes32`](../types/bytes32.md) `\| None` | `None` → latest | Explicit block hash for freshness proof |
| `expires_at_block` | `int \| None` | `None` → computed | Explicit expiry block number |

## Example

```python
from seismic_web3 import SeismicSecurityParams

# Transaction expires in 200 blocks instead of default 100
security_params = SeismicSecurityParams(blocks_window=200)

result = await contract.write.transfer(
    recipient,
    amount,
    security_params=security_params,
)
```

## Field relationships

- If both `blocks_window` and `expires_at_block` are provided, `expires_at_block` takes precedence
- If neither is provided, defaults to `current_block + 100`

## See Also

- [SeismicElements](seismic-elements.md) — constructed from these params
- [EncryptionNonce](../types/encryption-nonce.md) — type for `encryption_nonce`
- [Bytes32](../types/bytes32.md) — type for `recent_block_hash`
