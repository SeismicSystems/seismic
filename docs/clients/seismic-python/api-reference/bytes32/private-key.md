---
description: 32-byte secp256k1 private key
icon: key
---

# PrivateKey

32-byte secp256k1 private key used for signing transactions and ECDH key derivation.

## Overview

`PrivateKey` is a specialized [`Bytes32`](./) type representing a secp256k1 private key. It's used for transaction signing and deriving shared secrets via ECDH.

## Definition

```python
class PrivateKey(Bytes32):
    """32-byte secp256k1 private key."""
```

## Inheritance

* Inherits from [`Bytes32`](./)
* Inherits all validation and construction methods from `Bytes32`
* Always exactly 32 bytes

## Construction

`PrivateKey` can be constructed from:

* Hex strings (with or without `"0x"` prefix)
* Raw `bytes` objects
* Integer values

### Parameters

| Parameter | Type                  | Required | Description                                   |
| --------- | --------------------- | -------- | --------------------------------------------- |
| `val`     | `bytes \| str \| int` | Yes      | The value to convert to a 32-byte private key |

### Raises

* `ValueError` - If the resulting length is not exactly 32 bytes

## Examples

### From Hex String

```python
from seismic_web3 import PrivateKey

# With 0x prefix
private_key = PrivateKey("0x1234567890abcdef" + "0" * 48)

# From environment variable
import os
private_key = PrivateKey(os.environ["PRIVATE_KEY"])
```

### Generate Random Private Key

```python
import os
from seismic_web3 import PrivateKey

# Generate cryptographically secure random key
private_key = PrivateKey(os.urandom(32))
```

### Use with Client

```python
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET

private_key = PrivateKey(os.environ["PRIVATE_KEY"])

w3 = create_wallet_client(
    rpc_url="https://gcp-1.seismictest.net/rpc",
    chain=SEISMIC_TESTNET,
    account=private_key,
)
```

### Use with EIP-712 Signing

```python
from seismic_web3 import sign_seismic_tx_eip712, PrivateKey

# Build unsigned transaction
unsigned_tx = ...  # UnsignedSeismicTx

# Sign with private key
private_key = PrivateKey(os.environ["PRIVATE_KEY"])
signed_tx = sign_seismic_tx_eip712(unsigned_tx, private_key)
```

## Security Considerations

* **Never log or print private keys** - They provide full control over the account
* **Store securely** - Use environment variables or secure key management systems
* **Use OS-provided randomness** - `os.urandom()` for key generation
* **Avoid hardcoding** - Never commit private keys to source control

## Methods

Inherits all methods from [`Bytes32`](./):

* `to_0x_hex()` - Convert to hex string (use carefully!)

## Properties

* **Immutable** - Cannot be modified after construction
* **Hashable** - Can be used as dictionary keys
* **32 bytes** - Always exactly 32 bytes
* **Compatible with bytes** - Passes `isinstance(x, bytes)` checks

## Notes

* Used exclusively for secp256k1 curve operations
* Compatible with `eth_keys.PrivateKey` after conversion: `eth_keys.PrivateKey(bytes(private_key))`
* No validation is performed on the mathematical validity of the key (i.e., no check that `0 < key < n`)

## See Also

* [Bytes32](./) - Parent type
* [CompressedPublicKey](compressed-public-key.md) - Corresponding public key type
* [sign\_seismic\_tx\_eip712](../sign-seismic-tx-eip712/) - Sign transactions with private key
* [create\_wallet\_client](../../client/create-wallet-client.md) - Create client with private key
* [EncryptionState](../../client/encryption-state.md) - Derives keys from private key
