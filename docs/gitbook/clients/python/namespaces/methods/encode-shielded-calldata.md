---
description: Encode calldata for ABI entries with Seismic shielded types
icon: brackets
---

# encode_shielded_calldata

Encode calldata for contract ABIs that include Seismic shielded types (`suint256`, `sbool`, `saddress`, etc.).

## Signature

```python
def encode_shielded_calldata(
    abi: list[dict[str, Any]],
    function_name: str,
    args: list[Any],
) -> HexBytes
```

## Behavior

- Selector is computed from original ABI type names (including shielded types).
- Parameter encoding remaps shielded types to standard ABI types.
- Raises `ValueError` if function is not found.

## Example

```python
calldata = encode_shielded_calldata(SRC20_ABI, "transfer", ["0xRecipient", 100])
tx_hash = w3.seismic.send_shielded_transaction(to="0xToken", data=calldata)
```
