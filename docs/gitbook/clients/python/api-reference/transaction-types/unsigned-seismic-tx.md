---
description: Unsigned TxSeismic dataclass
icon: file-code
---

# UnsignedSeismicTx

Full transaction fields before signing.

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

## Important

`data` is encrypted calldata in standard shielded flows.

## Example

```python
from seismic_web3 import sign_seismic_tx_eip712

# signed = sign_seismic_tx_eip712(unsigned_tx, private_key)
```
