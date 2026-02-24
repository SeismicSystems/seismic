---
description: register_viewing_key and compute_key_hash signatures
icon: key
---

# register_viewing_key

| Function | Signature | Returns |
| --- | --- | --- |
| `register_viewing_key` | `register_viewing_key(w3: Web3, encryption: EncryptionState, private_key: PrivateKey, key: Bytes32)` | `HexBytes` |
| `async_register_viewing_key` | `async_register_viewing_key(w3: AsyncWeb3, encryption: EncryptionState, private_key: PrivateKey, key: Bytes32)` | `HexBytes` |
| `compute_key_hash` | `compute_key_hash(aes_key: Bytes32)` | `bytes` |
