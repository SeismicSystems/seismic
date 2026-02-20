---
description: Compute EIP-712 domain separator
icon: network-wired
---

# domain_separator

Compute the EIP-712 domain separator for a given chain ID.

## Overview

`domain_separator()` computes the EIP-712 domain separator, which binds typed data to a specific chain and verifying contract. This prevents cross-chain replay attacks and ensures signatures are only valid in their intended context.

## Signature

```python
def domain_separator(chain_id: int) -> bytes
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `chain_id` | `int` | Yes | Numeric chain identifier (e.g., 1 for Ethereum mainnet) |

## Returns

| Type | Description |
|------|-------------|
| `bytes` | 32-byte keccak256 hash of the domain separator |

## Examples

### Basic Usage

```python
from seismic_web3 import domain_separator

# Ethereum mainnet
mainnet_domain = domain_separator(1)
print(f"Mainnet domain: {mainnet_domain.hex()}")

# Sepolia testnet
sepolia_domain = domain_separator(11155111)
print(f"Sepolia domain: {sepolia_domain.hex()}")
```

### Compare Across Chains

```python
from seismic_web3 import domain_separator

# Different chains have different domain separators
mainnet = domain_separator(1)
sepolia = domain_separator(11155111)
custom = domain_separator(12345)

assert mainnet != sepolia
assert mainnet != custom
assert sepolia != custom
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
- **Type hash** - Hash of `EIP712Domain` struct type string
- **Name** - `"Seismic Transaction"` (domain name)
- **Version** - `"2"` (matches `TYPED_DATA_MESSAGE_VERSION`)
- **Chain ID** - Numeric chain identifier
- **Verifying contract** - `0x0000000000000000000000000000000000000000` (signing is off-chain)

## Domain Fields

The domain separator encodes:

| Field | Value | Description |
|-------|-------|-------------|
| `name` | `"Seismic Transaction"` | Domain name |
| `version` | `"2"` | Message version |
| `chainId` | `chain_id` parameter | Chain identifier |
| `verifyingContract` | `0x0...0` | Zero address (off-chain signing) |

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
- **Chain isolation** - Signatures for chain A won't work on chain B
- **Contract binding** - Signatures bound to specific verifying contract
- **Replay prevention** - Can't reuse signatures across different contexts

## Notes

- Used in [`eip712_signing_hash`](eip712-signing-hash.md)
- Always 32 bytes (keccak256 output)
- Same for all transactions on the same chain
- Can be pre-computed and cached per chain

## Performance Optimization

```python
from seismic_web3 import domain_separator

# Cache domain separator per chain
_domain_cache = {}

def get_cached_domain(chain_id: int) -> bytes:
    if chain_id not in _domain_cache:
        _domain_cache[chain_id] = domain_separator(chain_id)
    return _domain_cache[chain_id]

# Use cached version
domain = get_cached_domain(1)
```

## Verifying Contract

The verifying contract is set to `0x0000000000000000000000000000000000000000` because:
- Signing happens **off-chain** via RPC
- No on-chain contract verifies the signature
- The Seismic node verifies the signature directly

## See Also

- [eip712_signing_hash](eip712-signing-hash.md) - Uses domain separator
- [struct_hash](struct-hash.md) - The other component of signing hash
- [build_seismic_typed_data](build-seismic-typed-data.md) - Includes domain in typed data
- [sign_seismic_tx_eip712](sign-seismic-tx-eip712.md) - Full signing process
