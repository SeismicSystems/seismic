---
description: On-chain secp256k1 ECDSA signing
icon: signature
---

# secp256k1_sign

Sign a message with the secp256k1 signing precompile at `0x69`.

## Overview

`secp256k1_sign()` and `async_secp256k1_sign()`:
- hash the input `message` with an EIP-191 personal-sign prefix
- sign the hash using the provided private key
- return signature bytes (`HexBytes`)

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

| Parameter | Type | Required | Description |
|---|---|---|---|
| `w3` | `Web3` / `AsyncWeb3` | Yes | Connected Seismic client |
| `sk` | [`PrivateKey`](../api-reference/types/private-key.md) | Yes | 32-byte secp256k1 private key |
| `message` | `str` | Yes | Message text to sign |

## Returns

| Type | Description |
|---|---|
| `HexBytes` | Signature bytes (r/s plus recovery byte) |

## Examples

### Basic Usage

```python
import os
from seismic_web3 import PrivateKey, create_public_client
from seismic_web3 import precompiles as sp

w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

sk = PrivateKey(os.urandom(32))
sig = sp.secp256k1_sign(w3, sk=sk, message="hello")
print(sig.hex())
```

### Verify Off-Chain

```python
from eth_account import Account
from eth_account.messages import encode_defunct

message = "hello"
sig = sp.secp256k1_sign(w3, sk=sk, message=message)

recovered = Account.recover_message(encode_defunct(text=message), signature=sig)
expected = Account.from_key(bytes(sk)).address
assert recovered == expected
```

### Async Usage

```python
import os
from seismic_web3 import create_async_public_client
from seismic_web3 import PrivateKey
from seismic_web3 import precompiles as sp

async def main():
    w3 = create_async_public_client("https://gcp-1.seismictest.net/rpc")
    sk = PrivateKey(os.urandom(32))
    sig = await sp.async_secp256k1_sign(w3, sk=sk, message="async hello")
    print(sig.hex())
```

### Read Signature Components

```python
r = int.from_bytes(sig[0:32], "big")
s = int.from_bytes(sig[32:64], "big")
v = sig[64] if len(sig) > 64 else None
```

## Hashing Behavior

The SDK hashes with:

```python
from eth_hash.auto import keccak

prefix = f"\x19Ethereum Signed Message:\n{len(message)}".encode()
message_hash = keccak(prefix + message.encode())
```

## Gas Cost

Fixed cost: `3000`.

## Notes

- `message` is UTF-8 encoded before hashing/signing.
- Keep private keys out of logs and telemetry.
- The output format is suitable for typical Ethereum signature verification flows.

## See Also

- [PrivateKey](../api-reference/types/private-key.md)
- [EIP-191](https://eips.ethereum.org/EIPS/eip-191)
- [ecdh](ecdh.md)
