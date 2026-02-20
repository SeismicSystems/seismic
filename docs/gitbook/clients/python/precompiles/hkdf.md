---
description: On-chain HKDF-SHA256 key derivation
icon: fingerprint
---

# hkdf

Derive cryptographic keys on-chain using Mercury EVM's HKDF precompile.

## Overview

`hkdf()` and `async_hkdf()` call the HKDF precompile at address `0x68` to perform HKDF-SHA256 key derivation on input key material (IKM). The result is a uniformly distributed 32-byte derived key.

## Signature

```python
def hkdf(
    w3: Web3,
    ikm: bytes,
) -> Bytes32

async def async_hkdf(
    w3: AsyncWeb3,
    ikm: bytes,
) -> Bytes32
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `w3` | `Web3` or `AsyncWeb3` | Yes | Web3 instance connected to a Seismic node |
| `ikm` | `bytes` | Yes | Input key material (arbitrary bytes) |

## Returns

| Type | Description |
|------|-------------|
| [`Bytes32`](../api-reference/types/bytes32.md) | 32-byte derived key |

## Examples

### Basic Usage

```python
from seismic_web3.precompiles import hkdf
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://sepolia.seismic.foundation"))

# Derive key from input material
ikm = b"my-input-key-material"
derived_key = hkdf(w3, ikm)
print(f"Derived key: {derived_key.to_0x_hex()}")
```

### Derive from Password

```python
from seismic_web3.precompiles import hkdf
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://sepolia.seismic.foundation"))

# Derive key from password (not recommended for real password hashing)
password = b"user-password-123"
derived_key = hkdf(w3, password)
print(f"Derived key: {derived_key.to_0x_hex()}")
```

### Async Usage

```python
from seismic_web3.precompiles import async_hkdf
from web3 import AsyncWeb3

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://sepolia.seismic.foundation"))

    ikm = b"input-key-material"
    derived_key = await async_hkdf(w3, ikm)
    print(f"Derived key: {derived_key.to_0x_hex()}")

# Run with asyncio.run(main())
```

### Derive Multiple Keys

```python
from seismic_web3.precompiles import hkdf
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://sepolia.seismic.foundation"))

# Derive different keys by varying input
base_ikm = b"shared-secret"
contexts = [b"encryption", b"authentication", b"signing"]

for ctx in contexts:
    ikm = base_ikm + ctx
    key = hkdf(w3, ikm)
    print(f"Key for {ctx.decode()}: {key.to_0x_hex()}")
```

### Use as AES Key

```python
from seismic_web3.precompiles import hkdf, aes_gcm_encrypt
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://sepolia.seismic.foundation"))

# Derive AES key from input material
ikm = b"my-master-secret"
aes_key = hkdf(w3, ikm)

# Use derived key for encryption
plaintext = b"Encrypted with derived key"
ciphertext = aes_gcm_encrypt(w3, aes_key=aes_key, nonce=1, plaintext=plaintext)
print(f"Ciphertext: {ciphertext.hex()}")
```

### Derive from ECDH Output

```python
from seismic_web3.precompiles import ecdh, hkdf
from seismic_web3 import PrivateKey, CompressedPublicKey
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://sepolia.seismic.foundation"))

# First perform ECDH
my_sk = PrivateKey.from_hex("0x1234...")
peer_pk = CompressedPublicKey.from_hex("0x03...")
shared_secret = ecdh(w3, sk=my_sk, pk=peer_pk)

# Further derive key from shared secret
ikm = bytes(shared_secret) + b"application-context"
derived_key = hkdf(w3, ikm)
print(f"Derived key: {derived_key.to_0x_hex()}")
```

### Deterministic Key Derivation

```python
from seismic_web3.precompiles import hkdf
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://sepolia.seismic.foundation"))

# Same input always produces same output
ikm = b"deterministic-input"
key1 = hkdf(w3, ikm)
key2 = hkdf(w3, ikm)

assert key1 == key2
print(f"Deterministic key: {key1.to_0x_hex()}")
```

### Variable-Length Input

```python
from seismic_web3.precompiles import hkdf
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://sepolia.seismic.foundation"))

# HKDF accepts arbitrary-length input
inputs = [
    b"short",
    b"medium length input",
    b"very long input key material " * 10,
]

for ikm in inputs:
    key = hkdf(w3, ikm)
    print(f"Input length {len(ikm)}: {key.to_0x_hex()}")
```

### Key Separation by Context

```python
from seismic_web3.precompiles import hkdf
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://sepolia.seismic.foundation"))

master_secret = b"shared-master-key"

# Derive separate keys for different purposes
encryption_key = hkdf(w3, master_secret + b"\x01encryption")
auth_key = hkdf(w3, master_secret + b"\x02authentication")
signing_key = hkdf(w3, master_secret + b"\x03signing")

print(f"Encryption: {encryption_key.to_0x_hex()}")
print(f"Auth: {auth_key.to_0x_hex()}")
print(f"Signing: {signing_key.to_0x_hex()}")
```

## How It Works

1. **Encode parameters** - Passes input key material as-is
2. **Call precompile** - Issues an `eth_call` to address `0x68` with gas proportional to input length
3. **HKDF derivation** - Precompile performs HKDF-SHA256 extract and expand phases
4. **Return key** - Returns first 32 bytes of derived key material

## Gas Cost

Gas cost is calculated as:
```python
sha256_base = 60
sha256_per_word = 12  # per 32-byte word
hkdf_expand = 120

# Extract phase (twice)
extract_cost = 2 * (sha256_base + (len(ikm) // 32) * sha256_per_word + 3000)

# Expand phase
total_gas = extract_cost + hkdf_expand
```

For example:
- 32 bytes IKM: ~6144 gas
- 64 bytes IKM: ~6168 gas
- 128 bytes IKM: ~6216 gas

## Notes

- Uses HKDF-SHA256 from RFC 5869
- Always returns exactly 32 bytes regardless of input length
- Input key material can be any length
- The derivation is deterministic: same IKM always produces same output
- Internally performs both HKDF-Extract and HKDF-Expand phases
- The derived key has uniform distribution suitable for cryptographic use

## Use Cases

- Derive encryption keys from shared secrets
- Convert non-uniform entropy into uniform keys
- Key separation: derive multiple keys from one master secret
- Post-process ECDH output for additional security

## Warnings

- **Not for password hashing** - Use proper password hashing algorithms (bcrypt, argon2) for passwords
- **Input entropy** - Output security depends entirely on input entropy
- **Deterministic** - Same input always yields same output (no salt/randomness added)

## See Also

- [ecdh](ecdh.md) - Often used before HKDF to derive keys
- [aes-gcm-encrypt](aes-gcm-encrypt.md) - Use derived keys for encryption
- [rng](rng.md) - Generate random input material
- [Bytes32](../api-reference/types/bytes32.md) - 32-byte derived key type
