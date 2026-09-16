---
description: Holds AES key and encryption keypair derived from ECDH
icon: key
---

# EncryptionState

Holds the [AES](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard)-GCM key and encryption keypair derived from [ECDH](https://en.wikipedia.org/wiki/Elliptic-curve_Diffie%E2%80%93Hellman) key exchange.

## Overview

`EncryptionState` encapsulates all cryptographic material needed for shielded transactions and signed reads. It's created by [`get_encryption()`](get-encryption.md) during wallet client setup and attached to [`w3.seismic`](../namespaces/seismic-namespace.md)`.encryption`.

The class provides [`encrypt()`](#encrypt) and [`decrypt()`](#decrypt) methods that handle AES-GCM encryption with metadata-bound Additional Authenticated Data (AAD).

## Definition

```python
@dataclass
class EncryptionState:
    """Holds the AES key and encryption keypair derived from ECDH.

    Created by :func:`get_encryption` during client setup.  Pure
    computation - works in both sync and async contexts.

    Attributes:
        aes_key: 32-byte AES-256 key derived from ECDH + HKDF.
        encryption_pubkey: Client's compressed secp256k1 public key.
        encryption_private_key: Client's secp256k1 private key.
    """

    aes_key: Bytes32
    encryption_pubkey: CompressedPublicKey
    encryption_private_key: PrivateKey
```

## Attributes

| Attribute | Type | Description |
|-----------|------|-------------|
| `aes_key` | [`Bytes32`](../api-reference/types/bytes32.md) | 32-byte AES-256 key derived from ECDH + [HKDF](https://en.wikipedia.org/wiki/HKDF) |
| `encryption_pubkey` | [`CompressedPublicKey`](../api-reference/types/compressed-public-key.md) | Client's 33-byte compressed secp256k1 public key |
| `encryption_private_key` | [`PrivateKey`](../api-reference/types/private-key.md) | Client's 32-byte secp256k1 private key |

## Methods

### encrypt()

Encrypt plaintext calldata with metadata-bound AAD.

#### Signature

```python
def encrypt(
    self,
    plaintext: HexBytes,
    nonce: EncryptionNonce,
    metadata: TxSeismicMetadata,
) -> HexBytes
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `plaintext` | `HexBytes` | Raw calldata to encrypt |
| `nonce` | [`EncryptionNonce`](../api-reference/types/encryption-nonce.md) | 12-byte AES-GCM nonce |
| `metadata` | [`TxSeismicMetadata`](../api-reference/transaction-types/tx-seismic-metadata.md) | Transaction metadata (used to build AAD) |

#### Returns

| Type | Description |
|------|-------------|
| `HexBytes` | Ciphertext with 16-byte authentication tag appended |

#### Example

```python
from seismic_web3 import get_encryption, EncryptionNonce
from hexbytes import HexBytes
import os

# Setup encryption state
encryption = get_encryption(tee_public_key, client_private_key)

# Encrypt calldata
plaintext = HexBytes("0x1234abcd...")
nonce = EncryptionNonce(os.urandom(12))

ciphertext = encryption.encrypt(
    plaintext=plaintext,
    nonce=nonce,
    metadata=tx_metadata,
)

# Ciphertext is len(plaintext) + 16 bytes (auth tag)
assert len(ciphertext) == len(plaintext) + 16
```

### decrypt()

Decrypt a signed-read response with metadata-bound AAD.

The node draws a fresh IV for every response and prepends it behind a one-byte format version, so the response is `version || iv || ciphertext || tag` and no nonce argument is needed.

#### Signature

```python
def decrypt(
    self,
    ciphertext: HexBytes,
    metadata: TxSeismicMetadata,
) -> HexBytes
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `ciphertext` | `HexBytes` | Response bytes of the form `version \|\| iv \|\| ciphertext \|\| tag` |
| `metadata` | [`TxSeismicMetadata`](../api-reference/transaction-types/tx-seismic-metadata.md) | Transaction metadata (used to build AAD) |

#### Returns

| Type | Description |
|------|-------------|
| `HexBytes` | Decrypted plaintext |

#### Raises

- `ValueError` - If the response is shorter than the 29-byte minimum envelope, or carries an unrecognised format version
- `cryptography.exceptions.InvalidTag` - If authentication fails (wrong key, tampered data, or mismatched metadata)

#### Example

```python
from seismic_web3 import get_encryption
from cryptography.exceptions import InvalidTag

encryption = get_encryption(tee_public_key, client_private_key)

try:
    plaintext = encryption.decrypt(
        ciphertext=encrypted_response,
        metadata=tx_metadata,
    )
    print(f"Decrypted: {plaintext.to_0x_hex()}")
except InvalidTag:
    print("Decryption failed: authentication tag mismatch")
```

## Examples

### Access from Client

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey

private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = create_wallet_client("https://testnet-1.seismictest.net/rpc", private_key=private_key)

# Access encryption state
encryption = w3.seismic.encryption

print(f"AES key: {encryption.aes_key.to_0x_hex()}")
print(f"Client pubkey: {encryption.encryption_pubkey.to_0x_hex()}")
```

### Manual Encryption Workflow

```python
import os
from seismic_web3 import get_encryption, PrivateKey, CompressedPublicKey
from hexbytes import HexBytes

# Get TEE public key from node
tee_pk = CompressedPublicKey("0x02abcd...")

# Create encryption state
client_sk = PrivateKey(os.urandom(32))
encryption = get_encryption(tee_pk, client_sk)

# Build transaction metadata (see TxSeismicMetadata docs)
metadata = ...  # TxSeismicMetadata for the transaction being encrypted

# Encrypt calldata for the request
plaintext = HexBytes("0x1234abcd")
nonce = os.urandom(12)

ciphertext = encryption.encrypt(
    plaintext=plaintext,
    nonce=nonce,
    metadata=metadata,
)

# Decrypt the node's signed-read response. `encrypt` and `decrypt` use
# separate directional keys, so a client cannot round-trip its own data
# through them -- only the node can produce input for `decrypt`.
response = ...  # raw bytes returned by eth_call: version || iv || ciphertext || tag
plaintext_response = encryption.decrypt(
    ciphertext=response,
    metadata=metadata,
)
```

### Custom Encryption Key

```python
import os
from seismic_web3 import get_encryption, PrivateKey, CompressedPublicKey

# Use a deterministic key (e.g., derived from mnemonic)
client_sk = PrivateKey.from_hex_str(os.environ["CLIENT_KEY"])

# Or use a random ephemeral key
# client_sk = PrivateKey(os.urandom(32))

tee_pk = CompressedPublicKey("0x02abcd...")
encryption = get_encryption(tee_pk, client_sk)

# Store client_sk securely if you need to recreate the same encryption state later
```

### Verify Encryption/Decryption

```python
from seismic_web3 import get_encryption, PrivateKey, CompressedPublicKey
from cryptography.exceptions import InvalidTag
from hexbytes import HexBytes

encryption = get_encryption(tee_pk, client_sk)

# `response` is what eth_call returned for a signed read.
plaintext = encryption.decrypt(response, metadata)

# Flipping any byte of the response fails authentication: the prepended
# IV is covered by the GCM tag, so it cannot be swapped either.
tampered = HexBytes(bytes([response[0] ^ 0x01]) + bytes(response[1:]))
try:
    encryption.decrypt(tampered, metadata)
    assert False, "Should have raised InvalidTag"
except InvalidTag:
    print("Authentication failed as expected")
```

## How It Works

### Initialization

When created, `EncryptionState` automatically initializes an internal `AesGcmCrypto` instance:

```python
def __post_init__(self) -> None:
    self._crypto = AesGcmCrypto(self.aes_key)
```

### Encryption

1. Encode metadata as AAD, with the response format version appended using [`encode_metadata_as_aad()`](../api-reference/transaction-types/tx-seismic-metadata.md)
2. Call `AesGcmCrypto.encrypt(plaintext, nonce, aad)`
3. Return ciphertext with 16-byte authentication tag

### Decryption

1. Encode metadata as AAD
2. Check the format version byte, split off the 12-byte IV, then call `AesGcmCrypto.decrypt(body, iv, aad)` with the version appended to the AAD
3. Verify authentication tag (raises `InvalidTag` if fails)
4. Return plaintext

### AAD Binding

The Additional Authenticated Data (AAD) ensures that ciphertext is cryptographically bound to transaction metadata:

- `message_version`
- `chain_id`
- `client_pubkey`
- `nonce_seed`
- `recent_block_hash`
- `expires_at_block`

If any metadata field changes, decryption will fail even with the correct key and nonce.

## Notes

- Pure computation - no I/O operations
- Works in both sync and async contexts
- Created automatically by [`create_wallet_client()`](create-wallet-client.md) and [`create_async_wallet_client()`](create-async-wallet-client.md)
- You rarely need to call [`encrypt()`](#encrypt) or [`decrypt()`](#decrypt) directly - the SDK handles this
- The internal `_crypto` field is excluded from `repr()` and comparison
- Authentication tag is always 16 bytes (AES-GCM standard)

## Security Considerations

- **Key derivation** - [AES](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard) key is derived from [ECDH](https://en.wikipedia.org/wiki/Elliptic-curve_Diffie%E2%80%93Hellman) + [HKDF](https://en.wikipedia.org/wiki/HKDF), ensuring forward secrecy
- **AAD binding** - Metadata binding prevents ciphertext reuse or manipulation
- **Nonce uniqueness** - Nonces must be unique per encryption; SDK generates fresh nonces automatically
- **Key storage** - `encryption_private_key` should be stored securely if deterministic keys are used

## See Also

- [get_encryption](get-encryption.md) - Derive encryption state from TEE public key
- [create_wallet_client](create-wallet-client.md) - Sync client factory (creates EncryptionState)
- [create_async_wallet_client](create-async-wallet-client.md) - Async client factory
- [EncryptionNonce](../api-reference/types/encryption-nonce.md) - 12-byte nonce type
- [TxSeismicMetadata](../api-reference/transaction-types/tx-seismic-metadata.md) - Metadata structure
- [Shielded Write Guide](../guides/shielded-write.md) - How shielded transactions work
