---
description: Send encrypted TxSeismic transaction
icon: send
---

# send_shielded_transaction

Build, encrypt, sign, and broadcast a shielded transaction.

## Signatures

```python
# sync
w3.seismic.send_shielded_transaction(
    *,
    to: ChecksumAddress,
    data: HexBytes,
    value: int = 0,
    gas: int | None = None,
    gas_price: int | None = None,
    security: SeismicSecurityParams | None = None,
    eip712: bool = False,
) -> HexBytes

# async
await w3.seismic.send_shielded_transaction(...same args...) -> HexBytes
```

## Behavior

- `data` is plaintext calldata; SDK encrypts it before signing.
- Default gas is `30_000_000` when `gas` is not provided.
- `gas_price` is fetched from chain when omitted.
- `eip712=True` switches signing hash path to EIP-712.

## Returns

Transaction hash (`HexBytes`).
