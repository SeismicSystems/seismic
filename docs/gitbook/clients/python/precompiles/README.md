---
description: Call Mercury EVM precompiles from Python
icon: microchip
---

# Precompiles

The Python SDK exposes wrappers for six Mercury EVM precompiles (`0x64` through `0x69`).

These wrappers use plain `eth_call`, so you only need a `Web3` connection to a Seismic node. No wallet client or encryption state is required.

## Quick Start

```python
import os
from seismic_web3 import PrivateKey, create_public_client
from seismic_web3.crypto.secp import private_key_to_compressed_public_key
from seismic_web3 import precompiles as sp

w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

# 1) Random bytes as int
random_val = sp.rng(w3, num_bytes=32)

# 2) Shared key from ECDH
my_sk = PrivateKey(os.urandom(32))
peer_sk = PrivateKey(os.urandom(32))
peer_pk = private_key_to_compressed_public_key(peer_sk)
shared_key = sp.ecdh(w3, sk=my_sk, pk=peer_pk)

# 3) AES-GCM encrypt/decrypt
ciphertext = sp.aes_gcm_encrypt(
    w3,
    aes_key=shared_key,
    nonce=1,
    plaintext=b"secret",
)
plaintext = sp.aes_gcm_decrypt(
    w3,
    aes_key=shared_key,
    nonce=1,
    ciphertext=bytes(ciphertext),
)

# 4) HKDF derivation
derived_key = sp.hkdf(w3, b"input key material")

# 5) secp256k1 signing
signature = sp.secp256k1_sign(w3, sk=my_sk, message="hello")
```

All wrappers also have async variants:
`async_rng`, `async_ecdh`, `async_aes_gcm_encrypt`, `async_aes_gcm_decrypt`, `async_hkdf`, `async_secp256k1_sign`.

## Reference

| Precompile | Address | Function | Returns |
|---|---|---|---|
| RNG | `0x64` | [`rng(w3, num_bytes=, pers=)`](rng.md) | `int` |
| ECDH | `0x65` | [`ecdh(w3, sk=, pk=)`](ecdh.md) | `Bytes32` |
| AES Encrypt | `0x66` | [`aes_gcm_encrypt(w3, aes_key=, nonce=, plaintext=)`](aes-gcm-encrypt.md) | `HexBytes` |
| AES Decrypt | `0x67` | [`aes_gcm_decrypt(w3, aes_key=, nonce=, ciphertext=)`](aes-gcm-decrypt.md) | `HexBytes` |
| HKDF | `0x68` | [`hkdf(w3, ikm)`](hkdf.md) | `Bytes32` |
| secp256k1 Sign | `0x69` | [`secp256k1_sign(w3, sk=, message=)`](secp256k1-sign.md) | `HexBytes` |
