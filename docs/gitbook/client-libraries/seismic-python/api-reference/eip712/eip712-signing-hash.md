---
description: Compute EIP-712 signing hash for Seismic transaction
icon: fingerprint
---

# eip712_signing_hash

Compute the EIP-712 signing hash for a Seismic transaction.

## Overview

`eip712_signing_hash()` computes the 32-byte message hash that gets ECDSA-signed for EIP-712 typed data transactions. This is the hash of the domain separator and struct hash, following the EIP-712 specification.

## Signature

```python
def eip712_signing_hash(tx: UnsignedSeismicTx) -> bytes
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tx` | [`UnsignedSeismicTx`](../transaction-types/unsigned-seismic-tx.md) | Yes | The unsigned Seismic transaction |

## Returns

| Type | Description |
|------|-------------|
| `bytes` | 32-byte keccak256 digest to be ECDSA-signed |

## Examples

### Basic Usage

```python
from seismic_web3 import eip712_signing_hash, UnsignedSeismicTx

unsigned_tx = UnsignedSeismicTx(...)

# Compute signing hash
signing_hash = eip712_signing_hash(unsigned_tx)

print(f"Signing hash: {signing_hash.hex()}")
print(f"Length: {len(signing_hash)} bytes")  # Always 32
```

### Manual Signing

```python
from seismic_web3 import eip712_signing_hash, PrivateKey
from eth_keys import keys as eth_keys

unsigned_tx = UnsignedSeismicTx(...)
private_key = PrivateKey(...)

# Compute hash
msg_hash = eip712_signing_hash(unsigned_tx)

# Sign manually
sk = eth_keys.PrivateKey(bytes(private_key))
sig = sk.sign_msg_hash(msg_hash)

print(f"v: {sig.v}")
print(f"r: {sig.r}")
print(f"s: {sig.s}")
```

### Compare with Raw Hash

```python
from seismic_web3 import eip712_signing_hash
from seismic_web3.transaction.serialize import hash_unsigned

unsigned_tx = UnsignedSeismicTx(...)

# EIP-712 hash
eip712_hash = eip712_signing_hash(unsigned_tx)

# Raw RLP hash (message_version=0)
raw_hash = hash_unsigned(unsigned_tx)

# They differ!
assert eip712_hash != raw_hash
print(f"EIP-712: {eip712_hash.hex()}")
print(f"Raw:     {raw_hash.hex()}")
```

### Verify Against External Signer

```python
from seismic_web3 import eip712_signing_hash, build_seismic_typed_data

unsigned_tx = UnsignedSeismicTx(...)

# Compute expected hash
expected_hash = eip712_signing_hash(unsigned_tx)

# Build typed data for external signer
typed_data = build_seismic_typed_data(unsigned_tx)

# External signer (e.g., MetaMask) should produce same hash
# signature = await wallet.signTypedData(typed_data)
# Then verify signature against expected_hash
```

## How It Works

The function computes:

```
keccak256("\x19\x01" ‖ domainSeparator ‖ structHash)
```

Where:
- `\x19\x01` - EIP-712 magic bytes
- `domainSeparator` - From [`domain_separator(chain_id)`](domain-separator.md)
- `structHash` - From [`struct_hash(tx)`](struct-hash.md)

## Implementation

```python
def eip712_signing_hash(tx: UnsignedSeismicTx) -> bytes:
    return keccak(
        b"\x19\x01" +
        domain_separator(tx.chain_id) +
        struct_hash(tx)
    )
```

## EIP-712 Structure

The hash encodes:
1. **Domain** - Chain ID and verifying contract
2. **Message** - All transaction fields (chain ID, nonce, gas, data, Seismic elements)

This provides:
- **Structured hashing** - Not just raw bytes
- **Domain separation** - Binds to specific chain and contract
- **Human-readable** - Wallets can display structured data

## Notes

- Used internally by [`sign_seismic_tx_eip712`](sign-seismic-tx-eip712.md)
- Different from raw signing hash (RLP hash of unsigned transaction)
- Always 32 bytes (keccak256 output)
- The Seismic node checks `message_version` to determine which hash to verify

## Use Cases

- **Custom signing** - Sign with external wallet or HSM
- **Verification** - Check signature correctness
- **Testing** - Verify hash computation
- **Debugging** - Inspect signing process

## Warnings

- **Message version** - Should be used with `message_version == 2`
- **Chain ID** - Must match the target chain
- **Hash only** - This returns the hash, not the signature

## See Also

- [sign_seismic_tx_eip712](sign-seismic-tx-eip712.md) - Uses this hash to sign
- [domain_separator](domain-separator.md) - Computes domain separator
- [struct_hash](struct-hash.md) - Computes struct hash
- [build_seismic_typed_data](build-seismic-typed-data.md) - Build full typed data
- [UnsignedSeismicTx](../transaction-types/unsigned-seismic-tx.md) - Input transaction type
