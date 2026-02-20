---
icon: microchip
---

# Precompiles

The Mercury EVM includes six cryptographic precompiles accessible through any Web3 instance connected to a Seismic node. All functions have async variants.

## RNG

Generate a random number on-chain.

```python
from seismic_web3.precompiles import rng

value = rng(w3, num_bytes=32)  # returns int
```

## ECDH

On-chain Elliptic Curve Diffie-Hellman key exchange.

```python
from seismic_web3.precompiles import ecdh

shared_secret = ecdh(w3, sk=my_private_key, pk=their_public_key)  # returns Bytes32
```

## AES-GCM encrypt / decrypt

On-chain AES-GCM encryption and decryption.

```python
from seismic_web3.precompiles import aes_gcm_encrypt, aes_gcm_decrypt

ciphertext = aes_gcm_encrypt(w3, aes_key=key, nonce=1, plaintext=b"secret")
plaintext = aes_gcm_decrypt(w3, aes_key=key, nonce=1, ciphertext=bytes(ciphertext))
```

## HKDF

On-chain HKDF key derivation.

```python
from seismic_web3.precompiles import hkdf

derived_key = hkdf(w3, b"input key material")  # returns Bytes32
```

## secp256k1 sign

On-chain secp256k1 signing.

```python
from seismic_web3.precompiles import secp256k1_sign

signature = secp256k1_sign(w3, sk=my_private_key, message="hello")  # returns HexBytes
```

## Precompile addresses

| Address | Function        |
| ------- | --------------- |
| `0x64`  | RNG             |
| `0x65`  | ECDH            |
| `0x66`  | AES-GCM Encrypt |
| `0x67`  | AES-GCM Decrypt |
| `0x68`  | HKDF            |
| `0x69`  | secp256k1 Sign  |

## Async variants

Every function has an async counterpart: `async_rng`, `async_ecdh`, `async_aes_gcm_encrypt`, `async_aes_gcm_decrypt`, `async_hkdf`, `async_secp256k1_sign`.
