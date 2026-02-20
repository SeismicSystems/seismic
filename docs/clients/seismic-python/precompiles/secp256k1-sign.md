---
description: On-chain secp256k1 ECDSA signing
icon: signature
---

# secp256k1\_sign

Sign messages on-chain using Mercury EVM's secp256k1 signing precompile.

## Overview

`secp256k1_sign()` and `async_secp256k1_sign()` call the secp256k1 signing precompile at address `0x69` to create ECDSA signatures. The message is hashed with the EIP-191 personal-sign prefix before signing.

## Signature

```python
def secp256k1_sign(
    w3: Web3,
    *,
    sk: PrivateKey,
    message: str,
) -> HexBytes

async def async_secp256k1_sign(
    w3: AsyncWeb3,
    *,
    sk: PrivateKey,
    message: str,
) -> HexBytes
```

## Parameters

| Parameter | Type                                                    | Required | Description                               |
| --------- | ------------------------------------------------------- | -------- | ----------------------------------------- |
| `w3`      | `Web3` or `AsyncWeb3`                                   | Yes      | Web3 instance connected to a Seismic node |
| `sk`      | [`PrivateKey`](../api-reference/bytes32/private-key.md) | Yes      | 32-byte secp256k1 private key             |
| `message` | `str`                                                   | Yes      | Message string to sign                    |

## Returns

| Type       | Description                                 |
| ---------- | ------------------------------------------- |
| `HexBytes` | ECDSA signature bytes (65 bytes: r + s + v) |

## Examples

### Basic Usage

```python
from seismic_web3.precompiles import secp256k1_sign
from seismic_web3 import PrivateKey
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Sign a message
private_key = PrivateKey.from_hex("0x1234...")
message = "Hello, Seismic!"
signature = secp256k1_sign(w3, sk=private_key, message=message)
print(f"Signature: {signature.hex()}")
print(f"Length: {len(signature)} bytes")
```

### Async Usage

```python
from seismic_web3.precompiles import async_secp256k1_sign
from seismic_web3 import PrivateKey
from web3 import AsyncWeb3

async def main():
    w3 = AsyncWeb3(AsyncWeb3.AsyncHTTPProvider("https://gcp-1.seismictest.net/rpc"))

    private_key = PrivateKey.from_hex("0x1234...")
    message = "Async signing"
    signature = await async_secp256k1_sign(w3, sk=private_key, message=message)
    print(f"Signature: {signature.hex()}")

# Run with asyncio.run(main())
```

### Sign Multiple Messages

```python
from seismic_web3.precompiles import secp256k1_sign
from seismic_web3 import PrivateKey
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

private_key = PrivateKey.from_hex("0x1234...")
messages = ["Message 1", "Message 2", "Message 3"]

for msg in messages:
    signature = secp256k1_sign(w3, sk=private_key, message=msg)
    print(f"Signature for '{msg}': {signature.hex()[:20]}...")
```

### Verify Signature Off-Chain

```python
from seismic_web3.precompiles import secp256k1_sign
from seismic_web3 import PrivateKey
from web3 import Web3, Account
from eth_hash.auto import keccak

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Sign message on-chain
private_key = PrivateKey.from_hex("0x1234...")
message = "Verify me"
signature = secp256k1_sign(w3, sk=private_key, message=message)

# Verify signature off-chain using web3.py
# Compute EIP-191 message hash
prefix = f"\x19Ethereum Signed Message:\n{len(message)}".encode()
message_hash = keccak(prefix + message.encode())

# Recover signer address
recovered_address = Account.recover_message_hash(message_hash, signature=signature)
expected_address = Account.from_key(bytes(private_key)).address

assert recovered_address == expected_address
print(f"Signature verified! Signer: {recovered_address}")
```

### Extract Signature Components

```python
from seismic_web3.precompiles import secp256k1_sign
from seismic_web3 import PrivateKey
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

private_key = PrivateKey.from_hex("0x1234...")
signature = secp256k1_sign(w3, sk=private_key, message="Extract components")

# Signature is 65 bytes: r (32) + s (32) + v (1)
r = signature[:32]
s = signature[32:64]
v = signature[64]

print(f"r: {r.hex()}")
print(f"s: {s.hex()}")
print(f"v: {v}")
```

### Generate Private Key

```python
from seismic_web3.precompiles import secp256k1_sign
from seismic_web3 import PrivateKey
from web3 import Web3
import os

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

# Generate random private key
private_key = PrivateKey(os.urandom(32))

message = "Signed with generated key"
signature = secp256k1_sign(w3, sk=private_key, message=message)
print(f"Signature: {signature.hex()}")
```

### Sign Transaction Hash

```python
from seismic_web3.precompiles import secp256k1_sign
from seismic_web3 import PrivateKey, Bytes32
from web3 import Web3

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

private_key = PrivateKey.from_hex("0x1234...")

# Sign a transaction hash (as a hex string)
tx_hash = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
signature = secp256k1_sign(w3, sk=private_key, message=tx_hash)
print(f"Transaction signature: {signature.hex()}")
```

### Compare On-Chain vs Off-Chain Signing

```python
from seismic_web3.precompiles import secp256k1_sign
from seismic_web3 import PrivateKey
from web3 import Web3, Account

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

private_key = PrivateKey.from_hex("0x1234...")
message = "Compare signing methods"

# On-chain signing (via precompile)
onchain_sig = secp256k1_sign(w3, sk=private_key, message=message)

# Off-chain signing (via web3.py)
account = Account.from_key(bytes(private_key))
offchain_sig = account.sign_message(
    {"message": message.encode(), "version": "E"}
).signature

# Signatures will differ due to randomness in 'k' value
# But both are valid signatures for the same message
print(f"On-chain:  {onchain_sig.hex()[:40]}...")
print(f"Off-chain: {offchain_sig.hex()[:40]}...")
```

### Sign with Message Context

```python
from seismic_web3.precompiles import secp256k1_sign
from seismic_web3 import PrivateKey
from web3 import Web3
import json

w3 = Web3(Web3.HTTPProvider("https://gcp-1.seismictest.net/rpc"))

private_key = PrivateKey.from_hex("0x1234...")

# Sign structured message
context = {
    "action": "transfer",
    "amount": 1000,
    "recipient": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
}
message = json.dumps(context, sort_keys=True)
signature = secp256k1_sign(w3, sk=private_key, message=message)
print(f"Context signature: {signature.hex()}")
```

## How It Works

1.  **Hash message** - Applies EIP-191 personal-sign prefix and keccak256 hash:

    ```python
    prefix = f"\x19Ethereum Signed Message:\n{len(message)}"
    message_hash = keccak256(prefix + message)
    ```
2. **Encode parameters** - ABI-encodes the private key and message hash
3. **Call precompile** - Issues an `eth_call` to address `0x69` with 3000 gas
4. **Sign hash** - Precompile performs ECDSA signing on secp256k1 curve
5. **Return signature** - Returns 65-byte signature (r + s + v)

## Gas Cost

Fixed gas cost: **3000 gas**

The cost is constant regardless of message length (since the message is hashed before signing).

## EIP-191 Message Hashing

The precompile follows EIP-191 personal-sign format, equivalent to:

```python
prefix = f"\x19Ethereum Signed Message:\n{len(message)}"
message_hash = keccak256(prefix.encode() + message.encode())
```

This ensures compatibility with standard Ethereum message signing (e.g., MetaMask's `personal_sign`).

## Signature Format

The returned signature is 65 bytes:

* **r** (32 bytes): Signature component
* **s** (32 bytes): Signature component
* **v** (1 byte): Recovery ID (27 or 28)

This format is compatible with `ecrecover` and standard Ethereum signature verification.

## Notes

* Uses secp256k1 curve (same as Ethereum)
* Message is automatically prefixed with EIP-191 header
* Signatures are non-deterministic (random 'k' value)
* Compatible with MetaMask and other Ethereum wallets
* Can be verified on-chain using `ecrecover` precompile

## Warnings

* **Private key security** - Never expose or log private keys
* **Message format** - Message is treated as UTF-8 string
* **Signature malleability** - Standard ECDSA signatures are malleable (use EIP-2098 compact signatures if needed)
* **Non-deterministic** - Multiple signatures of the same message will differ

## See Also

* [PrivateKey](../api-reference/bytes32/private-key.md) - 32-byte private key type
* [Bytes32](../api-reference/bytes32/) - 32-byte value type
* [ecdh](ecdh.md) - ECDH key exchange
* [EIP-191](https://eips.ethereum.org/EIPS/eip-191) - Signed data standard
