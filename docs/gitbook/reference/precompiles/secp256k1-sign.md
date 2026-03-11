---
icon: signature
---

# secp256k1 Sign (`0x69`)

Sign a 32-byte message hash using a [secp256k1](https://en.wikipedia.org/wiki/Elliptic_Curve_Digital_Signature_Algorithm) private key. Produces a 65-byte signature in the (r, s, v) format. This allows contracts to generate signatures on-chain -- something that is normally only possible off-chain with a user's wallet.

{% hint style="warning" %}
This precompile gives the contract the ability to sign arbitrary messages. The private key used must be one the contract has access to (e.g., derived from a seed stored in shielded storage). It does **not** use the caller's private key.
{% endhint %}

## Input

Raw bytes in the following layout (64 bytes total):

| Offset    | Field        | Type      | Description                             |
| --------- | ------------ | --------- | --------------------------------------- |
| `[0:32]`  | private key  | 32 bytes | secp256k1 private key    |
| `[32:64]` | message hash | 32 bytes | Hash of the message to sign |

## Output

| Bytes     | Type           | Description                                  |
| --------- | -------------- | -------------------------------------------- |
| signature | 65 bytes | Signature in (r, s, v) format |

## Use cases

* Generating on-chain attestations or proofs
* Building contracts that can act as signers (e.g., smart-contract wallets)
* Creating signed messages for cross-chain verification

## Examples

### Built-in helper

```solidity
function secp256k1_sign(sbytes32 secretKey, bytes32 messageHash) view returns (bytes memory);
```

```solidity
bytes memory signature = secp256k1_sign(mySecretKey, msgHash);
```

### Manual usage

```solidity
function signMessage(
    bytes32 privateKey,
    bytes32 messageHash
) internal view returns (bytes memory) {
    (bool success, bytes memory result) = address(0x69).staticcall(
        abi.encodePacked(privateKey, messageHash)
    );
    require(success, "secp256k1 Sign precompile failed");
    return result;
}
```
