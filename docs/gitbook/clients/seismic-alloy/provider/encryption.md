---
description: TEE key exchange, ECDH shared secret derivation, and AES-GCM calldata encryption
icon: key
---

# Encryption

Detailed documentation of the encryption flow used by `SeismicSignedProvider` to protect calldata and decrypt responses.

## Overview

Seismic uses end-to-end encryption between the client and the node's Trusted Execution Environment (TEE). The encryption ensures that:

1. **Calldata confidentiality** -- Transaction input data is encrypted before leaving the client
2. **Context binding** -- Ciphertext is cryptographically bound to the transaction context (sender, nonce, chain ID) via Additional Authenticated Data (AAD)
3. **Response privacy** -- `seismic_call` responses are encrypted by the TEE and only the requesting client can decrypt them

The encryption scheme uses ECDH key agreement to derive a shared AES-GCM key, which is then used for symmetric encryption of calldata and decryption of responses.

## Encryption Flow

```
Client                                          Seismic Node (TEE)
------                                          ------------------

1. Generate ephemeral secp256k1 keypair
   (ephemeral_sk, ephemeral_pk)

2. Fetch TEE public key  ----RPC----->
   seismic_getTeePublicKey()
                          <----RPC-----  tee_pubkey

3. ECDH(ephemeral_sk, tee_pubkey)
   = shared_point

4. Derive AES key from shared_point
   aes_key = KDF(shared_point)

5. For each transaction:
   a. Build AAD from tx context
      (sender, chain_id, nonce, to, value, seismic_elements)
   b. Generate encryption nonce (12 bytes)
   c. AES-GCM-encrypt(calldata, aes_key, nonce, AAD)
   d. Send encrypted tx  ----RPC----->
                                              e. Derive same shared_point
                                                 ECDH(tee_sk, ephemeral_pk)
                                              f. Derive same AES key
                                              g. Rebuild AAD from tx
                                              h. AES-GCM-decrypt(ciphertext)
                                              i. Execute with plaintext calldata

6. For seismic_call responses:
                          <----RPC-----  encrypted_response
   a. AES-GCM-decrypt(response, aes_key, nonce, AAD)
   b. Return plaintext result
```

## Key Generation

### Ephemeral Keypair

At provider creation, `SeismicSignedProvider` generates a fresh secp256k1 keypair:

```
ephemeral_sk  : 32-byte secp256k1 private key (random)
ephemeral_pk  : 33-byte compressed secp256k1 public key (derived from ephemeral_sk)
```

This keypair is:

- Generated once per provider instance
- Used for all ECDH key agreements with the TEE
- Not related to the wallet's signing key
- Ephemeral -- discarded when the provider is dropped

### TEE Public Key

The node's TEE public key is fetched via the `seismic_getTeePublicKey` RPC method:

```rust
let tee_pubkey = provider.get_tee_pubkey().await?;
```

For `SeismicSignedProvider`:

- Fetched automatically during `new()` construction
- Cached for the lifetime of the provider
- Can be pre-fetched and passed to `new_with_tee_pubkey()` to avoid the async RPC call

For `SeismicUnsignedProvider`:

- Not fetched or cached automatically
- Available via `get_tee_pubkey()` for manual use

## ECDH Shared Secret

The shared secret is derived using Elliptic Curve Diffie-Hellman (ECDH) on the secp256k1 curve:

```
shared_point = ECDH(ephemeral_sk, tee_pubkey)
             = ephemeral_sk * tee_pubkey
```

The TEE performs the same computation with its own private key:

```
shared_point = ECDH(tee_sk, ephemeral_pk)
             = tee_sk * ephemeral_pk
```

Both sides arrive at the same shared point due to the commutativity of ECDH:

```
ephemeral_sk * tee_pubkey = tee_sk * ephemeral_pk
```

## AES-GCM Encryption

### Key Derivation

The AES-256 key is derived from the ECDH shared point using a Key Derivation Function (KDF):

```
aes_key = KDF(shared_point)  // 32 bytes for AES-256
```

### Encryption Parameters

| Parameter   | Size     | Description                                         |
| ----------- | -------- | --------------------------------------------------- |
| `aes_key`   | 32 bytes | AES-256 key derived from ECDH shared secret         |
| `nonce`     | 12 bytes | AES-GCM nonce (unique per encryption)               |
| `plaintext` | Variable | Transaction calldata to encrypt                     |
| `aad`       | Variable | Additional Authenticated Data (transaction context) |

### Encryption Operation

```
ciphertext || auth_tag = AES-GCM-Encrypt(aes_key, nonce, plaintext, aad)
```

The output is the concatenation of:

- **Ciphertext** -- same length as the plaintext
- **Authentication tag** -- 16 bytes (AES-GCM standard)

### Decryption Operation

```
plaintext = AES-GCM-Decrypt(aes_key, nonce, ciphertext || auth_tag, aad)
```

Decryption fails (raises an error) if:

- The AES key is incorrect
- The nonce does not match
- The AAD does not match (transaction context was altered)
- The ciphertext or authentication tag was tampered with

## Additional Authenticated Data (AAD)

The AAD cryptographically binds the ciphertext to the transaction context. This prevents:

- **Replay attacks** -- Ciphertext cannot be reused in a different transaction
- **Context manipulation** -- Changing any transaction field invalidates the ciphertext
- **Cross-chain attacks** -- Ciphertext is bound to a specific chain ID

### AAD Composition

The AAD is constructed from the following transaction fields, RLP-encoded:

| Field              | Type        | Description                                                                      |
| ------------------ | ----------- | -------------------------------------------------------------------------------- |
| `sender`           | `Address`   | Transaction sender address                                                       |
| `chain_id`         | `u64`       | Chain ID of the target network                                                   |
| `nonce`            | `u64`       | Transaction nonce                                                                |
| `to`               | `Address`   | Transaction recipient address                                                    |
| `value`            | `U256`      | Transaction value in wei                                                         |
| `seismic_elements` | RLP-encoded | Seismic-specific transaction fields (encryption nonce, block hash, expiry, etc.) |

{% hint style="info" %}
If any AAD field changes between encryption and decryption, the authentication tag verification will fail and decryption will return an error. This is a security feature -- it ensures the ciphertext can only be used in the exact transaction context it was encrypted for.
{% endhint %}

### AAD Construction

The AAD fields are RLP-encoded into a single byte sequence:

```
aad = RLP(sender, chain_id, nonce, to, value, seismic_elements)
```

The `seismic_elements` field itself contains:

- Encryption nonce (12 bytes)
- Client's ephemeral public key
- Recent block hash
- Expiry block number

## Response Decryption

For `seismic_call()` requests, the TEE encrypts the response using the same shared secret:

1. The TEE computes `ECDH(tee_sk, ephemeral_pk)` to get the shared point
2. Derives the same AES key
3. Encrypts the call response with AES-GCM
4. Returns the encrypted response to the client

The `SeismicSignedProvider` decrypts the response:

```rust
// Internally, seismic_call() does:
// 1. Fill transaction fields (filler pipeline)
// 2. Encrypt calldata with AES-GCM
// 3. Send encrypted call to node
// 4. Receive encrypted response
// 5. Decrypt response with the same AES key
let result = provider.seismic_call(tx.into()).await?;
```

{% hint style="info" %}
`SeismicUnsignedProvider` cannot decrypt responses because it does not have an ephemeral keypair. Use `SeismicSignedProvider` for any operation that requires reading encrypted responses.
{% endhint %}

## Provider Comparison

| Encryption Capability        | SeismicSignedProvider        | SeismicUnsignedProvider     |
| ---------------------------- | ---------------------------- | --------------------------- |
| Ephemeral keypair generation | Yes (at construction)        | No                          |
| TEE pubkey fetching          | Yes (cached at construction) | Yes (on-demand, not cached) |
| ECDH shared secret           | Yes                          | No                          |
| Calldata encryption          | Yes (automatic via fillers)  | No                          |
| Response decryption          | Yes (in `seismic_call`)      | No                          |

## Security Properties

### Forward Secrecy

Each `SeismicSignedProvider` instance generates a fresh ephemeral keypair. Compromising one provider's ephemeral key does not affect other providers or past sessions.

### Nonce Uniqueness

AES-GCM requires unique nonces for each encryption under the same key. The Seismic filler pipeline generates fresh 12-byte nonces for each transaction, ensuring nonce uniqueness.

{% hint style="warning" %}
Reusing an AES-GCM nonce with the same key is catastrophic -- it allows an attacker to recover the plaintext. The SDK handles nonce generation automatically. Do not manually set encryption nonces unless you have a strong reason and understand the implications.
{% endhint %}

### Authentication

The AES-GCM authentication tag ensures:

- The ciphertext has not been modified
- The AAD (transaction context) has not been modified
- The encryption key is correct

Any tampering with the ciphertext, AAD, or key results in a decryption failure.

### Key Separation

The ephemeral keypair used for ECDH is separate from the wallet's signing key. Compromising the wallet key does not compromise past encrypted calldata (and vice versa).

## Examples

### Inspecting TEE Public Key

```rust
use seismic_alloy::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://node.seismicdev.net/rpc".parse()?;
    let provider = sreth_unsigned_provider(url);

    let tee_pubkey = provider.get_tee_pubkey().await?;
    println!("TEE public key: {tee_pubkey}");

    Ok(())
}
```

### Signed Provider with Automatic Encryption

```rust
use seismic_alloy::prelude::*;
use alloy_signer_local::PrivateKeySigner;
use alloy_primitives::address;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = "0xYOUR_PRIVATE_KEY".parse()?;
    let wallet = SeismicWallet::from(signer);
    let url = "https://node.seismicdev.net/rpc".parse()?;

    // Provider automatically:
    // 1. Generates ephemeral keypair
    // 2. Fetches and caches TEE pubkey
    // 3. Derives ECDH shared secret
    let provider = sreth_signed_provider(wallet, url).await?;

    // Transaction calldata is automatically encrypted by the filler pipeline
    let tx = TransactionRequest::default()
        .to(address!("0x1234567890abcdef1234567890abcdef12345678"))
        .input(bytes!("0x60fe47b10000000000000000000000000000000000000000000000000000000000000042").into());

    // send_transaction: fillers populate fields, encrypt calldata, sign, send
    let pending = provider.send_transaction(tx).await?;
    let tx_hash = pending.watch().await?;

    // seismic_call: encrypt calldata, send, decrypt response
    let read_tx = TransactionRequest::default()
        .to(address!("0x1234567890abcdef1234567890abcdef12345678"))
        .input(bytes!("0x3fb5c1cb").into());

    let result = provider.seismic_call(read_tx.into()).await?;
    println!("Decrypted result: {result}");

    Ok(())
}
```

## Notes

- Encryption is handled automatically by the filler pipeline -- you do not need to encrypt calldata manually
- The ephemeral keypair is generated per provider instance, not per transaction
- The TEE public key is assumed to be static for the lifetime of a provider instance
- All encryption uses AES-256-GCM (256-bit key, 96-bit nonce, 128-bit authentication tag)
- The ECDH key agreement uses the secp256k1 curve (same curve as Ethereum signatures)
- The `seismic-enclave` crate (v0.1.0) provides the underlying cryptographic operations

## See Also

- [SeismicSignedProvider](seismic-signed-provider.md) -- Provider with full encryption capabilities
- [SeismicUnsignedProvider](seismic-unsigned-provider.md) -- Provider without encryption
- [Provider Overview](./) -- Comparison of provider types and filler pipelines
- [Installation](../installation.md) -- Add seismic-alloy to your project
