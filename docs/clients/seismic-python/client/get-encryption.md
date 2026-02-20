---
description: Derive encryption state from TEE public key using ECDH
icon: shield-halved
---

# get\_encryption

Derive encryption state from a TEE public key using ECDH key exchange.

## Overview

`get_encryption()` performs ECDH key exchange between a client private key and the TEE's public key to derive a shared AES-GCM key. It returns an [`EncryptionState`](encryption-state.md) object containing the AES key, client public key, and client private key.

This is a pure computation function with no I/O - it works in both sync and async contexts.

## Signature

```python
def get_encryption(
    network_pk: CompressedPublicKey,
    client_sk: PrivateKey | None = None,
) -> EncryptionState
```

## Parameters

| Parameter    | Type                                                                       | Required | Description                                                                         |
| ------------ | -------------------------------------------------------------------------- | -------- | ----------------------------------------------------------------------------------- |
| `network_pk` | [`CompressedPublicKey`](../api-reference/bytes32/compressed-public-key.md) | Yes      | The TEE's 33-byte compressed secp256k1 public key                                   |
| `client_sk`  | [`PrivateKey`](../api-reference/bytes32/private-key.md)                    | No       | Optional 32-byte client private key. If `None`, a random ephemeral key is generated |

## Returns

| Type                                     | Description                                                 |
| ---------------------------------------- | ----------------------------------------------------------- |
| [`EncryptionState`](encryption-state.md) | Fully initialized encryption state with AES key and keypair |

## Examples

### Basic Usage

```python
from seismic_web3 import get_encryption, PrivateKey, CompressedPublicKey

# Get TEE public key (from node)
tee_pk = CompressedPublicKey("0x02abcd...")

# Derive encryption state (random ephemeral key)
encryption = get_encryption(tee_pk)

print(f"AES key: {encryption.aes_key.to_0x_hex()}")
print(f"Client pubkey: {encryption.encryption_pubkey.to_0x_hex()}")
```

### With Custom Client Key

```python
from seismic_web3 import get_encryption, PrivateKey, CompressedPublicKey
import os

tee_pk = CompressedPublicKey("0x02abcd...")

# Use a deterministic client key
client_sk = PrivateKey(bytes.fromhex("YOUR_CLIENT_KEY_HEX"))

encryption = get_encryption(tee_pk, client_sk)
```

### In Client Factory

```python
from seismic_web3 import get_encryption, get_tee_public_key, PrivateKey
from web3 import Web3

# This is what create_wallet_client() does internally
w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Step 1: Fetch TEE public key
network_pk = get_tee_public_key(w3)

# Step 2: Derive encryption state
signing_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))
encryption = get_encryption(network_pk, client_sk=None)  # Random ephemeral key

# Step 3: Attach to client
# w3.seismic = SeismicNamespace(w3, encryption, signing_key)
```

### Random Ephemeral Key

```python
from seismic_web3 import get_encryption, CompressedPublicKey

tee_pk = CompressedPublicKey("0x02abcd...")

# Each call generates a new random key
encryption1 = get_encryption(tee_pk)
encryption2 = get_encryption(tee_pk)

# Different keys
assert encryption1.encryption_private_key != encryption2.encryption_private_key
assert encryption1.aes_key != encryption2.aes_key
```

### Deterministic Key from Mnemonic

```python
from seismic_web3 import get_encryption, PrivateKey, CompressedPublicKey
from eth_account import Account

# Derive deterministic key from mnemonic
mnemonic = "your twelve word mnemonic phrase here ..."
account = Account.from_mnemonic(mnemonic)

# Use account key for encryption (or derive a separate BIP-44 path)
client_sk = PrivateKey(account.key)

tee_pk = CompressedPublicKey("0x02abcd...")
encryption = get_encryption(tee_pk, client_sk)

# Same mnemonic will always produce same encryption state
```

### Verify Key Derivation

```python
from seismic_web3 import get_encryption, PrivateKey, CompressedPublicKey
from seismic_web3.crypto.secp import private_key_to_compressed_public_key

tee_pk = CompressedPublicKey("0x02abcd...")
client_sk = PrivateKey(bytes.fromhex("abcd..."))

encryption = get_encryption(tee_pk, client_sk)

# Verify public key derivation
expected_pubkey = private_key_to_compressed_public_key(client_sk)
assert encryption.encryption_pubkey == expected_pubkey

# Verify keys are stored correctly
assert encryption.encryption_private_key == client_sk
```

## How It Works

The function performs three steps:

1.  **Generate client key if needed**

    ```python
    if client_sk is None:
        client_sk = PrivateKey(os.urandom(32))
    ```
2.  **Derive AES key via ECDH + HKDF**

    ```python
    aes_key = generate_aes_key(client_sk, network_pk)
    ```

    This performs:

    * ECDH: Compute shared secret from `client_sk` and `network_pk`
    * HKDF: Derive 32-byte AES key from shared secret
3.  **Derive client public key**

    ```python
    client_pubkey = private_key_to_compressed_public_key(client_sk)
    ```
4.  **Return EncryptionState**

    ```python
    return EncryptionState(
        aes_key=aes_key,
        encryption_pubkey=client_pubkey,
        encryption_private_key=client_sk,
    )
    ```

## ECDH Key Exchange

The ECDH key exchange works as follows:

```
Client has:     client_sk (private), client_pk (public)
TEE has:        tee_sk (private), tee_pk (public)

Client computes:  shared_secret = ECDH(client_sk, tee_pk)
TEE computes:     shared_secret = ECDH(tee_sk, client_pk)

Both derive:      aes_key = HKDF(shared_secret)
```

The client sends `client_pk` in the transaction's `SeismicElements`, allowing the TEE to derive the same AES key and decrypt the calldata.

## Random vs Deterministic Keys

| Key Type             | Pros                                                          | Cons                                                   |
| -------------------- | ------------------------------------------------------------- | ------------------------------------------------------ |
| **Random ephemeral** | No key management needed, fresh key per session               | Cannot recreate encryption state, no key persistence   |
| **Deterministic**    | Can recreate same state, key persistence, backup via mnemonic | Requires secure key storage, key management complexity |

The SDK defaults to **random ephemeral keys** for simplicity. Use deterministic keys only if you need to recreate the same encryption state across sessions.

## Notes

* Pure computation - no RPC calls or I/O
* Works in both sync and async contexts
* If `client_sk` is `None`, generates a cryptographically secure random key via `os.urandom(32)`
* The AES key is derived using ECDH + HKDF (NIST SP 800-56C)
* Called automatically by [`create_wallet_client()`](create-wallet-client.md) and [`create_async_wallet_client()`](create-async-wallet-client.md)
* You rarely need to call this directly unless implementing custom client logic

## Security Considerations

* **Random key generation** - Uses `os.urandom()` which is cryptographically secure on all platforms
* **Forward secrecy** - Each encryption session can use a different ephemeral key
* **Key storage** - If using deterministic keys, store `client_sk` securely (encrypted at rest, never logged)
* **ECDH security** - Based on secp256k1 elliptic curve discrete logarithm problem
* **HKDF** - Uses SHA-256 for key derivation

## Common Patterns

### Ephemeral Session Keys

```python
from seismic_web3 import get_encryption, CompressedPublicKey

# New random key for each session (recommended)
def create_session_encryption(tee_pk: CompressedPublicKey):
    return get_encryption(tee_pk)  # Random key
```

### Persistent Keys

```python
from seismic_web3 import get_encryption, PrivateKey, CompressedPublicKey
import os

# Load persisted key from secure storage
def load_encryption(tee_pk: CompressedPublicKey):
    client_sk = PrivateKey(bytes.fromhex(os.environ["ENCRYPTION_KEY"]))
    return get_encryption(tee_pk, client_sk)
```

### Rotate Keys

```python
from seismic_web3 import get_encryption, PrivateKey, CompressedPublicKey

# Rotate to a new key periodically
def rotate_encryption_key(tee_pk: CompressedPublicKey):
    new_client_sk = PrivateKey(os.urandom(32))
    return get_encryption(tee_pk, new_client_sk)
```

## See Also

* [EncryptionState](encryption-state.md) - Returned encryption state class
* [create\_wallet\_client](create-wallet-client.md) - Sync client factory (calls get\_encryption)
* [create\_async\_wallet\_client](create-async-wallet-client.md) - Async client factory
* [PrivateKey](../api-reference/bytes32/private-key.md) - Client private key type
* [CompressedPublicKey](../api-reference/bytes32/compressed-public-key.md) - TEE public key type
* [Shielded Write Guide](../../../gitbook/client-libraries/seismic-python/shielded-write.md) - How encryption is used in transactions
