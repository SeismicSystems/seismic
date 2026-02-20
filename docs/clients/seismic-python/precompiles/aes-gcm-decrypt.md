---
description: On-chain AES-256-GCM decryption
icon: unlock
---

# aes\_gcm\_decrypt

Decrypt data on-chain using Mercury EVM's AES-GCM decryption precompile.

## Overview

`aes_gcm_decrypt()` and `async_aes_gcm_decrypt()` call the AES-GCM decryption precompile at address `0x67` to decrypt ciphertext using AES-256-GCM. The ciphertext must include the 16-byte authentication tag.

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

| Parameter    | Type                                                                       | Required | Description                                           |
| ------------ | -------------------------------------------------------------------------- | -------- | ----------------------------------------------------- |
| `w3`         | `Web3` or `AsyncWeb3`                                                      | Yes      | Web3 instance connected to a Seismic node             |
| `aes_key`    | [`Bytes32`](../api-reference/bytes32/)                                     | Yes      | 32-byte AES-256 decryption key                        |
| `nonce`      | `int` or [`EncryptionNonce`](../api-reference/bytes32/encryption-nonce.md) | Yes      | 12-byte nonce (must match encryption nonce)           |
| `ciphertext` | `bytes`                                                                    | Yes      | Data to decrypt (includes 16-byte authentication tag) |

## Returns

| Type       | Description               |
| ---------- | ------------------------- |
| `HexBytes` | Decrypted plaintext bytes |

## Examples

### Basic Usage

```python
from seismic_web3.precompiles import aes_gcm_encrypt, aes_gcm_decrypt
from seismic_web3 import Bytes32
from web3 import Web3
import os

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Generate AES key
aes_key = Bytes32(os.urandom(32))

# Encrypt then decrypt
plaintext = b"Secret message"
ciphertext = aes_gcm_encrypt(w3, aes_key=aes_key, nonce=1, plaintext=plaintext)
decrypted = aes_gcm_decrypt(w3, aes_key=aes_key, nonce=1, ciphertext=ciphertext)

assert decrypted == plaintext
print(f"Decrypted: {decrypted.decode()}")
```

### With Integer Nonce

```python
from seismic_web3.precompiles import aes_gcm_encrypt, aes_gcm_decrypt
from seismic_web3 import Bytes32
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

aes_key = Bytes32.from_hex("0x1234...")

# Encrypt with integer nonce
plaintext = b"Test message"
ciphertext = aes_gcm_encrypt(w3, aes_key=aes_key, nonce=42, plaintext=plaintext)

# Decrypt with same nonce
decrypted = aes_gcm_decrypt(w3, aes_key=aes_key, nonce=42, ciphertext=ciphertext)
print(f"Decrypted: {decrypted}")
```

### With EncryptionNonce

```python
from seismic_web3.precompiles import aes_gcm_encrypt, aes_gcm_decrypt
from seismic_web3 import Bytes32, EncryptionNonce
from web3 import Web3
import os

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

aes_key = Bytes32.from_hex("0x1234...")
nonce = EncryptionNonce(os.urandom(12))

# Encrypt and decrypt with EncryptionNonce
plaintext = b"Secure data"
ciphertext = aes_gcm_encrypt(w3, aes_key=aes_key, nonce=nonce, plaintext=plaintext)
decrypted = aes_gcm_decrypt(w3, aes_key=aes_key, nonce=nonce, ciphertext=ciphertext)

assert decrypted == plaintext
```

### Async Usage

```python
from seismic_web3.precompiles import async_aes_gcm_encrypt, async_aes_gcm_decrypt
from seismic_web3 import Bytes32
from web3 import AsyncWeb3
import os

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))

    aes_key = Bytes32(os.urandom(32))
    plaintext = b"Async secret"

    # Encrypt and decrypt asynchronously
    ciphertext = await async_aes_gcm_encrypt(
        w3, aes_key=aes_key, nonce=1, plaintext=plaintext
    )
    decrypted = await async_aes_gcm_decrypt(
        w3, aes_key=aes_key, nonce=1, ciphertext=ciphertext
    )

    assert decrypted == plaintext
    print(f"Decrypted: {decrypted}")

# Run with asyncio.run(main())
```

### Decrypt Multiple Messages

```python
from seismic_web3.precompiles import aes_gcm_encrypt, aes_gcm_decrypt
from seismic_web3 import Bytes32
from web3 import Web3
import os

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

aes_key = Bytes32(os.urandom(32))

# Encrypt multiple messages
messages = [b"Message 1", b"Message 2", b"Message 3"]
ciphertexts = [
    aes_gcm_encrypt(w3, aes_key=aes_key, nonce=i, plaintext=msg)
    for i, msg in enumerate(messages)
]

# Decrypt all messages
decrypted = [
    aes_gcm_decrypt(w3, aes_key=aes_key, nonce=i, ciphertext=ct)
    for i, ct in enumerate(ciphertexts)
]

for i, msg in enumerate(decrypted):
    print(f"Message {i}: {msg}")
```

### Handle Decryption Failure

```python
from seismic_web3.precompiles import aes_gcm_decrypt
from seismic_web3 import Bytes32
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

aes_key = Bytes32.from_hex("0x1234...")
bad_ciphertext = b"invalid ciphertext data"

# Decryption will fail if authentication tag doesn't verify
try:
    decrypted = aes_gcm_decrypt(w3, aes_key=aes_key, nonce=1, ciphertext=bad_ciphertext)
except Exception as e:
    print(f"Decryption failed: {e}")
```

### With ECDH-Derived Key

```python
from seismic_web3.precompiles import ecdh, aes_gcm_encrypt, aes_gcm_decrypt
from seismic_web3 import PrivateKey, CompressedPublicKey
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Alice encrypts for Bob
alice_sk = PrivateKey.from_hex("0x1234...")
bob_pk = CompressedPublicKey.from_hex("0x03...")
alice_key = ecdh(w3, sk=alice_sk, pk=bob_pk)

plaintext = b"Message for Bob"
ciphertext = aes_gcm_encrypt(w3, aes_key=alice_key, nonce=1, plaintext=plaintext)

# Bob decrypts from Alice
bob_sk = PrivateKey.from_hex("0x5678...")
alice_pk = CompressedPublicKey.from_hex("0x02...")
bob_key = ecdh(w3, sk=bob_sk, pk=alice_pk)

decrypted = aes_gcm_decrypt(w3, aes_key=bob_key, nonce=1, ciphertext=ciphertext)
assert decrypted == plaintext
print(f"Bob received: {decrypted}")
```

### Convert Result to String

```python
from seismic_web3.precompiles import aes_gcm_decrypt
from seismic_web3 import Bytes32
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

aes_key = Bytes32.from_hex("0x1234...")
ciphertext = bytes.fromhex("...")  # From previous encryption

# Decrypt and convert to string
decrypted = aes_gcm_decrypt(w3, aes_key=aes_key, nonce=1, ciphertext=ciphertext)
message = decrypted.decode("utf-8")
print(f"Message: {message}")
```

## How It Works

1. **Encode parameters** - Concatenates 32-byte key + 12-byte nonce + ciphertext (with tag)
2. **Call precompile** - Issues an `eth_call` to address `0x67` with estimated gas
3. **Decrypt and verify** - Precompile performs AES-256-GCM decryption and verifies authentication tag
4. **Return plaintext** - Returns decrypted data if tag verification succeeds

## Gas Cost

Gas cost is calculated as:

```python
base_gas = 1000
per_block = 30  # per 16-byte block
num_blocks = (len(ciphertext) + 15) // 16
total_gas = base_gas + (num_blocks * per_block)
```

The gas cost is proportional to ciphertext length (including the 16-byte tag).

## Notes

* Uses AES-256-GCM authenticated decryption
* Nonce must exactly match the nonce used during encryption
* Ciphertext must include the 16-byte authentication tag (appended by encryption)
* Decryption fails if the authentication tag doesn't verify
* Plaintext length = ciphertext length - 16 bytes (authentication tag)

## Warnings

* **Authentication failure** - If the tag doesn't verify, the precompile reverts (wrong key/nonce/tampered data)
* **Nonce mismatch** - Using a different nonce than encryption will cause decryption to fail
* **Key mismatch** - Using a different key than encryption will cause authentication failure
* **Ciphertext integrity** - Any modification to ciphertext causes authentication failure

## See Also

* [aes-gcm-encrypt](aes-gcm-encrypt.md) - Encrypt with AES-GCM
* [ecdh](ecdh.md) - Derive shared decryption keys
* [Bytes32](../api-reference/bytes32/) - 32-byte AES key type
* [EncryptionNonce](../api-reference/bytes32/encryption-nonce.md) - 12-byte nonce type
