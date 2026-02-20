---
description: Holds AES key and encryption keypair derived from ECDH
icon: key
---

# EncryptionState

Holds the AES-GCM key and encryption keypair derived from ECDH key exchange.

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

Decrypt ciphertext with metadata-bound AAD.

#### Signature

```python
def decrypt(
    self,
    ciphertext: HexBytes,
    nonce: EncryptionNonce,
    metadata: TxSeismicMetadata,
) -> HexBytes
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `ciphertext` | `HexBytes` | Encrypted data (includes 16-byte auth tag) |
| `nonce` | [`EncryptionNonce`](../api-reference/types/encryption-nonce.md) | 12-byte AES-GCM nonce |
| `metadata` | [`TxSeismicMetadata`](../api-reference/transaction-types/tx-seismic-metadata.md) | Transaction metadata (used to build AAD) |

#### Returns

| Type | Description |
|------|-------------|
| `HexBytes` | Decrypted plaintext |

#### Raises

- `cryptography.exceptions.InvalidTag` - If authentication fails (wrong key, tampered data, or mismatched metadata)

#### Example

```python
from seismic_web3 import get_encryption
from cryptography.exceptions import InvalidTag

encryption = get_encryption(tee_public_key, client_private_key)

try:
    plaintext = encryption.decrypt(
        ciphertext=encrypted_data,
        nonce=nonce,
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

private_key = PrivateKey(bytes.fromhex(os.environ["PRIVATE_KEY"]))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Access encryption state
encryption = w3.seismic.encryption

print(f"AES key: {encryption.aes_key.to_0x_hex()}")
print(f"Client pubkey: {encryption.encryption_pubkey.to_0x_hex()}")
```

### Manual Encryption Workflow

```python
from seismic_web3 import get_encryption, PrivateKey, CompressedPublicKey
from hexbytes import HexBytes
import os

# Get TEE public key from node
tee_pk = CompressedPublicKey("0x02abcd...")

# Create encryption state
client_sk = PrivateKey(os.urandom(32))
encryption = get_encryption(tee_pk, client_sk)

# Encrypt some data
plaintext = HexBytes("0x1234abcd")
nonce = os.urandom(12)

ciphertext = encryption.encrypt(
    plaintext=plaintext,
    nonce=nonce,
    metadata=metadata,
)

# Decrypt it back
decrypted = encryption.decrypt(
    ciphertext=ciphertext,
    nonce=nonce,
    metadata=metadata,
)

assert decrypted == plaintext
```

### Custom Encryption Key

```python
from seismic_web3 import get_encryption, PrivateKey, CompressedPublicKey
import os

# Use a deterministic key (e.g., derived from mnemonic)
client_sk = PrivateKey(bytes.fromhex("YOUR_DETERMINISTIC_KEY_HEX"))

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
import os

encryption = get_encryption(tee_pk, client_sk)

plaintext = b"Hello, Seismic!"
nonce = os.urandom(12)

# Encrypt
ciphertext = encryption.encrypt(plaintext, nonce, metadata)

# Decrypt with correct parameters
assert encryption.decrypt(ciphertext, nonce, metadata) == plaintext

# Decrypt with wrong nonce - should fail
wrong_nonce = os.urandom(12)
try:
    encryption.decrypt(ciphertext, wrong_nonce, metadata)
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

1. Encode metadata as AAD using [`encode_metadata_as_aad()`](../api-reference/transaction-types/tx-seismic-metadata.md)
2. Call `AesGcmCrypto.encrypt(plaintext, nonce, aad)`
3. Return ciphertext with 16-byte authentication tag

### Decryption

1. Encode metadata as AAD
2. Call `AesGcmCrypto.decrypt(ciphertext, nonce, aad)`
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

- **Key derivation** - AES key is derived from ECDH + HKDF, ensuring forward secrecy
- **AAD binding** - Metadata binding prevents ciphertext reuse or manipulation
- **Nonce uniqueness** - Nonces must be unique per encryption; SDK generates fresh nonces automatically
- **Key storage** - `encryption_private_key` should be stored securely if deterministic keys are used

## See Also

- [get_encryption](get-encryption.md) - Derive encryption state from TEE public key
- [create_wallet_client](create-wallet-client.md) - Sync client factory (creates EncryptionState)
- [create_async_wallet_client](create-async-wallet-client.md) - Async client factory
- [EncryptionNonce](../api-reference/types/encryption-nonce.md) - 12-byte nonce type
- [TxSeismicMetadata](../api-reference/transaction-types/tx-seismic-metadata.md) - Metadata structure
- [Shielded Write Guide](../shielded-write.md) - How shielded transactions work
