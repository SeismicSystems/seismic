---
description: Sign and serialize Seismic transaction using EIP-712
icon: pen-to-square
---

# EIP-712

Sign and serialize a Seismic transaction using EIP-712 typed data.

## Overview

`sign_seismic_tx_eip712()` is the primary function for signing Seismic transactions with EIP-712 structured data hashing. It computes the EIP-712 signing hash, applies an ECDSA signature, and returns the complete RLP-serialized transaction bytes ready for broadcast.

## Signature

```python
def sign_seismic_tx_eip712(
    tx: UnsignedSeismicTx,
    private_key: PrivateKey,
) -> HexBytes
```

## Parameters

| Parameter     | Type                                                       | Required | Description                                                           |
| ------------- | ---------------------------------------------------------- | -------- | --------------------------------------------------------------------- |
| `tx`          | [`UnsignedSeismicTx`](../signature/unsigned-seismic-tx.md) | Yes      | The unsigned Seismic transaction (should have `message_version == 2`) |
| `private_key` | [`PrivateKey`](../bytes32/private-key.md)                  | Yes      | 32-byte secp256k1 private key for signing                             |

## Returns

| Type       | Description                                                             |
| ---------- | ----------------------------------------------------------------------- |
| `HexBytes` | Full signed transaction bytes (`0x4a` prefix + RLP-encoded transaction) |

## Examples

### Basic Usage

```python
from seismic_web3 import (
    sign_seismic_tx_eip712,
    UnsignedSeismicTx,
    PrivateKey,
)
import os

# Build unsigned transaction
unsigned_tx = UnsignedSeismicTx(
    chain_id=5124,
    nonce=42,
    gas_price=20_000_000_000,
    gas=100_000,
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    value=1_000_000_000_000_000_000,
    data=encrypted_calldata,
    seismic=seismic_elements,
)

# Sign with private key
private_key = PrivateKey(os.environ["PRIVATE_KEY"])
signed_tx = sign_seismic_tx_eip712(unsigned_tx, private_key)

# Broadcast
tx_hash = w3.eth.send_raw_transaction(signed_tx)
print(f"Transaction hash: {tx_hash.to_0x_hex()}")
```

### From Debug Write

```python
from seismic_web3 import sign_seismic_tx_eip712, PrivateKey

# Get unsigned transaction from debug write
result = await contract.dwrite.transfer(recipient, 1000)
unsigned_tx = result.shielded_tx

# Sign manually (normally SDK does this)
private_key = PrivateKey(os.environ["PRIVATE_KEY"])
signed_tx = sign_seismic_tx_eip712(unsigned_tx, private_key)

# Broadcast manually
tx_hash = w3.eth.send_raw_transaction(signed_tx)
```

### Verify Message Version

```python
from seismic_web3 import sign_seismic_tx_eip712, PrivateKey

unsigned_tx = UnsignedSeismicTx(...)

# Ensure message_version is 2 for EIP-712
assert unsigned_tx.seismic.message_version == 2, \
    "EIP-712 signing requires message_version == 2"

private_key = PrivateKey(...)
signed_tx = sign_seismic_tx_eip712(unsigned_tx, private_key)
```

### Inspect Signed Transaction

```python
from seismic_web3 import sign_seismic_tx_eip712, PrivateKey

unsigned_tx = UnsignedSeismicTx(...)
private_key = PrivateKey(...)

signed_tx = sign_seismic_tx_eip712(unsigned_tx, private_key)

# Check transaction type (first byte)
tx_type = signed_tx[0]
assert tx_type == 0x4a  # TxSeismic type

# Get full signed transaction hex
print(f"Signed tx: {signed_tx.to_0x_hex()}")
print(f"Length: {len(signed_tx)} bytes")
```

## How It Works

The function performs three steps:

1.  **Compute EIP-712 signing hash**

    ```python
    msg_hash = eip712_signing_hash(tx)
    ```
2.  **Sign with ECDSA**

    ```python
    sk = eth_keys.PrivateKey(bytes(private_key))
    sig_obj = sk.sign_msg_hash(msg_hash)
    ```
3.  **Serialize with signature**

    ```python
    return serialize_signed(tx, signature)
    ```

The RLP serialization is identical to raw signing; only the message hash differs.

## EIP-712 vs Raw Signing

| Aspect             | EIP-712 (`message_version=2`)  | Raw (`message_version=0`) |
| ------------------ | ------------------------------ | ------------------------- |
| **Signing hash**   | Structured EIP-712 hash        | RLP hash of unsigned tx   |
| **Wallet support** | Better UX (structured display) | Generic message signing   |
| **Security**       | Same                           | Same                      |
| **RLP output**     | Identical                      | Identical                 |
| **Verification**   | Node uses EIP-712 path         | Node uses raw path        |

## Transaction Format

The returned bytes have this structure:

```
[0x4a] + RLP([
    chain_id,
    nonce,
    gas_price,
    gas,
    to,
    value,
    data,
    seismic_elements,
    v, r, s  # Signature
])
```

## Notes

* The transaction's `message_version` should be `2` for EIP-712 signing
* The RLP serialization is identical to raw signing mode
* The Seismic node checks `message_version` to determine verification path
* The SDK defaults to EIP-712 for better wallet UX
* Returned bytes are ready for `eth_sendRawTransaction`

## Warnings

* **Private key security** - Never log or expose private keys
* **Message version** - Must match signing method (`2` for EIP-712)
* **Transaction validity** - Ensure `expires_at_block` hasn't passed
* **Nonce management** - Incorrect nonce causes transaction rejection

## See Also

* [eip712\_signing\_hash](eip712-signing-hash.md) - Compute the signing hash
* [domain\_separator](domain-separator.md) - EIP-712 domain separator
* [struct\_hash](struct-hash.md) - EIP-712 struct hash
* [build\_seismic\_typed\_data](build-seismic-typed-data.md) - Build typed data dict
* [UnsignedSeismicTx](../signature/unsigned-seismic-tx.md) - Input transaction type
* [PrivateKey](../bytes32/private-key.md) - Signing key type
