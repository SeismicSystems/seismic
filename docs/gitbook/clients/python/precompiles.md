---
description: Call Mercury EVM precompiles from Python
icon: microchip
---

# Precompiles

Mercury EVM ships with precompiles for common cryptographic operations. They're callable via `eth_call` — no encryption state needed, just a `Web3` instance connected to a Seismic node.

```python
from seismic_web3.precompiles import (
    rng, ecdh, aes_gcm_encrypt, aes_gcm_decrypt, hkdf, secp256k1_sign,
)
from seismic_web3 import Bytes32, PrivateKey, CompressedPublicKey

# On-chain random number generation
random_val = rng(w3, num_bytes=32)

# On-chain ECDH key exchange
shared_secret = ecdh(w3, sk=my_private_key, pk=their_public_key)

# On-chain AES-GCM encrypt / decrypt
ciphertext = aes_gcm_encrypt(w3, aes_key=key, nonce=1, plaintext=b"secret")
plaintext = aes_gcm_decrypt(w3, aes_key=key, nonce=1, ciphertext=bytes(ciphertext))

# On-chain HKDF key derivation
derived_key = hkdf(w3, b"input key material")

# On-chain secp256k1 signing
signature = secp256k1_sign(w3, sk=my_private_key, message="hello")
```

All functions have async variants: `async_rng`, `async_ecdh`, `async_aes_gcm_encrypt`, `async_aes_gcm_decrypt`, `async_hkdf`, `async_secp256k1_sign`.

***

### Reference

| Precompile | Address | Function | Returns |
|---|---|---|---|
| RNG | `0x64` | `rng(w3, num_bytes=, pers=)` | `int` |
| ECDH | `0x65` | `ecdh(w3, sk=, pk=)` | `Bytes32` |
| AES Encrypt | `0x66` | `aes_gcm_encrypt(w3, aes_key=, nonce=, plaintext=)` | `HexBytes` |
| AES Decrypt | `0x67` | `aes_gcm_decrypt(w3, aes_key=, nonce=, ciphertext=)` | `HexBytes` |
| HKDF | `0x68` | `hkdf(w3, ikm)` | `Bytes32` |
| secp256k1 Sign | `0x69` | `secp256k1_sign(w3, sk=, message=)` | `HexBytes` |
