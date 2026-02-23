---
description: Encode calldata for ABI entries with Seismic shielded types
icon: brackets
---

# encode_shielded_calldata

Encode calldata for contract ABIs that include Seismic shielded types like `suint256`.

## Signature

```python
def encode_shielded_calldata(
    abi: list[dict[str, Any]],
    function_name: str,
    args: list[Any],
) -> HexBytes
```

## Behavior

- Selector is computed from original ABI types (`suint*`, `sbool`, etc.).
- Parameter encoding remaps shielded types to standard Solidity ABI types (`uint*`, `bool`, etc.).
- Raises `ValueError` if `function_name` is not found.

## Usage

This is the same helper used by `contract.*` namespaces and deposit/public helper methods.
