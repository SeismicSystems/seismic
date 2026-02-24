---
description: Compute Seismic EIP-712 domain separator
icon: network-wired
---

# domain_separator

Compute the EIP-712 domain separator for Seismic transactions.

## Signature

```python
def domain_separator(chain_id: int) -> bytes
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `chain_id` | `int` | Yes | Numeric chain identifier |

## Returns

| Type | Description |
|------|-------------|
| `bytes` | 32-byte keccak256 hash |

## How it works

```text
keccak256(
    typeHash(EIP712Domain)
    ‖ keccak256("Seismic Transaction")
    ‖ keccak256("2")
    ‖ abi.encode(uint256, chainId)
    ‖ abi.encode(address, 0x0…0)
)
```

## Domain constants

- `name = "Seismic Transaction"`
- `version = str(TYPED_DATA_MESSAGE_VERSION)` (currently `"2"`)
- `verifyingContract = 0x0000000000000000000000000000000000000000` (signing is off-chain)

## Example

```python
from seismic_web3 import domain_separator

d = domain_separator(5124)
print(d.hex())
```

## See Also

- [eip712_signing_hash](eip712-signing-hash.md) — uses domain separator
- [struct_hash](struct-hash.md) — the other component of the signing hash
- [build_seismic_typed_data](build-seismic-typed-data.md) — includes domain in typed data
