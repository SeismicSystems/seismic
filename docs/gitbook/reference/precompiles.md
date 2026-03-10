---
icon: microchip
---

# Precompiled Contracts

## Overview

Seismic adds six precompiled contracts to the EVM, giving smart contracts access to cryptographic primitives that would be prohibitively expensive to implement in Solidity. All standard Ethereum precompiles (ecRecover, SHA-256, RIPEMD-160, identity, modexp, ecAdd, ecMul, ecPairing, blake2f) remain available at their usual addresses.

Precompiles are called like regular contracts using `staticcall` or `call` to their fixed addresses. They execute native code rather than EVM bytecode, making them significantly cheaper and faster than equivalent Solidity implementations.

| Address | Name            | Purpose                                                    |
| ------- | --------------- | ---------------------------------------------------------- |
| `0x64`  | RNG             | Securely generate random bytes inside the TEE              |
| `0x65`  | ECDH            | Derive an AES key from a private key and a public key      |
| `0x66`  | AES-GCM Encrypt | Encrypt data with AES-256-GCM authenticated encryption     |
| `0x67`  | AES-GCM Decrypt | Decrypt data with AES-256-GCM authenticated encryption     |
| `0x68`  | HKDF            | Derive a 32-byte key from input key material               |
| `0x69`  | secp256k1 Sign  | Sign a message hash with a secp256k1 private key           |

{% hint style="warning" %}
All precompiles expect **raw concatenated bytes** as input, not ABI-encoded data. Use `abi.encodePacked` (not `abi.encode`) when constructing input in Solidity.
{% endhint %}

***

## RNG (`0x64`)

Securely generate random bytes inside the TEE. The randomness is derived from TEE-internal entropy combined with optional personalization data, ensuring that the output is unpredictable to all parties -- including the node operator.

### Input

Raw bytes in the following layout:

| Offset  | Field              | Type     | Description                                              |
| ------- | ------------------ | -------- | -------------------------------------------------------- |
| `[0:4]` | output length      | `uint32` | Number of random bytes to generate (big-endian)          |
| `[4:]`  | personalization    | `bytes`  | Optional personalization data to influence the RNG output |

### Output

| Field        | Type    | Description                                     |
| ------------ | ------- | ----------------------------------------------- |
| random bytes | `bytes` | Securely generated random bytes of the requested length |

### Use cases

* Fair randomness in games, lotteries, and raffles
* Shuffling hidden card decks or secret orderings
* Generating nonces for on-chain cryptographic operations

### Solidity example

```solidity
function getRandomBytes(uint32 numBytes) internal view returns (bytes memory) {
    (bool success, bytes memory result) = address(0x64).staticcall(
        abi.encodePacked(numBytes)
    );
    require(success, "RNG precompile failed");
    return result;
}

function getRandomBytesWithPersonalization(
    uint32 numBytes,
    bytes memory pers
) internal view returns (bytes memory) {
    (bool success, bytes memory result) = address(0x64).staticcall(
        abi.encodePacked(numBytes, pers)
    );
    require(success, "RNG precompile failed");
    return result;
}
```

***

## ECDH (`0x65`)

Elliptic Curve Diffie-Hellman key agreement on the secp256k1 curve, followed by HKDF key derivation. Given a private key and a public key, produces a derived AES-256 key that both parties can independently compute. This is the foundation for encrypted communication between a contract and a user.

{% hint style="info" %}
The output is a **derived AES key**, not the raw ECDH shared secret. The precompile internally runs ECDH followed by HKDF-SHA256 to produce a key suitable for AES-256-GCM encryption.
{% endhint %}

### Input

Raw bytes in the following layout (65 bytes total):

| Offset    | Field       | Type      | Description                                |
| --------- | ----------- | --------- | ------------------------------------------ |
| `[0:32]`  | private key | `bytes32` | secp256k1 private key (32 bytes)           |
| `[32:65]` | public key  | `bytes33` | Compressed secp256k1 public key (33 bytes) |

### Output

| Field       | Type      | Description                                  |
| ----------- | --------- | -------------------------------------------- |
| derived key | `bytes32` | A 32-byte AES key derived via ECDH + HKDF    |

### Use cases

* Key agreement between a contract and a user for encrypting event data
* Establishing shared encryption keys for private communication channels
* Enabling per-recipient encryption of on-chain data

### Solidity example

```solidity
function deriveAESKey(
    bytes32 privateKey,
    bytes memory compressedPubKey  // 33 bytes
) internal view returns (bytes32) {
    (bool success, bytes memory result) = address(0x65).staticcall(
        abi.encodePacked(privateKey, compressedPubKey)
    );
    require(success, "ECDH precompile failed");
    return bytes32(result);
}
```

***

## AES-GCM Encrypt (`0x66`)

Encrypt data using AES-256-GCM (Galois/Counter Mode) authenticated encryption. Produces ciphertext with an appended authentication tag that guarantees integrity and authenticity.

{% hint style="info" %}
Only AES-256 is supported. The key must be exactly 32 bytes.
{% endhint %}

### Input

Raw bytes in the following layout:

| Offset    | Field     | Type      | Description                      |
| --------- | --------- | --------- | -------------------------------- |
| `[0:32]`  | key       | `bytes32` | AES-256 key (32 bytes)           |
| `[32:44]` | nonce     | `bytes12` | Initialization vector (12 bytes) |
| `[44:]`   | plaintext | `bytes`   | The data to encrypt              |

### Output

| Field                 | Type    | Description                                         |
| --------------------- | ------- | --------------------------------------------------- |
| ciphertext + auth tag | `bytes` | Encrypted data concatenated with authentication tag |

### Use cases

* Encrypting sensitive event parameters before emission
* Building encrypted storage helpers
* Encrypting messages or data for specific recipients

### Solidity example

See [CryptoUtils.sol](https://github.com/SeismicSystems/seismic/blob/main/contracts/src/seismic-std-lib/utils/precompiles/CryptoUtils.sol) for the production implementation.

```solidity
function aesEncrypt(
    suint256 key,
    uint96 nonce,
    bytes memory plaintext
) internal view returns (bytes memory) {
    bytes memory input = abi.encodePacked(uint256(key), nonce, plaintext);
    (bool success, bytes memory result) = address(0x66).staticcall(input);
    require(success, "AES-GCM Encrypt precompile failed");
    return result;
}
```

***

## AES-GCM Decrypt (`0x67`)

Decrypt data that was encrypted with AES-256-GCM. Verifies the authentication tag before returning the plaintext. If the tag does not match (indicating tampering or wrong key), the call fails.

### Input

Raw bytes in the following layout:

| Offset    | Field                 | Type      | Description                                         |
| --------- | --------------------- | --------- | --------------------------------------------------- |
| `[0:32]`  | key                   | `bytes32` | AES-256 key (must match the key used for encryption) |
| `[32:44]` | nonce                 | `bytes12` | Initialization vector (12 bytes)                    |
| `[44:]`   | ciphertext + auth tag | `bytes`   | Encrypted data concatenated with authentication tag |

### Output

| Field     | Type    | Description    |
| --------- | ------- | -------------- |
| plaintext | `bytes` | Decrypted data |

### Use cases

* On-chain decryption of previously encrypted data
* Decrypting messages from other contracts or users
* Verifying and reading encrypted event data within a contract

### Solidity example

```solidity
function aesDecrypt(
    suint256 key,
    uint96 nonce,
    bytes memory ciphertext
) internal view returns (bytes memory) {
    bytes memory input = abi.encodePacked(uint256(key), nonce, ciphertext);
    (bool success, bytes memory result) = address(0x67).staticcall(input);
    require(success, "AES-GCM Decrypt precompile failed");
    return result;
}
```

***

## HKDF (`0x68`)

HMAC-based Key Derivation Function ([RFC 5869](https://datatracker.ietf.org/doc/html/rfc5869)). Derives a 32-byte cryptographic key from input key material. Commonly used to turn a shared secret (from ECDH) into an encryption key suitable for AES-GCM.

{% hint style="info" %}
The Seismic HKDF precompile uses hardcoded parameters: salt is `None`, info is a fixed internal string, and the output is always 32 bytes. The only user-supplied input is the raw key material.
{% endhint %}

### Input

| Field              | Type    | Description                                           |
| ------------------ | ------- | ----------------------------------------------------- |
| input key material | `bytes` | The source key material (e.g., raw ECDH shared point) |

### Output

| Field       | Type      | Description                       |
| ----------- | --------- | --------------------------------- |
| derived key | `bytes32` | The derived 32-byte key           |

### Use cases

* Deriving AES encryption keys from shared secrets
* Key derivation with domain separation

### Solidity example

```solidity
function deriveKey(bytes memory ikm) internal view returns (bytes32) {
    (bool success, bytes memory result) = address(0x68).staticcall(ikm);
    require(success, "HKDF precompile failed");
    require(result.length == 32, "Invalid HKDF output length");
    return bytes32(result);
}
```

***

## secp256k1 Sign (`0x69`)

Sign a 32-byte message hash using a secp256k1 private key. Produces a 65-byte signature in the (r, s, v) format. This allows contracts to generate signatures on-chain -- something that is normally only possible off-chain with a user's wallet.

{% hint style="warning" %}
This precompile gives the contract the ability to sign arbitrary messages. The private key used must be one the contract has access to (e.g., derived from a seed stored in shielded storage). It does **not** use the caller's private key.
{% endhint %}

### Input

Raw bytes in the following layout (64 bytes total):

| Offset    | Field        | Type      | Description                             |
| --------- | ------------ | --------- | --------------------------------------- |
| `[0:32]`  | private key  | `bytes32` | The secp256k1 private key (32 bytes)    |
| `[32:64]` | message hash | `bytes32` | The 32-byte hash of the message to sign |

### Output

| Field     | Type      | Description                       |
| --------- | --------- | --------------------------------- |
| signature | `bytes65` | The signature in (r, s, v) format |

### Use cases

* Generating on-chain attestations or proofs
* Building contracts that can act as signers (e.g., smart-contract wallets)
* Creating signed messages for cross-chain verification

### Solidity example

```solidity
function signMessage(
    bytes32 privateKey,
    bytes32 messageHash
) internal view returns (bytes memory) {
    (bool success, bytes memory result) = address(0x69).staticcall(
        abi.encodePacked(privateKey, messageHash)
    );
    require(success, "secp256k1 Sign precompile failed");
    return result;
}
```

***

## Common pattern: Encrypted events

A frequent use of the precompiles is encrypting event data so that only the intended recipient can read it. The typical flow chains ECDH and AES-GCM Encrypt together:

```solidity
// 1. Derive an AES key via ECDH (private key first, then public key)
(bool ok1, bytes memory aesKeyBytes) = address(0x65).staticcall(
    abi.encodePacked(contractPrivKey, recipientCompressedPubKey)
);
require(ok1, "ECDH failed");
suint256 aesKey = suint256(bytes32(aesKeyBytes));

// 2. Generate a random nonce
uint96 nonce = CryptoUtils.generateRandomNonce();

// 3. Encrypt the sensitive data
bytes memory input = abi.encodePacked(uint256(aesKey), nonce, plaintext);
(bool ok2, bytes memory encrypted) = address(0x66).staticcall(input);
require(ok2, "AES encrypt failed");

// 4. Emit a regular event with the encrypted bytes
emit EncryptedData(msg.sender, recipient, nonce, encrypted);
```

For a complete walkthrough of this pattern, see [Events](../seismic-solidity/events.md).
