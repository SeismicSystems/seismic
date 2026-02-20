---
description: Complete API reference for the Seismic Python SDK
icon: book
---

# API Reference

Complete reference documentation for all publicly exported functions, classes, and types in the Seismic Python SDK (`seismic_web3`).

## Quick Navigation

### [Types](types/)
Primitive byte types used throughout the SDK.

| Type | Description |
|------|-------------|
| [Bytes32](types/bytes32.md) | Exactly 32 bytes — used for hashes, AES keys, and similar values |
| [PrivateKey](types/private-key.md) | 32-byte secp256k1 private key |
| [CompressedPublicKey](types/compressed-public-key.md) | 33-byte compressed secp256k1 public key |
| [EncryptionNonce](types/encryption-nonce.md) | 12-byte AES-GCM encryption nonce |
| [hex\_to\_bytes](types/hex-to-bytes.md) | Convert a hex string to raw bytes, stripping optional `0x` |

### [Transaction Types](transaction-types/)
Dataclasses for building and signing Seismic transactions.

| Type | Description |
|------|-------------|
| [Signature](transaction-types/signature.md) | ECDSA signature components (v, r, s) |
| [SeismicElements](transaction-types/seismic-elements.md) | Seismic-specific transaction fields |
| [SeismicSecurityParams](transaction-types/seismic-security-params.md) | Optional security parameters for shielded transactions |
| [UnsignedSeismicTx](transaction-types/unsigned-seismic-tx.md) | Complete unsigned Seismic transaction |
| [TxSeismicMetadata](transaction-types/tx-seismic-metadata.md) | Transaction metadata used for AAD context |
| [LegacyFields](transaction-types/legacy-fields.md) | Standard EVM transaction fields |
| [PlaintextTx](transaction-types/plaintext-tx.md) | Unencrypted transaction view |
| [DebugWriteResult](transaction-types/debug-write-result.md) | Result from debug shielded write |

### [EIP-712](eip712/)
Functions for EIP-712 typed data signing of Seismic transactions.

| Function | Description |
|----------|-------------|
| [sign_seismic_tx_eip712](eip712/sign-seismic-tx-eip712.md) | Sign and serialize a transaction using EIP-712 |
| [eip712_signing_hash](eip712/eip712-signing-hash.md) | Compute the EIP-712 signing hash |
| [domain_separator](eip712/domain-separator.md) | Compute the EIP-712 domain separator |
| [struct_hash](eip712/struct-hash.md) | Compute the EIP-712 struct hash |
| [build_seismic_typed_data](eip712/build-seismic-typed-data.md) | Build EIP-712 typed data dict |

## See Also

- [Client Documentation](../client/) - Client creation and configuration
- [Contract Documentation](../contract/) - Contract interaction patterns
- [Chains Configuration](../chains/) - Chain configuration and constants
- [Precompiles](../precompiles/) - Privacy-preserving cryptographic functions
- [SRC20](../src20/) - SRC20 token standard support
- [ABIs](../abis/) - Built-in contract ABIs and helpers
