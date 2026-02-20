---
description: On-chain AES-256-GCM encryption
icon: lock
---

# aes\_gcm\_encrypt

Encrypt data on-chain using Mercury EVM's AES-GCM encryption precompile.

## Overview

`aes_gcm_encrypt()` and `async_aes_gcm_encrypt()` call the AES-GCM encryption precompile at address `0x66` to encrypt plaintext using AES-256-GCM. The ciphertext includes a 16-byte authentication tag.

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

| Parameter   | Type                                                                       | Required | Description                                       |
| ----------- | -------------------------------------------------------------------------- | -------- | ------------------------------------------------- |
| `w3`        | `Web3` or `AsyncWeb3`                                                      | Yes      | Web3 instance connected to a Seismic node         |
| `aes_key`   | [`Bytes32`](../api-reference/bytes32/)                                     | Yes      | 32-byte AES-256 encryption key                    |
| `nonce`     | `int` or [`EncryptionNonce`](../api-reference/bytes32/encryption-nonce.md) | Yes      | 12-byte nonce (can be integer or EncryptionNonce) |
| `plaintext` | `bytes`                                                                    | Yes      | Data to encrypt                                   |

## Returns

| Type       | Description                                                     |
| ---------- | --------------------------------------------------------------- |
| `HexBytes` | Ciphertext bytes (includes 16-byte authentication tag appended) |

## Examples

### Basic Usage

```python
from seismic_web3.precompiles import aes_gcm_encrypt
from seismic_web3 import Bytes32
from web3 import Web3
import os

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Generate AES key
aes_key = Bytes32(os.urandom(32))

# Encrypt plaintext
plaintext = b"Secret message"
ciphertext = aes_gcm_encrypt(w3, aes_key=aes_key, nonce=1, plaintext=plaintext)
print(f"Ciphertext: {ciphertext.hex()}")
print(f"Length: {len(ciphertext)} bytes (plaintext + 16-byte tag)")
```

### With Integer Nonce

```python
from seismic_web3.precompiles import aes_gcm_encrypt
from seismic_web3 import Bytes32
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

aes_key = Bytes32.from_hex("0x1234...")

# Use integer nonce (converted to 12 bytes)
for i in range(5):
    plaintext = f"Message {i}".encode()
    ciphertext = aes_gcm_encrypt(w3, aes_key=aes_key, nonce=i, plaintext=plaintext)
    print(f"Ciphertext {i}: {ciphertext.hex()}")
```

### With EncryptionNonce

```python
from seismic_web3.precompiles import aes_gcm_encrypt
from seismic_web3 import Bytes32, EncryptionNonce
from web3 import Web3
import os

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

aes_key = Bytes32.from_hex("0x1234...")

# Use EncryptionNonce (12 bytes)
nonce = EncryptionNonce(os.urandom(12))
plaintext = b"Secret data"
ciphertext = aes_gcm_encrypt(w3, aes_key=aes_key, nonce=nonce, plaintext=plaintext)
print(f"Ciphertext: {ciphertext.hex()}")
```

### Async Usage

```python
from seismic_web3.precompiles import async_aes_gcm_encrypt
from seismic_web3 import Bytes32
from web3 import AsyncWeb3
import os

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))

    aes_key = Bytes32(os.urandom(32))
    plaintext = b"Async secret"

    # Encrypt asynchronously
    ciphertext = await async_aes_gcm_encrypt(
        w3, aes_key=aes_key, nonce=1, plaintext=plaintext
    )
    print(f"Ciphertext: {ciphertext.hex()}")

# Run with asyncio.run(main())
```

### Encrypt-Decrypt Round Trip

```python
from seismic_web3.precompiles import aes_gcm_encrypt, aes_gcm_decrypt
from seismic_web3 import Bytes32
from web3 import Web3
import os

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

aes_key = Bytes32(os.urandom(32))
original = b"Round trip test"

# Encrypt
ciphertext = aes_gcm_encrypt(w3, aes_key=aes_key, nonce=42, plaintext=original)

# Decrypt
decrypted = aes_gcm_decrypt(w3, aes_key=aes_key, nonce=42, ciphertext=ciphertext)

assert decrypted == original
print(f"Success! Original: {original}")
```

### Encrypt Multiple Messages

```python
from seismic_web3.precompiles import aes_gcm_encrypt
from seismic_web3 import Bytes32
from web3 import Web3
import os

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

aes_key = Bytes32(os.urandom(32))

messages = [b"Message 1", b"Message 2", b"Message 3"]
ciphertexts = []

for i, msg in enumerate(messages):
    ct = aes_gcm_encrypt(w3, aes_key=aes_key, nonce=i, plaintext=msg)
    ciphertexts.append(ct)
    print(f"Encrypted message {i}: {len(ct)} bytes")
```

### With ECDH-Derived Key

```python
from seismic_web3.precompiles import ecdh, aes_gcm_encrypt
from seismic_web3 import PrivateKey, CompressedPublicKey
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Derive shared key via ECDH
my_sk = PrivateKey.from_hex("0x1234...")
peer_pk = CompressedPublicKey.from_hex("0x03...")
aes_key = ecdh(w3, sk=my_sk, pk=peer_pk)

# Encrypt with derived key
plaintext = b"Encrypted for peer"
ciphertext = aes_gcm_encrypt(w3, aes_key=aes_key, nonce=1, plaintext=plaintext)
print(f"Ciphertext: {ciphertext.hex()}")
```

## How It Works

1. **Encode parameters** - Concatenates 32-byte key + 12-byte nonce + plaintext
2. **Call precompile** - Issues an `eth_call` to address `0x66` with estimated gas
3. **Encrypt data** - Precompile performs AES-256-GCM encryption
4. **Return ciphertext** - Returns encrypted data with 16-byte authentication tag appended

## Gas Cost

Gas cost is calculated as:

```python
base_gas = 1000
per_block = 30  # per 16-byte block
num_blocks = (len(plaintext) + 15) // 16
total_gas = base_gas + (num_blocks * per_block)
```

For example:

* 16 bytes plaintext: 1030 gas
* 64 bytes plaintext: 1120 gas
* 256 bytes plaintext: 1480 gas

## Notes

* Uses AES-256-GCM authenticated encryption
* Nonce must be unique for each encryption with the same key
* Ciphertext length = plaintext length + 16 bytes (authentication tag)
* The authentication tag ensures ciphertext integrity
* Reusing a nonce with the same key breaks security

## Warnings

* **Nonce reuse** - NEVER reuse the same nonce with the same key (breaks confidentiality)
* **Key security** - Keep AES keys secure and never expose them
* **Authentication tag** - The 16-byte tag is appended to ciphertext and must be included during decryption
* **Counter management** - When using integer nonces, ensure they're sequential and never repeat

## See Also

* [aes-gcm-decrypt](aes-gcm-decrypt.md) - Decrypt AES-GCM ciphertext
* [ecdh](ecdh.md) - Derive shared encryption keys
* [Bytes32](../api-reference/bytes32/) - 32-byte AES key type
* [EncryptionNonce](../api-reference/bytes32/encryption-nonce.md) - 12-byte nonce type
