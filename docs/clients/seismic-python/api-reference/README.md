---
description: Complete API reference for the Seismic Python SDK
icon: book
---

# API Reference

Complete reference documentation for all publicly exported functions, classes, and types in the Seismic Python SDK (`seismic_web3`).

## Quick Navigation

### [Types](../../../gitbook/client-libraries/seismic-python/api-reference/types/)

Primitive byte types used throughout the SDK.

| Type                                                    | Description                                                      |
| ------------------------------------------------------- | ---------------------------------------------------------------- |
| [Bytes32](bytes32/)                                     | Exactly 32 bytes — used for hashes, AES keys, and similar values |
| [PrivateKey](bytes32/private-key.md)                    | 32-byte secp256k1 private key                                    |
| [CompressedPublicKey](bytes32/compressed-public-key.md) | 33-byte compressed secp256k1 public key                          |
| [EncryptionNonce](bytes32/encryption-nonce.md)          | 12-byte AES-GCM encryption nonce                                 |

### [Transaction Types](../../../gitbook/client-libraries/seismic-python/api-reference/transaction-types/)

Dataclasses for building and signing Seismic transactions.

| Type                                                          | Description                                            |
| ------------------------------------------------------------- | ------------------------------------------------------ |
| [Signature](signature/)                                       | ECDSA signature components (v, r, s)                   |
| [SeismicElements](signature/seismic-elements.md)              | Seismic-specific transaction fields                    |
| [SeismicSecurityParams](signature/seismic-security-params.md) | Optional security parameters for shielded transactions |
| [UnsignedSeismicTx](signature/unsigned-seismic-tx.md)         | Complete unsigned Seismic transaction                  |
| [TxSeismicMetadata](signature/tx-seismic-metadata.md)         | Transaction metadata used for AAD context              |
| [LegacyFields](signature/legacy-fields.md)                    | Standard EVM transaction fields                        |
| [PlaintextTx](signature/plaintext-tx.md)                      | Unencrypted transaction view                           |
| [DebugWriteResult](signature/debug-write-result.md)           | Result from debug shielded write                       |

### [EIP-712](../../../gitbook/client-libraries/seismic-python/api-reference/eip712/)

Functions for EIP-712 typed data signing of Seismic transactions.

| Function                                                                          | Description                                    |
| --------------------------------------------------------------------------------- | ---------------------------------------------- |
| [sign\_seismic\_tx\_eip712](sign-seismic-tx-eip712/)                              | Sign and serialize a transaction using EIP-712 |
| [eip712\_signing\_hash](sign-seismic-tx-eip712/eip712-signing-hash.md)            | Compute the EIP-712 signing hash               |
| [domain\_separator](sign-seismic-tx-eip712/domain-separator.md)                   | Compute the EIP-712 domain separator           |
| [struct\_hash](sign-seismic-tx-eip712/struct-hash.md)                             | Compute the EIP-712 struct hash                |
| [build\_seismic\_typed\_data](sign-seismic-tx-eip712/build-seismic-typed-data.md) | Build EIP-712 typed data dict                  |

## See Also

* [Client Documentation](../client/) - Client creation and configuration
* [Contract Documentation](../contract/) - Contract interaction patterns
* [Chains Configuration](../chains/) - Chain configuration and constants
* [Precompiles](../precompiles/) - Privacy-preserving cryptographic functions
* [SRC20](../src20/) - SRC20 token standard support
* [ABIs](../abis/) - Built-in contract ABIs and helpers
