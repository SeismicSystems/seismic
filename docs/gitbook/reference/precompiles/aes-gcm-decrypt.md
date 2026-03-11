---
icon: lock-open
---

# AES-GCM Decrypt (`0x67`)

Decrypt data that was encrypted with [AES-256-GCM](https://en.wikipedia.org/wiki/Galois/Counter_Mode). Verifies the authentication tag before returning the plaintext. If the tag does not match (indicating tampering or wrong key), the call fails.

## Input

Raw bytes in the following layout:

| Offset    | Field                 | Type      | Description                                         |
| --------- | --------------------- | --------- | --------------------------------------------------- |
| `[0:32]`  | key                   | `bytes32` | AES-256 key (must match the key used for encryption) |
| `[32:44]` | nonce                 | `bytes12` | Initialization vector (12 bytes)                    |
| `[44:]`   | ciphertext + auth tag | `bytes`   | Encrypted data concatenated with authentication tag |

## Output

| Bytes     | Type    | Description    |
| --------- | ------- | -------------- |
| plaintext | `bytes` | Decrypted data |

## Use cases

* On-chain decryption of previously encrypted data
* Decrypting messages from other contracts or users
* Verifying and reading encrypted event data within a contract

## Built-in helper

Seismic Solidity provides `aes_gcm_decrypt(sbytes32 key, uint96 nonce, bytes memory ciphertext)` which returns `bytes memory`. The key is shielded.

```solidity
bytes memory plaintext = aes_gcm_decrypt(aesKey, nonce, ciphertext);
```

## Manual usage

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
