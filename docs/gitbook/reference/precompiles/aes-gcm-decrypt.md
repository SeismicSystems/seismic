---
icon: lock-open
---

# AES-GCM Decrypt (`0x67`)

Decrypt data that was encrypted with [AES-256-GCM](https://en.wikipedia.org/wiki/Galois/Counter_Mode). Verifies the authentication tag before returning the plaintext. If the tag does not match (indicating tampering or wrong key), the call fails.

## Input

Raw bytes in the following layout:

| Offset    | Field                 | Type      | Description                                         |
| --------- | --------------------- | --------- | --------------------------------------------------- |
| `[0:32]`  | key                   | 32 bytes        | AES-256 key (must match encryption key) |
| `[32:44]` | nonce                 | 12 bytes        | Initialization vector                   |
| `[44:]`   | ciphertext + auth tag | bytes | Encrypted data with authentication tag  |

## Output

| Bytes     | Type            | Description    |
| --------- | --------------- | -------------- |
| plaintext | bytes | Decrypted data |

## Use cases

* On-chain decryption of previously encrypted data
* Decrypting messages from other contracts or users
* Verifying and reading encrypted event data within a contract

## Examples

### Built-in helper

```solidity
function aes_gcm_decrypt(sbytes32 key, uint96 nonce, bytes memory ciphertext) view returns (bytes memory);
```

```solidity
bytes memory plaintext = aes_gcm_decrypt(aesKey, nonce, ciphertext);
```

### Manual usage

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
