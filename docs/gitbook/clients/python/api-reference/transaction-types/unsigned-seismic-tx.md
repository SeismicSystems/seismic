---
description: Unsigned TxSeismic dataclass
icon: file-code
---

# UnsignedSeismicTx

`UnsignedSeismicTx` contains every transaction field before signing.

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

## Important Detail

`data` is encrypted calldata in normal shielded flows (ciphertext, not plaintext).

## Usage

Used as input to signing/serialization helpers (raw and EIP-712 paths).
