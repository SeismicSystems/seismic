---
description: Encode calldata for a shielded contract function
icon: code
---

# encode_shielded_calldata()

Encode calldata for a shielded contract function. The selector is computed from the **original** ABI (with shielded type names like `suint256`), while the parameters are encoded using **remapped** standard types (`uint256`).

This is useful when you need to construct calldata manually — for example, to pass to [`send_shielded_transaction()`](send-shielded-transaction.md) or [`signed_call()`](signed-call.md).

***

## Import

```python
from seismic_web3.contract.abi import encode_shielded_calldata
```

***

## Signature

```python
def encode_shielded_calldata(
    abi: list[dict[str, Any]],
    function_name: str,
    args: list[Any],
) -> HexBytes
```

***

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `abi` | `list[dict[str, Any]]` | The full contract ABI (list of function entries) |
| `function_name` | `str` | Name of the function to call |
| `args` | `list[Any]` | Positional arguments matching the function inputs |

***

## Returns

**Type:** `HexBytes`

Encoded calldata: 4-byte selector + ABI-encoded parameters.

***

## Raises

| Exception | When |
|-----------|------|
| `ValueError` | Function name not found in the ABI |

***

## Examples

### Basic Usage

```python
from seismic_web3.contract.abi import encode_shielded_calldata

abi = [
    {
        "name": "setNumber",
        "type": "function",
        "inputs": [{"name": "value", "type": "suint256"}],
    },
]

data = encode_shielded_calldata(abi, "setNumber", [42])
```

### With send_shielded_transaction

```python
from seismic_web3.contract.abi import encode_shielded_calldata

abi = [
    {
        "name": "transfer",
        "type": "function",
        "inputs": [
            {"name": "to", "type": "address"},
            {"name": "amount", "type": "suint256"},
        ],
    },
]

calldata = encode_shielded_calldata(
    abi,
    "transfer",
    ["0x1234567890123456789012345678901234567890", 1000],
)

tx_hash = w3.seismic.send_shielded_transaction(
    to=contract_address,
    data=calldata,
)
```

### With signed_call

```python
from seismic_web3.contract.abi import encode_shielded_calldata

abi = [
    {
        "name": "balanceOf",
        "type": "function",
        "inputs": [{"name": "account", "type": "address"}],
        "outputs": [{"name": "", "type": "uint256"}],
    },
]

calldata = encode_shielded_calldata(
    abi,
    "balanceOf",
    ["0x1234567890123456789012345678901234567890"],
)

result = w3.seismic.signed_call(
    to=contract_address,
    data=calldata,
)

if result:
    balance = int.from_bytes(result, byteorder='big')
    print(f"Balance: {balance}")
```

***

## How It Works

Seismic contracts use shielded types (`suint256`, `sbool`, `saddress`, etc.) that compile to `CLOAD`/`CSTORE` instead of `SLOAD`/`SSTORE`. These types need special handling for ABI encoding:

1. **Function selector** — computed from the **original** ABI signature (with `suint256`, `sbool`, etc.) so it matches the on-chain contract
2. **Parameter encoding** — uses **standard** Solidity types (`uint256`, `bool`, etc.) because the values are structurally identical

For example, a function `setNumber(suint256)` has a different selector than `setNumber(uint256)`, but the parameter is encoded as a standard `uint256`.

***

## See Also

- [send_shielded_transaction()](send-shielded-transaction.md) — Send encrypted transactions using manually encoded calldata
- [signed_call()](signed-call.md) — Execute encrypted reads using manually encoded calldata
- [Contract Namespaces](../../contract/namespaces/) — High-level API that handles encoding automatically
