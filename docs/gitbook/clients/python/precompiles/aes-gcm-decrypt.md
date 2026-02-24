---
description: On-chain AES-256-GCM decryption
icon: unlock
---

# aes_gcm_decrypt

Decrypt bytes with the AES-GCM precompile at `0x67`.

## Overview

`aes_gcm_decrypt()` and `async_aes_gcm_decrypt()` accept a 32-byte key, a nonce, and ciphertext bytes that include the 16-byte authentication tag.

If tag verification fails, the RPC call fails.

## Signature

```python
def aes_gcm_decrypt(
    w3: Web3,
    *,
    aes_key: Bytes32,
    nonce: int | EncryptionNonce,
    ciphertext: bytes,
) -> HexBytes

async def async_aes_gcm_decrypt(
    w3: AsyncWeb3,
    *,
    aes_key: Bytes32,
    nonce: int | EncryptionNonce,
    ciphertext: bytes,
) -> HexBytes
```

## Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `w3` | `Web3` / `AsyncWeb3` | Yes | Connected Seismic client |
| `aes_key` | [`Bytes32`](../api-reference/types/bytes32.md) | Yes | 32-byte AES key |
| `nonce` | `int` or [`EncryptionNonce`](../api-reference/types/encryption-nonce.md) | Yes | Must match encryption nonce |
| `ciphertext` | `bytes` | Yes | Ciphertext including authentication tag |

## Returns

| Type | Description |
|---|---|
| `HexBytes` | Decrypted plaintext bytes |

## Examples

### Basic Usage

```python
import os
from seismic_web3 import Bytes32, create_public_client
from seismic_web3 import precompiles as sp

w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

key = Bytes32(os.urandom(32))
pt = b"secret"
ct = sp.aes_gcm_encrypt(w3, aes_key=key, nonce=1, plaintext=pt)

decrypted = sp.aes_gcm_decrypt(w3, aes_key=key, nonce=1, ciphertext=bytes(ct))
assert bytes(decrypted) == pt
```

### Handle Authentication Failure

```python
try:
    sp.aes_gcm_decrypt(w3, aes_key=key, nonce=1, ciphertext=b"not-valid")
except RuntimeError as exc:
    print(f"precompile failed: {exc}")
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
    ct = sp.aes_gcm_encrypt(w3, aes_key=key, nonce=1, plaintext=b"async secret")
    pt = await sp.async_aes_gcm_decrypt(w3, aes_key=key, nonce=1, ciphertext=bytes(ct))
    print(bytes(pt))
```

## Gas Cost

The SDK uses:

```python
from math import ceil

total_gas = 1000 + ceil(len(ciphertext) / 16) * 30
```

`len(ciphertext)` includes the 16-byte authentication tag.

## Notes

- Use the same key and nonce used for encryption.
- The decrypt wrapper returns `HexBytes`; cast with `bytes(...)` if needed.
- Failed authentication/tag validation is surfaced as an RPC error.

## See Also

- [aes-gcm-encrypt](aes-gcm-encrypt.md)
- [ecdh](ecdh.md)
