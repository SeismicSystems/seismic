---
icon: blog
---

# Events

## The Limitation

Shielded types **cannot** be emitted directly in events. The following will not compile:

```solidity
event ConfidentialEvent(suint256 confidentialData); // Compilation error
```

This restriction exists because events are stored in transaction logs, which are publicly accessible on-chain. Emitting a shielded value in an event would defeat the purpose of shielding it in the first place -- the value would be visible to anyone inspecting the logs.

This applies to all shielded types: `suint`, `sint`, `sbool`, and `saddress`.

## The Workaround: Encrypted Events via Precompiles

Although native encrypted events are not yet supported, you can achieve private event data today using the AES-GCM and ECDH precompiles. The approach is to encrypt the sensitive data before emitting it in a regular (unshielded) event, so that only the intended recipient can decrypt it.

Here is the general flow:

1. **Generate a shared secret** between the sender and the intended recipient using the ECDH precompile at address `0x65`.
2. **Derive an encryption key** from the shared secret using the HKDF precompile at address `0x68`.
3. **Encrypt the event data** using the AES-GCM Encrypt precompile at address `0x66`.
4. **Emit a regular event** containing the encrypted bytes. Since the event parameter is `bytes` (not a shielded type), this compiles and works normally.
5. **Recipient decrypts** the data using the AES-GCM Decrypt precompile at address `0x67`, either off-chain or on-chain if needed.

### Precompile Reference

| Precompile      | Address | Purpose                           |
| --------------- | ------- | --------------------------------- |
| ECDH            | `0x65`  | Shared secret generation          |
| AES-GCM Encrypt | `0x66`  | Encrypt data with AES-GCM         |
| AES-GCM Decrypt | `0x67`  | Decrypt data with AES-GCM         |
| HKDF            | `0x68`  | Key derivation from shared secret |

## Code Example

Below is a complete pattern showing how to emit encrypted event data for a private token transfer. The recipient can later decrypt the amount off-chain using their private key.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract PrivateToken {
    mapping(address => suint256) private balances;

    // Event with encrypted data -- uses regular `bytes`, not shielded types
    event EncryptedTransfer(
        address indexed from,
        address indexed to,
        bytes encryptedAmount
    );

    function transfer(address to, suint256 amount) public {
        balances[msg.sender] -= amount;
        balances[to] += amount;

        // Step 1: Generate shared secret via ECDH (0x65)
        // The sender's private key and recipient's public key produce
        // a shared secret that only both parties can derive.
        bytes memory sharedSecret;
        // ... call precompile 0x65 with sender privkey + recipient pubkey

        // Step 2: Derive encryption key via HKDF (0x68)
        bytes memory encryptionKey;
        // ... call precompile 0x68 with sharedSecret

        // Step 3: Encrypt the amount via AES-GCM Encrypt (0x66)
        bytes memory encrypted;
        // ... call precompile 0x66 with encryptionKey and plaintext amount

        // Step 4: Emit a regular event with the encrypted bytes
        emit EncryptedTransfer(msg.sender, to, encrypted);
    }
}
```

{% hint style="info" %}
The recipient reconstructs the shared secret off-chain using their own private key and the sender's public key (ECDH is symmetric). They then derive the same encryption key via HKDF and decrypt the event data using AES-GCM Decrypt (`0x67`).
{% endhint %}

### Decryption (Off-Chain)

The recipient can decrypt the event data by:

1. Listening for `EncryptedTransfer` events addressed to them.
2. Deriving the shared secret using ECDH with their private key and the sender's public key.
3. Deriving the encryption key using HKDF.
4. Decrypting the `encryptedAmount` field using AES-GCM Decrypt.

This can also be done on-chain if another contract needs access to the decrypted value.

## What Not to Do

Do not attempt to work around the restriction by casting a shielded value to its unshielded counterpart and then emitting it:

```solidity
// BAD: This exposes the confidential value to everyone
event Transfer(address from, address to, uint256 amount);

function transfer(address to, suint256 amount) public {
    // ...
    emit Transfer(msg.sender, to, uint256(amount)); // Leaks the amount!
}
```

Casting from a shielded type to an unshielded type makes the value visible in the execution trace. Emitting it in an event then permanently records it in publicly accessible logs.

## Future Improvements

Native encrypted events are planned for a future version of Seismic. This will allow shielded types to be emitted directly in events without requiring manual encryption via precompiles. The compiler and runtime will handle encryption transparently, making private events as simple to use as regular events.

## Key Takeaway

Privacy in events is achievable today -- it just requires explicit encryption via the AES-GCM and ECDH precompiles. Encrypt sensitive data before emitting it, and ensure only the intended recipient has the keys to decrypt. When native encrypted events ship, migrating will be straightforward: replace the manual encryption logic with direct shielded type emission.
