---
description: Build EIP-712 typed data dict for external signers
icon: code
---

# build_seismic_typed_data

Build the EIP-712 typed data dict for a Seismic transaction.

## Overview

`build_seismic_typed_data()` constructs a JSON-serializable dictionary matching the format expected by `eth_signTypedData_v4` (used by MetaMask, WalletConnect, and other external signers). This enables human-readable transaction signing in wallet UIs.

## Signature

```python
def build_seismic_typed_data(tx: UnsignedSeismicTx) -> dict[str, Any]
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tx` | [`UnsignedSeismicTx`](../transaction-types/unsigned-seismic-tx.md) | Yes | The unsigned Seismic transaction |

## Returns

| Type | Description |
|------|-------------|
| `dict[str, Any]` | Dict with keys `types`, `primaryType`, `domain`, `message` |

## Examples

### Basic Usage

```python
from seismic_web3 import build_seismic_typed_data, UnsignedSeismicTx
import json

unsigned_tx = UnsignedSeismicTx(...)

# Build typed data
typed_data = build_seismic_typed_data(unsigned_tx)

# Pretty print
print(json.dumps(typed_data, indent=2))
```

### Use with External Signer (MetaMask)

```python
from seismic_web3 import build_seismic_typed_data

unsigned_tx = UnsignedSeismicTx(...)

# Build typed data for MetaMask
typed_data = build_seismic_typed_data(unsigned_tx)

# Send to frontend
# const signature = await ethereum.request({
#   method: 'eth_signTypedData_v4',
#   params: [account, JSON.stringify(typedData)]
# })
```

### Inspect Structure

```python
from seismic_web3 import build_seismic_typed_data

unsigned_tx = UnsignedSeismicTx(...)
typed_data = build_seismic_typed_data(unsigned_tx)

# Inspect keys
print(f"Keys: {typed_data.keys()}")
# Keys: dict_keys(['types', 'primaryType', 'domain', 'message'])

# Inspect domain
print(f"Domain: {typed_data['domain']}")
# Domain: {'name': 'Seismic Transaction', 'version': '2', 'chainId': 1, ...}

# Inspect message fields
print(f"Message fields: {typed_data['message'].keys()}")
```

### Verify Against Manual Hash

```python
from seismic_web3 import build_seismic_typed_data, eip712_signing_hash

unsigned_tx = UnsignedSeismicTx(...)

# Build typed data
typed_data = build_seismic_typed_data(unsigned_tx)

# Compute expected hash
expected_hash = eip712_signing_hash(unsigned_tx)

# If signed externally, verify the signature recovers to expected_hash
```

### Export for Testing

```python
from seismic_web3 import build_seismic_typed_data
import json

unsigned_tx = UnsignedSeismicTx(...)

# Export typed data for testing
typed_data = build_seismic_typed_data(unsigned_tx)

with open('typed_data.json', 'w') as f:
    json.dump(typed_data, f, indent=2)
```

## Output Structure

The returned dictionary has this structure:

```python
{
    "types": {
        "EIP712Domain": [
            {"name": "name", "type": "string"},
            {"name": "version", "type": "string"},
            {"name": "chainId", "type": "uint256"},
            {"name": "verifyingContract", "type": "address"}
        ],
        "TxSeismic": [
            {"name": "chainId", "type": "uint64"},
            {"name": "nonce", "type": "uint64"},
            {"name": "gasPrice", "type": "uint128"},
            {"name": "gasLimit", "type": "uint64"},
            {"name": "to", "type": "address"},
            {"name": "value", "type": "uint256"},
            {"name": "input", "type": "bytes"},
            {"name": "encryptionPubkey", "type": "bytes"},
            {"name": "encryptionNonce", "type": "uint96"},
            {"name": "messageVersion", "type": "uint8"},
            {"name": "recentBlockHash", "type": "bytes32"},
            {"name": "expiresAtBlock", "type": "uint64"},
            {"name": "signedRead", "type": "bool"}
        ]
    },
    "primaryType": "TxSeismic",
    "domain": {
        "name": "Seismic Transaction",
        "version": "2",
        "chainId": 1,
        "verifyingContract": "0x0000000000000000000000000000000000000000"
    },
    "message": {
        "chainId": 1,
        "nonce": 42,
        "gasPrice": 20000000000,
        "gasLimit": 100000,
        "to": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
        "value": 1000000000000000000,
        "input": "0xabcd...",
        "encryptionPubkey": "0x02...",
        "encryptionNonce": 12345,
        "messageVersion": 2,
        "recentBlockHash": "0x1234...",
        "expiresAtBlock": 12345678,
        "signedRead": false
    }
}
```

## Field Encoding

### types

Defines the structure of both the domain and the message:
- **EIP712Domain** - Standard domain fields
- **TxSeismic** - All transaction fields with their Solidity types

### primaryType

The primary type being signed:
- Always `"TxSeismic"` for Seismic transactions

### domain

Identifies the signing context:
- **name** - `"Seismic Transaction"`
- **version** - `"2"` (matches message_version)
- **chainId** - Numeric chain identifier
- **verifyingContract** - `"0x0000000000000000000000000000000000000000"` (off-chain signing)

### message

All transaction field values:
- Numeric fields as integers
- Address fields as checksummed hex strings
- Bytes fields as `"0x"` prefixed hex strings
- Boolean fields as `true`/`false`

## Hex String Conversions

Certain fields are converted to hex strings for JSON serialization:

```python
"input": HexBytes(tx.data).to_0x_hex()
"encryptionPubkey": HexBytes(bytes(se.encryption_pubkey)).to_0x_hex()
"recentBlockHash": HexBytes(bytes(se.recent_block_hash)).to_0x_hex()
```

## encryption_nonce Conversion

The 12-byte nonce is converted to an integer:

```python
enc_nonce_int = int.from_bytes(bytes(se.encryption_nonce), "big")
```

Then included as a numeric value in the message.

## Use Cases

### External Wallet Integration

```python
# Backend generates typed data
typed_data = build_seismic_typed_data(unsigned_tx)

# Send to frontend
return jsonify(typed_data)

# Frontend signs with MetaMask
# const sig = await ethereum.request({
#   method: 'eth_signTypedData_v4',
#   params: [account, JSON.stringify(typedData)]
# })
```

### Hardware Wallet Signing

```python
# Generate typed data for hardware wallet
typed_data = build_seismic_typed_data(unsigned_tx)

# Use with Ledger/Trezor via appropriate library
# signature = ledger.sign_typed_data(typed_data)
```

### Testing and Debugging

```python
# Export for manual inspection
typed_data = build_seismic_typed_data(unsigned_tx)
print(json.dumps(typed_data, indent=2))

# Verify all fields are correct before signing
```

## Notes

- Used for integration with external signers
- The hash computed from this data matches [`eip712_signing_hash(tx)`](eip712-signing-hash.md)
- JSON-serializable (no Python-specific types)
- Follows `eth_signTypedData_v4` specification

## Warnings

- **to field handling** - If `tx.to` is `None`, it's encoded as the zero address
- **Hex encoding** - All bytes fields must be `"0x"` prefixed hex strings
- **Integer sizes** - Ensure values fit in declared types (e.g., `uint64`)

## See Also

- [sign_seismic_tx_eip712](sign-seismic-tx-eip712.md) - Signs using this data internally
- [eip712_signing_hash](eip712-signing-hash.md) - Computes hash from typed data
- [domain_separator](domain-separator.md) - Domain component
- [struct_hash](struct-hash.md) - Message component
- [UnsignedSeismicTx](../transaction-types/unsigned-seismic-tx.md) - Input transaction type
