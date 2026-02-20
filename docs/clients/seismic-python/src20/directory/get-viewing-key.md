---
description: Fetch your viewing key from the Directory contract
icon: download
---

# get\_viewing\_key

Fetch your viewing key from the Directory genesis contract using signed read authentication.

## Overview

`get_viewing_key()` and `async_get_viewing_key()` retrieve your registered viewing key from the Directory contract at `0x1000...0004`. The function uses a **signed read** to authenticate `msg.sender`, ensuring only the key owner can retrieve their key.

This function is typically used internally by [`watch_src20_events()`](../event-watching/watch-src20-events.md), but can also be called directly when you need access to your viewing key.

## Signature

```python
def get_viewing_key(
    w3: Web3,
    encryption: EncryptionState,
    private_key: PrivateKey,
) -> Bytes32

async def async_get_viewing_key(
    w3: AsyncWeb3,
    encryption: EncryptionState,
    private_key: PrivateKey,
) -> Bytes32
```

## Parameters

| Parameter     | Type                                                       | Required | Description                            |
| ------------- | ---------------------------------------------------------- | -------- | -------------------------------------- |
| `w3`          | `Web3` or `AsyncWeb3`                                      | Yes      | Web3 instance with Seismic support     |
| `encryption`  | [`EncryptionState`](../../client/encryption-state.md)      | Yes      | Encryption state from wallet client    |
| `private_key` | [`PrivateKey`](../../api-reference/bytes32/private-key.md) | Yes      | 32-byte signing key for authentication |

## Returns

| Type                                      | Description                 |
| ----------------------------------------- | --------------------------- |
| [`Bytes32`](../../api-reference/bytes32/) | 32-byte AES-256 viewing key |

## Raises

* `ValueError` - If no viewing key is registered for the caller's address

## Examples

### Basic Usage (Sync)

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import get_viewing_key

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Fetch your viewing key
viewing_key = get_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
)

print(f"Your viewing key: {viewing_key.hex()}")
```

### With Error Handling

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import get_viewing_key, register_viewing_key
import os

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

try:
    viewing_key = get_viewing_key(
        w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
    )
    print(f"Existing key: {viewing_key.hex()}")
except ValueError:
    print("No key registered. Registering new key...")
    new_key = os.urandom(32)
    tx_hash = register_viewing_key(
        w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
        key=new_key,
    )
    w3.eth.wait_for_transaction_receipt(tx_hash)
    print(f"Registered new key: {new_key.hex()}")
```

### Use with Event Watcher

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import get_viewing_key, watch_src20_events_with_key

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Fetch viewing key once
viewing_key = get_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
)

# Use with explicit key watcher (avoids re-fetching)
watcher = watch_src20_events_with_key(
    w3,
    viewing_key=viewing_key,
    on_transfer=lambda log: print(f"Transfer: {log.decrypted_amount}"),
)
```

### Async Usage

```python
from seismic_web3 import create_async_wallet_client, PrivateKey
from seismic_web3.src20 import async_get_viewing_key
import asyncio

async def main():
    private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
    w3 = await create_async_wallet_client(
        "wss://gcp-1.seismictest.net/ws",
        private_key=private_key,
    )

    # Fetch your viewing key
    viewing_key = await async_get_viewing_key(
        w3,
        encryption=w3.seismic.encryption,
        private_key=private_key,
    )

    print(f"Your viewing key: {viewing_key.hex()}")

asyncio.run(main())
```

### Cache the Key

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import get_viewing_key
import json

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Fetch and cache the key
viewing_key = get_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
)

# Save to file for later use
cache = {"viewing_key": viewing_key.hex()}
with open("viewing_key_cache.json", "w") as f:
    json.dump(cache, f)

print("Viewing key cached")
```

### Compare with Stored Key

```python
from seismic_web3 import create_wallet_client, PrivateKey, Bytes32
from seismic_web3.src20 import get_viewing_key

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Fetch from chain
onchain_key = get_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
)

# Compare with locally stored key
local_key = Bytes32(bytes.fromhex("YOUR_STORED_KEY_HEX"))

if onchain_key == local_key:
    print("Keys match!")
else:
    print("WARNING: Keys do not match!")
```

### Export for Sharing

```python
from seismic_web3 import create_wallet_client, PrivateKey
from seismic_web3.src20 import get_viewing_key

private_key = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY"))
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)

# Fetch viewing key
viewing_key = get_viewing_key(
    w3,
    encryption=w3.seismic.encryption,
    private_key=private_key,
)

# Export for sharing (e.g., with auditor or monitoring service)
print("Share this viewing key to allow others to decrypt your events:")
print(viewing_key.hex())
print("\nWARNING: Anyone with this key can see all your SRC20 events!")
```

## How It Works

1. **Encode calldata** - Encodes `getKey()` with no arguments
2. **Signed read** - Calls [`signed_call()`](../../api-reference/sign-seismic-tx-eip712/) to the Directory contract at `0x1000...0004`
3. **Authentication** - The contract validates `msg.sender` matches the key owner
4. **Response** - Returns the 32-byte viewing key
5. **Validation** - Checks that the key is not zero (indicates no key registered)

## Notes

* Uses signed read for authentication (ensures only the owner can retrieve the key)
* Makes one RPC call to fetch the key
* The key is returned as a `Bytes32` instance
* Raises `ValueError` if no key is registered
* The function validates that the returned key is non-zero
* The key is stored encrypted on-chain and decrypted in the TEE

## Warnings

* **Key security** - The viewing key allows decrypting all SRC20 events; keep it secure
* **No key registered** - Raises `ValueError` if you haven't registered a key yet
* **Authentication required** - Requires private key and encryption state
* **RPC call** - Makes a network call; cache the result if using frequently

## See Also

* [register\_viewing\_key](register-viewing-key.md) - Register a viewing key in Directory
* [check\_has\_key](check-has-key.md) - Check if an address has a registered key
* [watch\_src20\_events](../event-watching/watch-src20-events.md) - Watch events with automatic key fetching
* [watch\_src20\_events\_with\_key](../event-watching/watch-src20-events-with-key.md) - Watch with explicit key
* [Bytes32](../../api-reference/bytes32/) - 32-byte value type
