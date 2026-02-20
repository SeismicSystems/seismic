---
description: Optional security parameters for shielded transactions
icon: shield-halved
---

# SeismicSecurityParams

Optional security parameters for customizing shielded transaction behavior.

## Overview

`SeismicSecurityParams` allows you to override the SDK's default security parameters when creating shielded transactions. All fields default to `None`, meaning the SDK will use sensible defaults.

## Definition

```python
@dataclass(frozen=True)
class SeismicSecurityParams:
    """Optional security parameters for shielded transactions.

    All fields default to None, meaning the SDK will use sensible
    defaults (e.g. fetch latest block, generate a random nonce, use a
    100-block expiry window).
    """
    blocks_window: int | None = None
    encryption_nonce: EncryptionNonce | None = None
    recent_block_hash: Bytes32 | None = None
    expires_at_block: int | None = None
```

## Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `blocks_window` | `int \| None` | `None` (→ 100) | Number of blocks before the transaction expires |
| `encryption_nonce` | [`EncryptionNonce`](../types/encryption-nonce.md) `\| None` | `None` (→ random) | Explicit AES-GCM nonce |
| `recent_block_hash` | [`Bytes32`](../types/bytes32.md) `\| None` | `None` (→ latest) | Explicit block hash for freshness proof |
| `expires_at_block` | `int \| None` | `None` (→ computed) | Explicit expiry block number |

## Default Behavior

When fields are `None`:
- **`blocks_window`** → `100` blocks
- **`encryption_nonce`** → Cryptographically random 12 bytes
- **`recent_block_hash`** → Fetched from latest block
- **`expires_at_block`** → `recent_block_number + blocks_window`

## Examples

### Use Default Values

```python
# No security params needed for defaults
result = await contract.write.transfer(recipient, amount)
```

### Custom Expiry Window

```python
from seismic_web3 import SeismicSecurityParams

# Transaction expires in 200 blocks instead of 100
security_params = SeismicSecurityParams(blocks_window=200)

result = await contract.write.transfer(
    recipient,
    amount,
    security_params=security_params,
)
```

### Explicit Nonce (Testing)

```python
from seismic_web3 import SeismicSecurityParams, EncryptionNonce

# Use specific nonce (for testing/debugging)
nonce = EncryptionNonce("0x123456789012345678901234")
security_params = SeismicSecurityParams(encryption_nonce=nonce)

result = await contract.write.transfer(
    recipient,
    amount,
    security_params=security_params,
)
```

### Explicit Block Hash and Expiry

```python
from seismic_web3 import SeismicSecurityParams, Bytes32

# Use specific block hash and expiry
security_params = SeismicSecurityParams(
    recent_block_hash=Bytes32("0x1234..."),
    expires_at_block=12345700,
)

result = await contract.write.transfer(
    recipient,
    amount,
    security_params=security_params,
)
```

### Combine Multiple Overrides

```python
from seismic_web3 import SeismicSecurityParams, EncryptionNonce, Bytes32

security_params = SeismicSecurityParams(
    blocks_window=150,
    encryption_nonce=EncryptionNonce(...),
    recent_block_hash=Bytes32(...),
)

result = await contract.write.transfer(
    recipient,
    amount,
    security_params=security_params,
)
```

## Use Cases

### Longer Expiry for Slow Networks

```python
# Give more time for transaction inclusion
security_params = SeismicSecurityParams(blocks_window=300)
```

### Deterministic Nonces (Testing)

```python
# Use predictable nonce for testing
security_params = SeismicSecurityParams(
    encryption_nonce=EncryptionNonce(b'\x00' * 12)
)
```

### Batch Transactions with Same Block

```python
# Use same recent_block_hash for multiple transactions
block_hash = Bytes32(w3.eth.get_block('latest')['hash'])
security_params = SeismicSecurityParams(recent_block_hash=block_hash)

# Send multiple transactions
await contract1.write.method1(..., security_params=security_params)
await contract2.write.method2(..., security_params=security_params)
```

## Properties

- **Immutable** - Cannot be modified after construction (`frozen=True`)
- **All optional** - Every field can be `None`
- **Type-safe** - Fields are validated at construction

## Field Relationships

### blocks_window vs expires_at_block

If both are provided:
- `expires_at_block` takes precedence
- `blocks_window` is ignored

If only `blocks_window` is provided:
- `expires_at_block = current_block_number + blocks_window`

If neither is provided:
- `blocks_window` defaults to 100
- `expires_at_block = current_block_number + 100`

### recent_block_hash and expires_at_block

These should be consistent:
- `recent_block_hash` proves the transaction is recent
- `expires_at_block` determines validity window
- SDK ensures consistency when auto-fetching

## Warnings

- **Don't reuse nonces in production** - Breaks AES-GCM security
- **Expiry too short** - Transaction may not be included in time
- **Expiry too long** - Increases replay window (though mitigated by block hash)
- **Stale block hash** - Transaction may be rejected if block hash is too old

## Notes

- Passed to contract write methods via `security_params` parameter
- Used to construct [`SeismicElements`](seismic-elements.md)
- Not used for signed reads (they have their own parameters)
- All `None` values are resolved before transaction construction

## See Also

- [SeismicElements](seismic-elements.md) - Constructed from SeismicSecurityParams
- [EncryptionNonce](../types/encryption-nonce.md) - Type for encryption_nonce field
- [Bytes32](../types/bytes32.md) - Type for recent_block_hash field
- [ShieldedContract](../../contract/shielded-contract.md) - Uses SeismicSecurityParams
- [Shielded Write Guide](../../guides/shielded-write.md) - Examples with security params
