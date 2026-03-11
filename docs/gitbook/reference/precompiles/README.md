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

| Address | Name            | Purpose                                                    |
| ------- | --------------- | ---------------------------------------------------------- |
| `0x64`  | [RNG](rng.md)             | Securely generate random bytes inside the TEE              |
| `0x65`  | [ECDH](ecdh.md)            | Derive an AES key from a private key and a public key      |
| `0x66`  | [AES-GCM Encrypt](aes-gcm-encrypt.md) | Encrypt data with AES-256-GCM authenticated encryption     |
| `0x67`  | [AES-GCM Decrypt](aes-gcm-decrypt.md) | Decrypt data with AES-256-GCM authenticated encryption     |
| `0x68`  | [HKDF](hkdf.md)            | Derive a 32-byte key from input key material               |
| `0x69`  | [secp256k1 Sign](secp256k1-sign.md)  | Sign a message hash with a secp256k1 private key           |

## Common pattern: Encrypted events

A frequent use of the precompiles is encrypting event data so that only the intended recipient can read it. The typical flow chains ECDH and AES-GCM Encrypt together:

```solidity
// 1. Derive an AES key via ECDH
sbytes32 aesKey = sbytes32(ecdh(contractPrivKey, recipientCompressedPubKey));

// 2. Generate a random nonce
uint96 nonce = uint96(sync_rng128());

// 3. Encrypt the sensitive data
bytes memory encrypted = aes_gcm_encrypt(aesKey, nonce, plaintext);

// 4. Emit a regular event with the encrypted bytes
emit EncryptedData(msg.sender, recipient, nonce, encrypted);
```

For a complete walkthrough of this pattern, see [Events](../../seismic-solidity/events.md).
