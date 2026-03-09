---
description: On-chain AES-256-GCM encryption
icon: lock
---

# aes_gcm_encrypt

Encrypt bytes with the AES-GCM precompile at `0x66`.

## Overview

`aes_gcm_encrypt()` and `async_aes_gcm_encrypt()` accept a 32-byte key, a 12-byte nonce (or nonce integer), and plaintext bytes.

The returned ciphertext includes the 16-byte authentication tag.

## Signature

```python
def aes_gcm_encrypt(
    w3: Web3,
    *,
    aes_key: Bytes32,
    nonce: int | EncryptionNonce,
    plaintext: bytes,
) -> HexBytes

async def async_aes_gcm_encrypt(
    w3: AsyncWeb3,
    *,
    aes_key: Bytes32,
    nonce: int | EncryptionNonce,
    plaintext: bytes,
) -> HexBytes
```

## Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `w3` | `Web3` / `AsyncWeb3` | Yes | Connected Seismic client |
| `aes_key` | [`Bytes32`](../api-reference/types/bytes32.md) | Yes | 32-byte AES key |
| `nonce` | `int` or [`EncryptionNonce`](../api-reference/types/encryption-nonce.md) | Yes | 12-byte nonce (or int converted to 12-byte big-endian) |
| `plaintext` | `bytes` | Yes | Data to encrypt |

## Returns

| Type | Description |
|---|---|
| `HexBytes` | Ciphertext with authentication tag appended |

## Examples

### Basic Usage

```python
import os
from seismic_web3 import Bytes32, create_public_client
from seismic_web3 import precompiles as sp

w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

key = Bytes32(os.urandom(32))
plaintext = b"secret message"

ciphertext = sp.aes_gcm_encrypt(w3, aes_key=key, nonce=1, plaintext=plaintext)
print(ciphertext.hex())
```

### With `EncryptionNonce`

```python
import os
from seismic_web3 import EncryptionNonce
from seismic_web3 import precompiles as sp

nonce = EncryptionNonce(os.urandom(12))
ciphertext = sp.aes_gcm_encrypt(w3, aes_key=key, nonce=nonce, plaintext=b"hello")
```

### Async Usage

```python
import os
from seismic_web3 import create_async_public_client
from seismic_web3 import Bytes32
from seismic_web3 import precompiles as sp

async def main():
    w3 = create_async_public_client("https://gcp-1.seismictest.net/rpc")
    key = Bytes32(os.urandom(32))
    ct = await sp.async_aes_gcm_encrypt(w3, aes_key=key, nonce=1, plaintext=b"async")
    print(ct.hex())
```

### Encrypt/Decrypt Round Trip

```python
from seismic_web3 import precompiles as sp

ct = sp.aes_gcm_encrypt(w3, aes_key=key, nonce=42, plaintext=b"round trip")
pt = sp.aes_gcm_decrypt(w3, aes_key=key, nonce=42, ciphertext=bytes(ct))
assert bytes(pt) == b"round trip"
```

## Gas Cost

The SDK uses:

```python
from math import ceil

total_gas = 1000 + ceil(len(plaintext) / 16) * 30
```

Examples:
- `len(plaintext)=0`: `1000`
- `len(plaintext)=16`: `1030`
- `len(plaintext)=17`: `1060`

## Notes

- Never reuse a nonce with the same key.
- `int` nonces are encoded to 12-byte big-endian values.
- Returned bytes are `ciphertext || tag`.

## See Also

- [aes-gcm-decrypt](aes-gcm-decrypt.md)
- [ecdh](ecdh.md)
- [Bytes32](../api-reference/types/bytes32.md)
- [EncryptionNonce](../api-reference/types/encryption-nonce.md)
