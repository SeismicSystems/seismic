---
description: Result from debug shielded write
icon: bug
---

# DebugWriteResult

Result from a debug shielded write (`.dwrite` namespace). The transaction **is** broadcast (like `.write`), but you also get both plaintext and encrypted views for inspection.

## Definition

```python
@dataclass(frozen=True)
class DebugWriteResult:
    plaintext_tx: PlaintextTx
    shielded_tx: UnsignedSeismicTx
    tx_hash: HexBytes
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `plaintext_tx` | [`PlaintextTx`](plaintext-tx.md) | Transaction with unencrypted calldata |
| `shielded_tx` | [`UnsignedSeismicTx`](unsigned-seismic-tx.md) | Full unsigned TxSeismic with encrypted calldata |
| `tx_hash` | `HexBytes` | Transaction hash from `eth_sendRawTransaction` |

## Example

```python
result = await contract.dwrite.transfer(recipient, 1000)

print(f"Tx hash: {result.tx_hash.to_0x_hex()}")
print(f"Plaintext data: {result.plaintext_tx.data.to_0x_hex()}")
print(f"Encrypted data: {result.shielded_tx.data.to_0x_hex()}")
```

## Notes

- `.dwrite` broadcasts a real transaction — it consumes gas and changes state
- Use `.write` in production; `.dwrite` is for debugging only
- The `shielded_tx` field is unsigned (the SDK adds the signature before broadcast)

## See Also

- [PlaintextTx](plaintext-tx.md) — plaintext transaction view
- [UnsignedSeismicTx](unsigned-seismic-tx.md) — shielded transaction structure
