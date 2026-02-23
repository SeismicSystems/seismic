---
description: Core Python SDK types and EIP-712 helpers
icon: code
---

# API Reference

This section documents the low-level types and helpers exported by `seismic_web3`.

## Sections

- [Types](types/bytes32.md): `Bytes32`, `PrivateKey`, `CompressedPublicKey`, `EncryptionNonce`, `hex_to_bytes`
- [Transaction Types](transaction-types/signature.md): dataclasses used for `TxSeismic`
- [EIP-712](eip712/sign-seismic-tx-eip712.md): typed-data hashing/signing helpers

## Source Of Truth

All signatures and behavior in this section are based on:

- `clients/py/src/seismic_web3/_types.py`
- `clients/py/src/seismic_web3/transaction_types.py`
- `clients/py/src/seismic_web3/transaction/eip712.py`
