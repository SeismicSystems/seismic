---
icon: arrows-split-up-and-left
---

# HKDF (`0x68`)

[HMAC-based Key Derivation Function](https://en.wikipedia.org/wiki/HKDF) ([RFC 5869](https://datatracker.ietf.org/doc/html/rfc5869)). Derives a 32-byte cryptographic key from input key material. Commonly used to turn a shared secret (from ECDH) into an encryption key suitable for AES-GCM.

{% hint style="info" %}
The Seismic HKDF precompile uses hardcoded parameters: salt is `None`, info is a fixed internal string, and the output is always 32 bytes. The only user-supplied input is the raw key material.
{% endhint %}

## Input

| Field              | Type    | Description                                           |
| ------------------ | ------- | ----------------------------------------------------- |
| input key material | bytes | Source key material (e.g., raw ECDH shared point) |

## Output

| Bytes       | Type           | Description                       |
| ----------- | -------------- | --------------------------------- |
| derived key | 32 bytes | Derived key material |

## Use cases

* Deriving AES encryption keys from shared secrets
* Key derivation with domain separation

## Examples

### Built-in helper

```solidity
function hkdf(bytes memory input) view returns (bytes32);
```

```solidity
bytes32 derivedKey = hkdf(inputKeyMaterial);
```

### Manual usage

```solidity
function deriveKey(bytes memory ikm) internal view returns (bytes32) {
    (bool success, bytes memory result) = address(0x68).staticcall(ikm);
    require(success, "HKDF precompile failed");
    require(result.length == 32, "Invalid HKDF output length");
    return bytes32(result);
}
```
