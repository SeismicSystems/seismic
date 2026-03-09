---
description: On-chain elliptic-curve Diffie-Hellman key exchange
icon: key
---

# ecdh

Compute a shared key with the ECDH precompile at `0x65`.

## Overview

`ecdh()` and `async_ecdh()` take a `PrivateKey` and a `CompressedPublicKey` and return a `Bytes32` shared key.

The on-chain precompile performs ECDH and an HKDF derivation step, so the returned `Bytes32` is ready to use as an AES key.

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
|---|---|---|---|
| `w3` | `Web3` / `AsyncWeb3` | Yes | Connected Seismic client |
| `sk` | [`PrivateKey`](../api-reference/types/private-key.md) | Yes | Local 32-byte private key |
| `pk` | [`CompressedPublicKey`](../api-reference/types/compressed-public-key.md) | Yes | Peer compressed public key (33 bytes) |

## Returns

| Type | Description |
|---|---|
| [`Bytes32`](../api-reference/types/bytes32.md) | Derived shared key |

## Examples

### Two-Party Key Exchange

```python
import os
from seismic_web3 import PrivateKey, create_public_client
from seismic_web3.crypto.secp import private_key_to_compressed_public_key
from seismic_web3 import precompiles as sp

w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

alice_sk = PrivateKey(os.urandom(32))
bob_sk = PrivateKey(os.urandom(32))

alice_pk = private_key_to_compressed_public_key(alice_sk)
bob_pk = private_key_to_compressed_public_key(bob_sk)

alice_shared = sp.ecdh(w3, sk=alice_sk, pk=bob_pk)
bob_shared = sp.ecdh(w3, sk=bob_sk, pk=alice_pk)

assert alice_shared == bob_shared
```

### Use Shared Key for AES-GCM

```python
from seismic_web3 import precompiles as sp

key = sp.ecdh(w3, sk=alice_sk, pk=bob_pk)
ct = sp.aes_gcm_encrypt(w3, aes_key=key, nonce=1, plaintext=b"secret")
pt = sp.aes_gcm_decrypt(w3, aes_key=key, nonce=1, ciphertext=bytes(ct))
assert bytes(pt) == b"secret"
```

### Async Usage

```python
import os
from seismic_web3 import create_async_public_client
from seismic_web3 import PrivateKey
from seismic_web3.crypto.secp import private_key_to_compressed_public_key
from seismic_web3 import precompiles as sp

async def main():
    w3 = create_async_public_client("https://gcp-1.seismictest.net/rpc")
    alice_sk = PrivateKey(os.urandom(32))
    bob_sk = PrivateKey(os.urandom(32))
    bob_pk = private_key_to_compressed_public_key(bob_sk)
    shared = await sp.async_ecdh(w3, sk=alice_sk, pk=bob_pk)
    print(shared.to_0x_hex())
```

## Gas Cost

Fixed cost: `3120`.

## Notes

- Input layout is `sk(32) || pk(33)`.
- Invalid keys cause the call to fail.
- Keep private keys out of logs and telemetry.

## See Also

- [aes-gcm-encrypt](aes-gcm-encrypt.md)
- [aes-gcm-decrypt](aes-gcm-decrypt.md)
- [hkdf](hkdf.md)
