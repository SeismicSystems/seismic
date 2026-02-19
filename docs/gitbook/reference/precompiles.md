---
icon: microchip
---

# Precompiles

## Overview

Seismic adds six precompiled contracts to the EVM, giving smart contracts access to cryptographic primitives that would be prohibitively expensive to implement in Solidity. All standard Ethereum precompiles (ecRecover, SHA-256, RIPEMD-160, identity, modexp, ecAdd, ecMul, ecPairing, blake2f) remain available at their usual addresses.

Precompiles are called like regular contracts using `staticcall` or `call` to their fixed addresses. They execute native code rather than EVM bytecode, making them significantly cheaper and faster than equivalent Solidity implementations.

| Address | Name            | Purpose                                                    |
| ------- | --------------- | ---------------------------------------------------------- |
| `0x64`  | RNG             | Securely generate a random number inside the TEE           |
| `0x65`  | ECDH            | Derive a shared secret from a public key and a private key |
| `0x66`  | AES-GCM Encrypt | Encrypt data with AES-GCM authenticated encryption         |
| `0x67`  | AES-GCM Decrypt | Decrypt data with AES-GCM authenticated encryption         |
| `0x68`  | HKDF            | Derive cryptographic keys from input key material          |
| `0x69`  | secp256k1 Sign  | Sign a message hash with a secp256k1 private key           |

---

## RNG (`0x64`)

Securely generate a random number inside the TEE. The randomness is derived from the provided seed combined with TEE-internal entropy, ensuring that the output is unpredictable to all parties -- including the node operator.

### Input

| Field | Type    | Description                                           |
| ----- | ------- | ----------------------------------------------------- |
| seed  | `bytes` | Arbitrary seed bytes used to influence the RNG output |

### Output

| Field        | Type    | Description                     |
| ------------ | ------- | ------------------------------- |
| random bytes | `bytes` | Securely generated random bytes |

### Use cases

- Fair randomness in games, lotteries, and raffles
- Shuffling hidden card decks or secret orderings
- Generating nonces for on-chain cryptographic operations

### Solidity example

```solidity
function getRandomNumber(bytes memory seed) internal view returns (bytes memory) {
    (bool success, bytes memory result) = address(0x64).staticcall(
        abi.encode(seed)
    );
    require(success, "RNG precompile failed");
    return result;
}
```

---

## ECDH (`0x65`)

Elliptic Curve Diffie-Hellman key agreement on the secp256k1 curve. Given a public key and a private key, produces a shared secret that both parties can independently derive. This is the foundation for encrypted communication between a contract and a user.

### Input

| Field       | Type      | Description                                |
| ----------- | --------- | ------------------------------------------ |
| public key  | `bytes33` | Compressed secp256k1 public key (33 bytes) |
| private key | `bytes32` | secp256k1 private key (32 bytes)           |

### Output

| Field         | Type      | Description                          |
| ------------- | --------- | ------------------------------------ |
| shared secret | `bytes32` | The derived shared secret (32 bytes) |

### Use cases

- Key agreement between a contract and a user for encrypting event data
- Establishing shared secrets for private communication channels
- Enabling per-recipient encryption of on-chain data

### Solidity example

```solidity
function deriveSharedSecret(
    bytes memory recipientPubKey,
    bytes32 senderPrivKey
) internal view returns (bytes32) {
    (bool success, bytes memory result) = address(0x65).staticcall(
        abi.encode(recipientPubKey, senderPrivKey)
    );
    require(success, "ECDH precompile failed");
    return abi.decode(result, (bytes32));
}
```

---

## AES-GCM Encrypt (`0x66`)

Encrypt data using AES-GCM (Galois/Counter Mode) authenticated encryption. Supports both AES-128 and AES-256 depending on the key length. Produces ciphertext with an authentication tag that guarantees integrity and authenticity.

### Input

| Field     | Type      | Description                                                            |
| --------- | --------- | ---------------------------------------------------------------------- |
| key       | `bytes`   | AES key (16 bytes for AES-128, 32 bytes for AES-256)                   |
| nonce     | `bytes12` | Initialization vector / nonce (12 bytes)                               |
| plaintext | `bytes`   | The data to encrypt                                                    |
| aad       | `bytes`   | Additional authenticated data (AAD) -- authenticated but not encrypted |

### Output

| Field                 | Type    | Description                                         |
| --------------------- | ------- | --------------------------------------------------- |
| ciphertext + auth tag | `bytes` | Encrypted data concatenated with authentication tag |

### Use cases

- Encrypting sensitive event parameters before emission
- Building encrypted storage helpers
- Encrypting messages or data for specific recipients

### Solidity example

```solidity
function aesEncrypt(
    bytes memory key,
    bytes12 nonce,
    bytes memory plaintext,
    bytes memory aad
) internal view returns (bytes memory) {
    (bool success, bytes memory result) = address(0x66).staticcall(
        abi.encode(key, nonce, plaintext, aad)
    );
    require(success, "AES-GCM Encrypt precompile failed");
    return result;
}
```

---

## AES-GCM Decrypt (`0x67`)

Decrypt data that was encrypted with AES-GCM. Verifies the authentication tag before returning the plaintext. If the tag does not match (indicating tampering or wrong key), the call fails.

### Input

| Field                 | Type      | Description                                               |
| --------------------- | --------- | --------------------------------------------------------- |
| key                   | `bytes`   | AES key (must match the key used for encryption)          |
| nonce                 | `bytes12` | Initialization vector / nonce (12 bytes)                  |
| ciphertext + auth tag | `bytes`   | Encrypted data concatenated with authentication tag       |
| aad                   | `bytes`   | Additional authenticated data (must match encryption AAD) |

### Output

| Field     | Type    | Description    |
| --------- | ------- | -------------- |
| plaintext | `bytes` | Decrypted data |

### Use cases

- On-chain decryption of previously encrypted data
- Decrypting messages from other contracts or users
- Verifying and reading encrypted event data within a contract

### Solidity example

```solidity
function aesDecrypt(
    bytes memory key,
    bytes12 nonce,
    bytes memory ciphertext,
    bytes memory aad
) internal view returns (bytes memory) {
    (bool success, bytes memory result) = address(0x67).staticcall(
        abi.encode(key, nonce, ciphertext, aad)
    );
    require(success, "AES-GCM Decrypt precompile failed");
    return result;
}
```

---

## HKDF (`0x68`)

HMAC-based Key Derivation Function ([RFC 5869](https://datatracker.ietf.org/doc/html/rfc5869)). Derives one or more cryptographic keys from input key material. Commonly used to turn a shared secret (from ECDH) into an encryption key suitable for AES-GCM.

### Input

| Field              | Type    | Description                                           |
| ------------------ | ------- | ----------------------------------------------------- |
| input key material | `bytes` | The source key material (e.g., an ECDH shared secret) |
| salt               | `bytes` | Optional salt value (can be empty)                    |
| info               | `bytes` | Context and application-specific information          |
| output length      | `uint`  | Desired length of the derived key in bytes            |

### Output

| Field       | Type    | Description                   |
| ----------- | ------- | ----------------------------- |
| derived key | `bytes` | The derived cryptographic key |

### Use cases

- Deriving AES encryption keys from ECDH shared secrets
- Generating multiple keys from a single shared secret (e.g., one for encryption, one for MAC)
- Key stretching and domain separation

### Solidity example

```solidity
function deriveKey(
    bytes memory ikm,
    bytes memory salt,
    bytes memory info,
    uint256 outputLength
) internal view returns (bytes memory) {
    (bool success, bytes memory result) = address(0x68).staticcall(
        abi.encode(ikm, salt, info, outputLength)
    );
    require(success, "HKDF precompile failed");
    return result;
}
```

---

## secp256k1 Sign (`0x69`)

Sign a 32-byte message hash using a secp256k1 private key. Produces a 65-byte signature in the (r, s, v) format. This allows contracts to generate signatures on-chain -- something that is normally only possible off-chain with a user's wallet.

{% hint style="warning" %}
This precompile gives the contract the ability to sign arbitrary messages. The private key used must be one the contract has access to (e.g., derived from a seed stored in shielded storage). It does **not** use the caller's private key.
{% endhint %}

### Input

| Field        | Type      | Description                             |
| ------------ | --------- | --------------------------------------- |
| message hash | `bytes32` | The 32-byte hash of the message to sign |
| private key  | `bytes32` | The secp256k1 private key (32 bytes)    |

### Output

| Field     | Type      | Description                       |
| --------- | --------- | --------------------------------- |
| signature | `bytes65` | The signature in (r, s, v) format |

### Use cases

- Generating on-chain attestations or proofs
- Building contracts that can act as signers (e.g., smart-contract wallets)
- Creating signed messages for cross-chain verification

### Solidity example

```solidity
function signMessage(
    bytes32 messageHash,
    bytes32 privateKey
) internal view returns (bytes memory) {
    (bool success, bytes memory result) = address(0x69).staticcall(
        abi.encode(messageHash, privateKey)
    );
    require(success, "secp256k1 Sign precompile failed");
    return result;
}
```

---

## Common pattern: Encrypted events

A frequent use of the precompiles is encrypting event data so that only the intended recipient can read it. The typical flow chains ECDH, HKDF, and AES-GCM Encrypt together:

```solidity
// 1. Derive shared secret with the recipient
(bool ok1, bytes memory secret) = address(0x65).staticcall(
    abi.encode(recipientPubKey, contractPrivKey)
);

// 2. Derive an AES key from the shared secret
(bool ok2, bytes memory aesKey) = address(0x68).staticcall(
    abi.encode(secret, "", "encryption-key", 32)
);

// 3. Encrypt the sensitive data
(bool ok3, bytes memory encrypted) = address(0x66).staticcall(
    abi.encode(aesKey, nonce, plaintext, "")
);

// 4. Emit a regular event with the encrypted bytes
emit EncryptedData(msg.sender, recipient, encrypted);
```

For a complete walkthrough of this pattern, see [Events](../seismic-solidity/events.md).
