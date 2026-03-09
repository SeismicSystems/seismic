---
description: Unencrypted transaction view for debugging
icon: eye
---

# PlaintextTx

Unencrypted transaction view returned by debug writes (`.dwrite`). Contains the same fields as a shielded transaction but with **plaintext** calldata (before AES-GCM encryption).

## Definition

```python
@dataclass(frozen=True)
class PlaintextTx:
    to: ChecksumAddress | None
    data: HexBytes
    nonce: int
    gas: int
    gas_price: int
    value: int
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `to` | `ChecksumAddress \| None` | Recipient address, or `None` for contract creation |
| `data` | `HexBytes` | **Plaintext** calldata (before encryption) |
| `nonce` | `int` | Sender's transaction count |
| `gas` | `int` | Gas limit |
| `gas_price` | `int` | Gas price in wei |
| `value` | `int` | Amount of wei to transfer |

## Example

```python
result = await contract.dwrite.transfer(recipient, 1000)

plaintext = result.plaintext_tx
print(f"To: {plaintext.to}")
print(f"Plaintext data: {plaintext.data.to_0x_hex()}")
print(f"Gas: {plaintext.gas}")
```

## Notes

- The `data` field is unencrypted — contrast with [`UnsignedSeismicTx.data`](unsigned-seismic-tx.md) which holds ciphertext
- Only available via `.dwrite`, not `.write`
- The broadcast transaction uses the encrypted data, not this plaintext

## See Also

- [DebugWriteResult](debug-write-result.md) — contains PlaintextTx
- [UnsignedSeismicTx](unsigned-seismic-tx.md) — shielded version with encrypted data
