---
icon: microchip
---

# Precompiled Contracts

Seismic adds six precompiled contracts to the EVM, giving smart contracts access to cryptographic primitives that would be prohibitively expensive to implement in Solidity. All standard Ethereum precompiles (e.g. `ecrecover`, `sha256`) remain available at their usual addresses.

Precompiles are called like regular contracts using `staticcall` or `call` to their fixed addresses. They execute native code rather than EVM bytecode, making them significantly cheaper and faster than equivalent Solidity implementations.

{% hint style="warning" %}
All precompiles expect **raw concatenated bytes** as input, not ABI-encoded data. Use `abi.encodePacked` (not `abi.encode`) when constructing input in Solidity.
{% endhint %}

These are the precompiles Seismic adds:

| Name            | Address                    | Purpose                                                |
| --------------- | -------------------------- | ------------------------------------------------------ |
| RNG             | [`0x64`](rng.md)           | Securely generate random bytes inside the TEE          |
| ECDH            | [`0x65`](ecdh.md)          | Derive an AES key from a private key and a public key  |
| AES-GCM Encrypt | [`0x66`](aes-gcm-encrypt.md) | Encrypt data with AES-256-GCM authenticated encryption |
| AES-GCM Decrypt | [`0x67`](aes-gcm-decrypt.md) | Decrypt data with AES-256-GCM authenticated encryption |
| HKDF            | [`0x68`](hkdf.md)          | Derive a 32-byte key from input key material           |
| secp256k1 Sign  | [`0x69`](secp256k1-sign.md) | Sign a message hash with a secp256k1 private key       |

## Common pattern: Encrypted events

A frequent use of the precompiles is encrypting event data so that only the intended recipient can read it. The typical flow chains ECDH and AES-GCM Encrypt together:

```solidity
// 1. Derive an AES key via ECDH
sbytes32 aesKey = sbytes32(ecdh(contractPrivKey, recipientCompressedPubKey));

// 2. Generate a random nonce
uint96 nonce = uint96(unsafe_rng_u96());

// 3. Encrypt the sensitive data
bytes memory encrypted = aes_gcm_encrypt(aesKey, nonce, plaintext);

// 4. Emit a regular event with the encrypted bytes
emit EncryptedData(msg.sender, recipient, nonce, encrypted);
```

For a complete walkthrough of this pattern, see [Events](../../seismic-solidity/events.md).
