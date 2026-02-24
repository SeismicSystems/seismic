---
description: Directory helper function signatures
icon: key
---

# Directory

| Function | Signature | Returns |
| --- | --- | --- |
| `check_has_key` | `check_has_key(w3: Web3, address: ChecksumAddress)` | `bool` |
| `async_check_has_key` | `async_check_has_key(w3: AsyncWeb3, address: ChecksumAddress)` | `bool` |
| `get_key_hash` | `get_key_hash(w3: Web3, address: ChecksumAddress)` | `bytes` |
| `async_get_key_hash` | `async_get_key_hash(w3: AsyncWeb3, address: ChecksumAddress)` | `bytes` |
| `get_viewing_key` | `get_viewing_key(w3: Web3, encryption: EncryptionState, private_key: PrivateKey)` | `Bytes32` |
| `async_get_viewing_key` | `async_get_viewing_key(w3: AsyncWeb3, encryption: EncryptionState, private_key: PrivateKey)` | `Bytes32` |
| `register_viewing_key` | `register_viewing_key(w3: Web3, encryption: EncryptionState, private_key: PrivateKey, key: Bytes32)` | `HexBytes` |
| `async_register_viewing_key` | `async_register_viewing_key(w3: AsyncWeb3, encryption: EncryptionState, private_key: PrivateKey, key: Bytes32)` | `HexBytes` |
| `compute_key_hash` | `compute_key_hash(aes_key: Bytes32)` | `bytes` |
