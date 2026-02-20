---
description: On-chain elliptic-curve Diffie-Hellman key exchange
icon: key
---

# ecdh

Perform on-chain ECDH key exchange using Mercury EVM's ECDH precompile.

## Overview

`ecdh()` and `async_ecdh()` call the ECDH precompile at address `0x65` to compute a shared secret from a private key and a public key using elliptic-curve Diffie-Hellman. The result is a 32-byte shared secret derived via HKDF.

## Signature

```python
def ecdh(
    w3: Web3,
    *,
    sk: PrivateKey,
    pk: CompressedPublicKey,
) -> Bytes32

async def async_ecdh(
    w3: AsyncWeb3,
    *,
    sk: PrivateKey,
    pk: CompressedPublicKey,
) -> Bytes32
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `w3` | `Web3` or `AsyncWeb3` | Yes | Web3 instance connected to a Seismic node |
| `sk` | [`PrivateKey`](../api-reference/types/private-key.md) | Yes | 32-byte secret key |
| `pk` | [`CompressedPublicKey`](../api-reference/types/compressed-public-key.md) | Yes | 33-byte compressed public key |

## Returns

| Type | Description |
|------|-------------|
| [`Bytes32`](../api-reference/types/bytes32.md) | 32-byte shared secret |

## Examples

### Basic Usage

```python
from seismic_web3.precompiles import ecdh
from seismic_web3 import PrivateKey, CompressedPublicKey
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Alice's keys
alice_sk = PrivateKey.from_hex("0x1234...")
alice_pk = CompressedPublicKey.from_hex("0x02...")

# Bob's public key
bob_pk = CompressedPublicKey.from_hex("0x03...")

# Alice computes shared secret with Bob's public key
shared_secret = ecdh(w3, sk=alice_sk, pk=bob_pk)
print(f"Shared secret: {shared_secret.to_0x_hex()}")
```

### Two-Party Key Exchange

```python
from seismic_web3.precompiles import ecdh
from seismic_web3 import PrivateKey, CompressedPublicKey
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Alice's keypair
alice_sk = PrivateKey.from_hex("0x1234...")
alice_pk = CompressedPublicKey.from_private_key(alice_sk)

# Bob's keypair
bob_sk = PrivateKey.from_hex("0x5678...")
bob_pk = CompressedPublicKey.from_private_key(bob_sk)

# Both parties compute the same shared secret
alice_shared = ecdh(w3, sk=alice_sk, pk=bob_pk)
bob_shared = ecdh(w3, sk=bob_sk, pk=alice_pk)

assert alice_shared == bob_shared
print(f"Shared secret: {alice_shared.to_0x_hex()}")
```

### Async Usage

```python
from seismic_web3.precompiles import async_ecdh
from seismic_web3 import PrivateKey, CompressedPublicKey
from web3 import AsyncWeb3

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))

    alice_sk = PrivateKey.from_hex("0x1234...")
    bob_pk = CompressedPublicKey.from_hex("0x03...")

    # Compute shared secret asynchronously
    shared_secret = await async_ecdh(w3, sk=alice_sk, pk=bob_pk)
    print(f"Shared secret: {shared_secret.to_0x_hex()}")

# Run with asyncio.run(main())
```

### Use with AES Encryption

```python
from seismic_web3.precompiles import ecdh, aes_gcm_encrypt
from seismic_web3 import PrivateKey, CompressedPublicKey
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Derive shared AES key via ECDH
alice_sk = PrivateKey.from_hex("0x1234...")
bob_pk = CompressedPublicKey.from_hex("0x03...")
aes_key = ecdh(w3, sk=alice_sk, pk=bob_pk)

# Use shared key for encryption
plaintext = b"Secret message for Bob"
ciphertext = aes_gcm_encrypt(w3, aes_key=aes_key, nonce=1, plaintext=plaintext)
print(f"Ciphertext: {ciphertext.hex()}")
```

### Generate Keypair from Private Key

```python
from seismic_web3.precompiles import ecdh
from seismic_web3 import PrivateKey, CompressedPublicKey
from web3 import Web3
import os

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Generate new private key
my_sk = PrivateKey(os.urandom(32))

# Derive public key from private key
my_pk = CompressedPublicKey.from_private_key(my_sk)

# Exchange with peer
peer_pk = CompressedPublicKey.from_hex("0x03...")
shared_secret = ecdh(w3, sk=my_sk, pk=peer_pk)
```

## How It Works

1. **Encode parameters** - Concatenates 32-byte private key and 33-byte public key
2. **Call precompile** - Issues an `eth_call` to address `0x65` with 3120 gas
3. **Compute ECDH** - Precompile performs scalar multiplication on secp256k1 curve
4. **Derive key** - Applies HKDF to the ECDH point to produce a 32-byte secret

## Gas Cost

Fixed gas cost: **3120 gas**
- 3000 gas for ECDH scalar multiplication
- 120 gas for HKDF key derivation

## Notes

- Uses the secp256k1 elliptic curve (same as Ethereum)
- Public keys must be in compressed format (33 bytes starting with `0x02` or `0x03`)
- The ECDH point is passed through HKDF for key uniformity
- Both parties compute the same shared secret: `ecdh(sk_A, pk_B) == ecdh(sk_B, pk_A)`
- The shared secret can be used as an AES-256 key

## Warnings

- **Private key security** - Never expose or log private keys
- **Public key validation** - Invalid public keys will cause the precompile to revert
- **Key reuse** - Using the same keypair for multiple sessions reduces forward secrecy

## See Also

- [aes-gcm-encrypt](aes-gcm-encrypt.md) - Encrypt with derived key
- [aes-gcm-decrypt](aes-gcm-decrypt.md) - Decrypt with derived key
- [hkdf](hkdf.md) - Key derivation function
- [PrivateKey](../api-reference/types/private-key.md) - 32-byte private key type
- [CompressedPublicKey](../api-reference/types/compressed-public-key.md) - 33-byte public key type
- [Bytes32](../api-reference/types/bytes32.md) - 32-byte value type
