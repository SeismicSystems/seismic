---
icon: key
---

# ECDH (`0x65`)

[Elliptic Curve Diffie-Hellman](https://en.wikipedia.org/wiki/Elliptic-curve_Diffie%E2%80%93Hellman) (ECDH) key agreement on the secp256k1 curve, followed by HKDF key derivation. Given a private key and a public key, produces a derived AES-256 key that both parties can independently compute. This is the foundation for encrypted communication between a contract and a user.

{% hint style="info" %}
The output is a **derived AES key**, not the raw ECDH shared secret. The precompile internally runs ECDH followed by HKDF-SHA256 to produce a key suitable for AES-256-GCM encryption.
{% endhint %}

## Input

Raw bytes in the following layout (65 bytes total):

| Offset    | Field       | Type      | Description                                |
| --------- | ----------- | --------- | ------------------------------------------ |
| `[0:32]`  | private key | `bytes32` | secp256k1 private key (32 bytes)           |
| `[32:65]` | public key  | `bytes33` | Compressed secp256k1 public key (33 bytes) |

## Output

| Bytes       | Type           | Description                                  |
| ----------- | -------------- | -------------------------------------------- |
| derived key | `bytes memory` | 32 bytes — an AES key derived via ECDH + HKDF |

## Use cases

* Key agreement between a contract and a user for encrypting event data
* Establishing shared encryption keys for private communication channels
* Enabling per-recipient encryption of on-chain data

## Built-in helper

Seismic Solidity provides `ecdh(sbytes32 secretKey, bytes memory publicKey)` which returns `bytes32`. The secret key is shielded and the result is automatically extracted for you.

```solidity
bytes32 aesKey = ecdh(mySecretKey, recipientCompressedPubKey);
```

## Manual usage

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
