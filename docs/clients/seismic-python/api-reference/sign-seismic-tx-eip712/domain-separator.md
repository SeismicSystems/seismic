---
description: Compute EIP-712 domain separator
icon: network-wired
---

# domain\_separator

Compute the EIP-712 domain separator for a given chain ID.

## Overview

`domain_separator()` computes the EIP-712 domain separator, which binds typed data to a specific chain and verifying contract. This prevents cross-chain replay attacks and ensures signatures are only valid in their intended context.

## Signature

```python
def domain_separator(chain_id: int) -> bytes
```

## Parameters

| Parameter  | Type  | Required | Description                                               |
| ---------- | ----- | -------- | --------------------------------------------------------- |
| `chain_id` | `int` | Yes      | Numeric chain identifier (e.g., 5124 for Seismic testnet) |

## Returns

| Type    | Description                                    |
| ------- | ---------------------------------------------- |
| `bytes` | 32-byte keccak256 hash of the domain separator |

## Examples

### Basic Usage

```python
from seismic_web3 import domain_separator, SEISMIC_TESTNET

# Seismic testnet
testnet_domain = domain_separator(SEISMIC_TESTNET.chain_id)
print(f"Testnet domain: {testnet_domain.hex()}")
```

### Compare Across Chains

```python
from seismic_web3 import domain_separator, SEISMIC_TESTNET, SANVIL

# Different chains have different domain separators
testnet = domain_separator(SEISMIC_TESTNET.chain_id)
sanvil = domain_separator(SANVIL.chain_id)

assert testnet != sanvil
```

### Use in Custom Signing

```python
from seismic_web3 import domain_separator, struct_hash
from eth_hash.auto import keccak

unsigned_tx = UnsignedSeismicTx(...)

# Compute components
domain = domain_separator(unsigned_tx.chain_id)
struct = struct_hash(unsigned_tx)

# Build signing hash
signing_hash = keccak(b"\x19\x01" + domain + struct)
```

### Verify Chain-Specific

```python
from seismic_web3 import domain_separator, SEISMIC_TESTNET

# Get domain for Seismic testnet
testnet_domain = domain_separator(SEISMIC_TESTNET.chain_id)

# Signatures with this domain only valid on Seismic testnet
print(f"Seismic testnet domain: {testnet_domain.hex()}")
```

## How It Works

The function computes:

```
keccak256(
    typeHash(EIP712Domain)
    ‖ keccak256("Seismic Transaction")
    ‖ keccak256("2")
    ‖ abi.encode(uint256, chainId)
    ‖ abi.encode(address, 0x0…0)
)
```

Where:

* **Type hash** - Hash of `EIP712Domain` struct type string
* **Name** - `"Seismic Transaction"` (domain name)
* **Version** - `"2"` (matches `TYPED_DATA_MESSAGE_VERSION`)
* **Chain ID** - Numeric chain identifier
* **Verifying contract** - `0x0000000000000000000000000000000000000000` (signing is off-chain)

## Domain Fields

The domain separator encodes:

| Field               | Value                   | Description                      |
| ------------------- | ----------------------- | -------------------------------- |
| `name`              | `"Seismic Transaction"` | Domain name                      |
| `version`           | `"2"`                   | Message version                  |
| `chainId`           | `chain_id` parameter    | Chain identifier                 |
| `verifyingContract` | `0x0...0`               | Zero address (off-chain signing) |

## Implementation

```python
def domain_separator(chain_id: int) -> bytes:
    return keccak(
        EIP712_DOMAIN_TYPE_HASH +
        _DOMAIN_NAME_HASH +      # keccak("Seismic Transaction")
        _DOMAIN_VERSION_HASH +   # keccak("2")
        _pad32_int(chain_id) +
        _pad32_address(VERIFYING_CONTRACT)  # 0x0...0
    )
```

## Purpose

The domain separator ensures:

* **Chain isolation** - Signatures for chain A won't work on chain B
* **Contract binding** - Signatures bound to specific verifying contract
* **Replay prevention** - Can't reuse signatures across different contexts

## Notes

* Used in [`eip712_signing_hash`](eip712-signing-hash.md)
* Always 32 bytes (keccak256 output)
* Same for all transactions on the same chain
* Can be pre-computed and cached per chain

## Performance Optimization

```python
from seismic_web3 import domain_separator, SEISMIC_TESTNET

# Cache domain separator per chain
_domain_cache = {}

def get_cached_domain(chain_id: int) -> bytes:
    if chain_id not in _domain_cache:
        _domain_cache[chain_id] = domain_separator(chain_id)
    return _domain_cache[chain_id]

# Use cached version
domain = get_cached_domain(SEISMIC_TESTNET.chain_id)
```

## Verifying Contract

The verifying contract is set to `0x0000000000000000000000000000000000000000` because:

* Signing happens **off-chain** via RPC
* No on-chain contract verifies the signature
* The Seismic node verifies the signature directly

## See Also

* [eip712\_signing\_hash](eip712-signing-hash.md) - Uses domain separator
* [struct\_hash](struct-hash.md) - The other component of signing hash
* [build\_seismic\_typed\_data](build-seismic-typed-data.md) - Includes domain in typed data
* [sign\_seismic\_tx\_eip712](./) - Full signing process
