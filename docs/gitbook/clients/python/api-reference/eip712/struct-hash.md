---
description: Compute EIP-712 struct hash for TxSeismic
icon: hashtag
---

# struct_hash

Compute the EIP-712 struct hash for a TxSeismic transaction.

## Overview

`struct_hash()` computes the EIP-712 struct hash by encoding all transaction fields according to the EIP-712 specification. Dynamic types (like `bytes`) are hashed, while static types are left-padded to 32 bytes.

## Signature

```python
def struct_hash(tx: UnsignedSeismicTx) -> bytes
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tx` | [`UnsignedSeismicTx`](../transaction-types/unsigned-seismic-tx.md) | Yes | The unsigned Seismic transaction |

## Returns

| Type | Description |
|------|-------------|
| `bytes` | 32-byte keccak256 hash of the encoded struct |

## Examples

### Basic Usage

```python
from seismic_web3 import struct_hash, UnsignedSeismicTx

unsigned_tx = UnsignedSeismicTx(...)

# Compute struct hash
hash_value = struct_hash(unsigned_tx)

print(f"Struct hash: {hash_value.hex()}")
print(f"Length: {len(hash_value)} bytes")  # Always 32
```

### Use in Signing Hash

```python
from seismic_web3 import struct_hash, domain_separator
from eth_hash.auto import keccak

unsigned_tx = UnsignedSeismicTx(...)

# Compute components
domain = domain_separator(unsigned_tx.chain_id)
struct = struct_hash(unsigned_tx)

# Build full signing hash
signing_hash = keccak(b"\x19\x01" + domain + struct)
```

### Compare Different Transactions

```python
from seismic_web3 import struct_hash

tx1 = UnsignedSeismicTx(...)
tx2 = UnsignedSeismicTx(...)

hash1 = struct_hash(tx1)
hash2 = struct_hash(tx2)

# Different transactions have different hashes
if hash1 == hash2:
    print("Identical transactions")
else:
    print("Different transactions")
```

### Debug Field Encoding

```python
from seismic_web3 import struct_hash, UnsignedSeismicTx
from eth_hash.auto import keccak

tx = UnsignedSeismicTx(...)

# Compute struct hash
result = struct_hash(tx)

# Manually verify data field encoding (dynamic type)
data_hash = keccak(bytes(tx.data))
print(f"Data hash: {data_hash.hex()}")

# Manually verify encryption pubkey (dynamic type)
pubkey_hash = keccak(bytes(tx.seismic.encryption_pubkey))
print(f"Pubkey hash: {pubkey_hash.hex()}")
```

## How It Works

The function computes:

```
keccak256(
    TX_SEISMIC_TYPE_HASH
    ‖ encode(chain_id)
    ‖ encode(nonce)
    ‖ encode(gas_price)
    ‖ encode(gas)
    ‖ encode(to)
    ‖ encode(value)
    ‖ keccak256(data)              // Dynamic type
    ‖ keccak256(encryption_pubkey) // Dynamic type
    ‖ encode(encryption_nonce)
    ‖ encode(message_version)
    ‖ recent_block_hash            // Already 32 bytes
    ‖ encode(expires_at_block)
    ‖ encode(signed_read)
)
```

## Field Encoding

### Static Types (Left-Padded to 32 Bytes)

- `uint64` - `chain_id`, `nonce`, `gas` (gasLimit), `expires_at_block`
- `uint128` - `gas_price`
- `uint256` - `value`
- `uint96` - `encryption_nonce` (converted from 12 bytes to integer)
- `uint8` - `message_version`
- `bool` - `signed_read` (0 or 1)
- `address` - `to` (left-padded 12 bytes + 20-byte address, or 32 zero bytes if `None`)
- `bytes32` - `recent_block_hash` (already 32 bytes)

### Dynamic Types (Hashed with keccak256)

- `bytes` - `data` (encrypted calldata)
- `bytes` - `encryption_pubkey` (33-byte compressed public key)

## Implementation

```python
def struct_hash(tx: UnsignedSeismicTx) -> bytes:
    se = tx.seismic
    enc_nonce_int = int.from_bytes(bytes(se.encryption_nonce), "big")

    return keccak(
        TX_SEISMIC_TYPE_HASH +
        _pad32_int(tx.chain_id) +
        _pad32_int(tx.nonce) +
        _pad32_int(tx.gas_price) +
        _pad32_int(tx.gas) +
        _pad32_address(tx.to) +
        _pad32_int(tx.value) +
        keccak(bytes(tx.data)) +
        keccak(bytes(se.encryption_pubkey)) +
        _pad32_int(enc_nonce_int) +
        _pad32_int(se.message_version) +
        bytes(se.recent_block_hash) +
        _pad32_int(se.expires_at_block) +
        _pad32_bool(se.signed_read)
    )
```

## Type Hash

The `TX_SEISMIC_TYPE_HASH` is the keccak256 hash of the type string:

```
TxSeismic(uint64 chainId,uint64 nonce,uint128 gasPrice,uint64 gasLimit,address to,uint256 value,bytes input,bytes encryptionPubkey,uint96 encryptionNonce,uint8 messageVersion,bytes32 recentBlockHash,uint64 expiresAtBlock,bool signedRead)
```

## Notes

- Used in [`eip712_signing_hash`](eip712-signing-hash.md)
- Always 32 bytes (keccak256 output)
- Encodes all transaction fields deterministically
- Dynamic types (`bytes`) are hashed before inclusion

## Encoding Details

### encryption_nonce Conversion

The 12-byte `EncryptionNonce` is converted to a `uint96` integer:

```python
enc_nonce_int = int.from_bytes(bytes(se.encryption_nonce), "big")
```

Then encoded as a 32-byte padded integer.

### Address Encoding

Addresses are left-padded to 32 bytes:
- Regular address: `0x000000000000000000000000` + `<20-byte address>`
- `None` (contract creation): `0x0000000000000000000000000000000000000000000000000000000000000000`

### Boolean Encoding

Booleans are encoded as 32-byte integers:
- `True` → `0x0000000000000000000000000000000000000000000000000000000000000001`
- `False` → `0x0000000000000000000000000000000000000000000000000000000000000000`

## See Also

- [eip712_signing_hash](eip712-signing-hash.md) - Uses struct hash
- [domain_separator](domain-separator.md) - The other component of signing hash
- [build_seismic_typed_data](build-seismic-typed-data.md) - Builds full typed data
- [sign_seismic_tx_eip712](sign-seismic-tx-eip712.md) - Full signing process
- [UnsignedSeismicTx](../transaction-types/unsigned-seismic-tx.md) - Input transaction type
