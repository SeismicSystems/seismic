---
description: Complete unsigned Seismic transaction
icon: file-contract
---

# UnsignedSeismicTx

All fields of a TxSeismic (type `0x4a`) transaction before signing.

## Overview

`UnsignedSeismicTx` represents a complete Seismic transaction with encrypted calldata, ready to be signed. It combines standard EVM fields with Seismic-specific encryption metadata.

## Definition

```python
@dataclass(frozen=True)
class UnsignedSeismicTx:
    """All fields of a TxSeismic before signing.

    The data field contains encrypted calldata (ciphertext).
    Use the serialization functions in transaction.serialize to
    RLP-encode this for hashing and signing.
    """
    chain_id: int
    nonce: int
    gas_price: int
    gas: int
    to: ChecksumAddress | None
    value: int
    data: HexBytes
    seismic: SeismicElements
```

## Fields

| Field       | Type                                     | Description                                               |
| ----------- | ---------------------------------------- | --------------------------------------------------------- |
| `chain_id`  | `int`                                    | Numeric chain identifier (e.g., 5124 for Seismic testnet) |
| `nonce`     | `int`                                    | Sender's transaction count                                |
| `gas_price` | `int`                                    | Gas price in wei                                          |
| `gas`       | `int`                                    | Gas limit                                                 |
| `to`        | `ChecksumAddress \| None`                | Recipient address, or `None` for contract creation        |
| `value`     | `int`                                    | Amount of wei to transfer                                 |
| `data`      | `HexBytes`                               | **Encrypted** calldata (ciphertext)                       |
| `seismic`   | [`SeismicElements`](seismic-elements.md) | Seismic-specific encryption and expiry fields             |

## Examples

### Manual Construction

```python
from seismic_web3 import (
    UnsignedSeismicTx,
    SeismicElements,
    CompressedPublicKey,
    EncryptionNonce,
    Bytes32,
)
from hexbytes import HexBytes

unsigned_tx = UnsignedSeismicTx(
    chain_id=5124,
    nonce=42,
    gas_price=20_000_000_000,  # 20 gwei
    gas=100_000,
    to="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    value=1_000_000_000_000_000_000,  # 1 ETH in wei
    data=HexBytes("0xabcd..."),  # Encrypted calldata
    seismic=SeismicElements(...),
)
```

### Signing with EIP-712

```python
from seismic_web3 import sign_seismic_tx_eip712, PrivateKey

unsigned_tx = UnsignedSeismicTx(...)
private_key = PrivateKey(...)

# Sign and serialize
signed_tx = sign_seismic_tx_eip712(unsigned_tx, private_key)

# Broadcast
tx_hash = w3.eth.send_raw_transaction(signed_tx)
```

### Access from Debug Write

```python
# Debug write returns UnsignedSeismicTx
result = await contract.dwrite.transfer(recipient, amount)

# Access unsigned transaction
unsigned_tx = result.shielded_tx
print(f"Gas: {unsigned_tx.gas}")
print(f"Nonce: {unsigned_tx.nonce}")
print(f"Expires at: {unsigned_tx.seismic.expires_at_block}")
```

### Inspect Seismic Elements

```python
unsigned_tx = UnsignedSeismicTx(...)

# Access Seismic-specific fields
elements = unsigned_tx.seismic
print(f"Message version: {elements.message_version}")
print(f"Encryption nonce: {elements.encryption_nonce.to_0x_hex()}")
print(f"Recent block: {elements.recent_block_hash.to_0x_hex()}")
```

### Build EIP-712 Typed Data

```python
from seismic_web3 import build_seismic_typed_data

unsigned_tx = UnsignedSeismicTx(...)

# Get EIP-712 typed data dict
typed_data = build_seismic_typed_data(unsigned_tx)

# Use with external signers (MetaMask, WalletConnect, etc.)
# signature = await wallet.signTypedData(typed_data)
```

## Field Details

### data (Encrypted Calldata)

The `data` field contains **encrypted** calldata:

* Plaintext calldata is encrypted with AES-GCM
* Encryption key is derived via ECDH with the TEE's public key
* The TEE decrypts it inside the secure enclave

### seismic

Contains [`SeismicElements`](seismic-elements.md) with:

* Encryption parameters (`encryption_pubkey`, `encryption_nonce`)
* Signing mode (`message_version`)
* Expiry and freshness fields (`recent_block_hash`, `expires_at_block`)
* Read/write flag (`signed_read`)

### to (Recipient Address)

* Standard checksummed Ethereum address for normal transactions
* `None` for contract creation (deploys new contract)

### value

Amount of native currency (ETH) to transfer, in wei:

* `1 ETH = 1_000_000_000_000_000_000 wei`
* Can be `0` for pure function calls

## Properties

* **Immutable** - Cannot be modified after construction (`frozen=True`)
* **Type-safe** - All fields are validated at construction
* **Ready to sign** - Contains all fields needed for signing

## Transaction Lifecycle

```
1. Build plaintext transaction
        ↓
2. Encrypt calldata with TEE public key
        ↓
3. Create UnsignedSeismicTx (with encrypted data)
        ↓
4. Sign with private key → signed transaction bytes
        ↓
5. Broadcast to network
```

## Notes

* Created automatically by the SDK's write methods
* Visible in [`DebugWriteResult`](debug-write-result.md) for inspection
* The `data` field is **encrypted** — plaintext is not recoverable without the TEE's private key
* Compatible with both raw signing (`message_version=0`) and EIP-712 (`message_version=2`)

## See Also

* [SeismicElements](seismic-elements.md) - Seismic-specific fields
* [Signature](./) - Signature to apply after signing
* [sign\_seismic\_tx\_eip712](../sign-seismic-tx-eip712/) - Sign this transaction
* [DebugWriteResult](debug-write-result.md) - Contains UnsignedSeismicTx
* [build\_seismic\_typed\_data](../sign-seismic-tx-eip712/build-seismic-typed-data.md) - Convert to EIP-712 dict
