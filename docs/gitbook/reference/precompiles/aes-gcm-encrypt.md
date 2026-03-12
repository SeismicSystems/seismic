---
icon: lock
---

# AES-GCM Encrypt (`0x66`)

Encrypt data using [AES-256-GCM](https://en.wikipedia.org/wiki/Galois/Counter_Mode) (Galois/Counter Mode) authenticated encryption. Produces ciphertext with an appended authentication tag that guarantees integrity and authenticity.

{% hint style="info" %}
Only AES-256 is supported. The key must be exactly 32 bytes.
{% endhint %}

## Input

Raw bytes in the following layout:

| Offset    | Field     | Type      | Description                      |
| --------- | --------- | --------- | -------------------------------- |
| `[0:32]`  | key       | 32 bytes        | AES-256 key              |
| `[32:44]` | nonce     | 12 bytes        | Initialization vector    |
| `[44:]`   | plaintext | bytes | Data to encrypt          |

## Output

| Bytes                 | Type            | Description                                         |
| --------------------- | --------------- | --------------------------------------------------- |
| ciphertext + auth tag | bytes | Encrypted data concatenated with authentication tag |

## Use cases

* Encrypting sensitive event parameters before emission
* Building encrypted storage helpers
* Encrypting messages or data for specific recipients

## Examples

### Built-in helper

```solidity
function aes_gcm_encrypt(sbytes32 key, uint96 nonce, bytes memory plaintext) view returns (bytes memory);
```

```solidity
bytes memory ciphertext = aes_gcm_encrypt(aesKey, nonce, plaintext);
```

### Manual usage

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

See [CryptoUtils.sol](https://github.com/SeismicSystems/seismic/blob/main/contracts/src/seismic-std-lib/utils/precompiles/CryptoUtils.sol) for the production implementation.
