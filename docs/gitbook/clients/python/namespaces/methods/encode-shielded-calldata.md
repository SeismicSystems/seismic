---
description: Encode calldata for ABI entries with Seismic shielded types
icon: code
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

- **Selector** is computed from original ABI type names (e.g. `setNumber(suint256)`) so it matches the on-chain contract
- **Parameter encoding** remaps shielded types to standard ABI types (e.g. `suint256` → `uint256`) because the values are structurally identical
- Raises `ValueError` if function is not found

## Example

```python
calldata = encode_shielded_calldata(SRC20_ABI, "transfer", ["0xRecipient", 100])
tx_hash = w3.seismic.send_shielded_transaction(to="0xToken", data=calldata)
```

## Notes

- For contract interactions, prefer `contract.write.functionName(...)` or `contract.read.functionName(...)` which call this internally
- Handles arrays (`suint256[]`, `suint256[5]`), `sbool`, `saddress`, and recursive tuple components

## See Also

- [send_shielded_transaction](send-shielded-transaction.md) — Send encrypted transactions using manually encoded calldata
- [signed_call](signed-call.md) — Execute encrypted reads using manually encoded calldata
- [Contract Namespaces](../../contract/namespaces/) — High-level API that handles encoding automatically
