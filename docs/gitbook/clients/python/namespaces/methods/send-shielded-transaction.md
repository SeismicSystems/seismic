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

- `data` is plaintext calldata; SDK encrypts it.
- Default gas is `30_000_000` when `gas` is omitted.
- If `gas_price` is omitted, SDK fetches current chain gas price.
- `eip712=True` uses typed-data signing hash path.

## Example

```python
from seismic_web3.contract.abi import encode_shielded_calldata

data = encode_shielded_calldata(SRC20_ABI, "transfer", ["0xRecipient", 100])
tx_hash = w3.seismic.send_shielded_transaction(to="0xTokenAddress", data=data)
```
