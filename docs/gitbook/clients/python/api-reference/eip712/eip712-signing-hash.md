---
description: Compute final EIP-712 signing digest for TxSeismic
icon: shield
---

# eip712_signing_hash

Compute the digest that is actually signed in EIP-712 mode.

## Signature

```python
def eip712_signing_hash(tx: UnsignedSeismicTx) -> bytes
```

## Formula

`keccak256("\\x19\\x01" || domain_separator(tx.chain_id) || struct_hash(tx))`

## Returns

32-byte signing digest.
