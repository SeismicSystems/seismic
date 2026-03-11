<!-- TODO: Once secp256k1_pubkey precompile lands (seismic-revm#207, seismic-solidity#225),
     update the PrivateToken example to generate its keypair on-chain with rng256() +
     secp256k1_pubkey() instead of requiring setContractKey(). -->
---
icon: blog
metaLinks:
  alternates:
    - https://app.gitbook.com/s/hkB2uNxma1rxIgBfHgAT/core/events
---

# Events

## The Limitation

Shielded types **cannot** be emitted directly in events. The following will not compile:

```solidity
event ConfidentialEvent(suint256 confidentialData); // Compilation error
```

This restriction exists because events are stored in transaction logs, which are publicly accessible on-chain. Emitting a shielded value in an event would defeat the purpose of shielding it in the first place -- the value would be visible to anyone inspecting the logs.

This applies to all shielded types: `suint`, `sint`, `sbool`, `saddress`, and `sbytes`.

## The Workaround: Encrypted Events via Precompiles

Although native encrypted events are not yet supported, you can achieve private event data today using the AES-GCM and ECDH precompiles. The approach is to encrypt the sensitive data before emitting it in a regular (unshielded) event, so that only the intended recipient can decrypt it.

Here is the general flow:

1. **Generate a shared secret** between the sender and the intended recipient using the [ECDH precompile](../reference/precompiles.md#ecdh-0x65) at address `0x65`.
2. **Derive an encryption key** from the shared secret using the [HKDF precompile](../reference/precompiles.md#hkdf-0x68) at address `0x68`.
3. **Encrypt the event data** using the [AES-GCM Encrypt precompile](../reference/precompiles.md#aes-gcm-encrypt-0x66) at address `0x66`.
4. **Emit a regular event** containing the encrypted bytes. Since the event parameter is `bytes` (not a shielded type), this compiles and works normally.
5. **Recipient decrypts** the data using the [AES-GCM Decrypt precompile](../reference/precompiles.md#aes-gcm-decrypt-0x67) at address `0x67`, either off-chain or on-chain if needed.

### Precompile Reference

| Precompile      | Address | Purpose                           |
| --------------- | ------- | --------------------------------- |
| ECDH            | [`0x65`](../reference/precompiles.md#ecdh-0x65)  | Shared secret generation          |
| AES-GCM Encrypt | [`0x66`](../reference/precompiles.md#aes-gcm-encrypt-0x66)  | Encrypt data with AES-GCM         |
| AES-GCM Decrypt | [`0x67`](../reference/precompiles.md#aes-gcm-decrypt-0x67)  | Decrypt data with AES-GCM         |
| HKDF            | [`0x68`](../reference/precompiles.md#hkdf-0x68)  | Key derivation from shared secret |

## Code Example

Below is a minimal private token showing how to emit encrypted transfer events. The contract holds a keypair; users register their public keys so the contract can encrypt event data that only the recipient can read.

For a full implementation, see the [SRC20: Private Token](../tutorials/src20/encrypted-events.md) tutorial.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract PrivateToken {
    address public owner;
    mapping(address => suint256) private balances;
    mapping(address => bytes) public publicKeys;

    sbytes32 private contractPrivateKey;
    bytes public contractPublicKey;

    event Transfer(address indexed from, address indexed to, bytes encryptedAmount);

    constructor() {
        owner = msg.sender;
    }

    // Owner sets the contract keypair via a Seismic transaction (type 0x4A)
    // so the private key is encrypted in calldata and never exposed.
    function setContractKey(sbytes32 _privateKey, bytes memory _publicKey) external {
        require(msg.sender == owner, "Only owner");
        require(bytes32(contractPrivateKey) == bytes32(0), "Already set");
        contractPrivateKey = _privateKey;
        contractPublicKey = _publicKey;
    }

    function registerPublicKey(bytes memory pubKey) external {
        publicKeys[msg.sender] = pubKey;
    }

    function transfer(address to, suint256 amount) public {
        require(bytes32(contractPrivateKey) != bytes32(0), "Contract key not set");
        balances[msg.sender] -= amount;
        balances[to] += amount;

        bytes memory recipientPubKey = publicKeys[to];
        if (recipientPubKey.length > 0) {
            // 1. Shared secret via ECDH
            bytes32 sharedSecret = ecdh(contractPrivateKey, recipientPubKey);

            // 2. Derive encryption key via HKDF
            sbytes32 encKey = sbytes32(hkdf(abi.encodePacked(sharedSecret)));

            // 3. Encrypt the amount via AES-GCM
            uint96 nonce = uint96(bytes12(keccak256(abi.encodePacked(msg.sender, to, block.number))));
            bytes memory encrypted = aes_gcm_encrypt(encKey, nonce, abi.encode(uint256(amount)));

            // 4. Emit with encrypted bytes
            emit Transfer(msg.sender, to, encrypted);
        }
    }
}
```

The built-in helpers `ecdh()`, `hkdf()`, and `aes_gcm_encrypt()` are compiler-provided globals — no imports needed. See the [Precompiles reference](../reference/precompiles.md) for details on each.

### Decryption (Off-Chain)

The recipient reconstructs the shared secret off-chain using their own private key and the contract's public key (ECDH is symmetric). They then derive the same encryption key via HKDF and decrypt the event data using AES-GCM Decrypt.

The client libraries provide built-in helpers for this:

<!-- TODO: add Alloy (Rust) link once docs/gitbook/clients/alloy/src20/event-decryption.md is finalized -->
- [**Python**](../clients/python/src20/event-watching/README.md) — `watch_src20_events_with_key` and `SRC20EventWatcher`
- [**TypeScript (viem)**](../clients/typescript/viem/shielded-public-client.md) — `watchSRC20EventsWithKey()` action

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

Privacy in events is achievable today -- it just requires explicit encryption via the aforementioned precompiles. Encrypt sensitive data before emitting it, and ensure only the intended recipient has the keys to decrypt. When native encrypted events ship, migrating will be straightforward: replace the manual encryption logic with direct shielded type emission.
