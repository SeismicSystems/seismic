---
description: Fetch the TEE's compressed secp256k1 public key for ECDH encryption
icon: key
---

# get_tee_public_key()

Fetch the TEE's (Trusted Execution Environment) compressed secp256k1 public key used for ECDH-based encryption in Seismic shielded transactions.

***

## Overview

Every Seismic node runs a TEE that generates an ephemeral secp256k1 keypair on startup. The TEE's public key is used by clients to derive a shared AES encryption key via ECDH (Elliptic Curve Diffie-Hellman). This shared key encrypts transaction calldata end-to-end from the client to the TEE.

This method retrieves that public key from the node.

***

## Signatures

<table>
<thead>
<tr>
<th width="200">Sync</th>
<th>Async</th>
</tr>
</thead>
<tbody>
<tr>
<td>

```python
def get_tee_public_key(
) -> CompressedPublicKey
```

</td>
<td>

```python
async def get_tee_public_key(
) -> CompressedPublicKey
```

</td>
</tr>
</tbody>
</table>

***

## Parameters

This method takes no parameters.

***

## Returns

**Type:** [`CompressedPublicKey`](../../api-reference/types/compressed-public-key.md) (33-byte `bytes`)

A 33-byte compressed secp256k1 public key in SEC format:
- First byte: `0x02` or `0x03` (y-coordinate parity)
- Next 32 bytes: x-coordinate

***

## Examples

### Sync Usage

```python
from seismic_web3 import create_public_client

# Create public client
w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

# Fetch TEE public key
tee_key = w3.seismic.get_tee_public_key()

print(f"TEE public key: {tee_key.hex()}")
print(f"Key length: {len(tee_key)} bytes")
print(f"Y-parity: {'even' if tee_key[0] == 0x02 else 'odd'}")
```

### Async Usage

```python
from seismic_web3 import create_async_public_client

# Create async public client
w3 = create_async_public_client("https://gcp-1.seismictest.net/rpc")

# Fetch TEE public key
tee_key = await w3.seismic.get_tee_public_key()

print(f"TEE public key: {tee_key.hex()}")
print(f"Key length: {len(tee_key)} bytes")
```

### With Wallet Client

```python
from seismic_web3 import create_wallet_client, PrivateKey

# Wallet clients also have access to public methods
w3 = create_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=PrivateKey(b"..."),
)

# Fetch TEE public key
tee_key = w3.seismic.get_tee_public_key()
print(f"TEE public key: {tee_key.hex()}")
```

***

## Implementation Details

### RPC Call

This method calls the custom Seismic RPC method:

```
seismic_getTEEPublicKey
```

No parameters are required. The node returns the TEE's current public key.

### Encryption State

When you create a wallet client with `create_wallet_client()`, the SDK:
1. Calls `get_tee_public_key()` automatically
2. Generates an ephemeral client keypair
3. Derives a shared [AES-GCM](https://en.wikipedia.org/wiki/Galois/Counter_Mode) key via [ECDH](https://en.wikipedia.org/wiki/Elliptic-curve_Diffie%E2%80%93Hellman)
4. Stores the encryption state in `w3.seismic.encryption`

You don't need to call this method manually unless you're implementing custom encryption logic.

### Key Rotation

The TEE's public key is ephemeral and regenerated when the node restarts. If the key changes:
- Existing encryption states become invalid
- You must recreate wallet clients to re-derive the shared key

The SDK does not automatically detect key rotation. Monitor your node's uptime or handle RPC errors that might indicate stale encryption state.

***

## Notes

### Public Method

`get_tee_public_key()` is available on both:
- **Public clients** (`create_public_client`) — Read-only, no private key
- **Wallet clients** (`create_wallet_client`) — Full capabilities

### No Caching

The method queries the node every time. If you need to call it frequently, consider caching the result locally. However, be aware that the key may change if the node restarts.

### Testing

In test environments, the TEE may return a deterministic key for reproducibility. Consult your node's configuration for details.

***

## Error Handling

```python
try:
    tee_key = w3.seismic.get_tee_public_key()
    print(f"TEE key: {tee_key.hex()}")
except Exception as e:
    print(f"Failed to fetch TEE key: {e}")
    # Node may be offline or RPC method not supported
```

***

## See Also

- [EncryptionState](../../api-reference/client/encryption-state.md) — Client-side encryption state structure
- [create_wallet_client()](../../api-reference/client/create-wallet-client.md) — Automatically derives encryption state
- [Shielded Write Guide](../../guides/shielded-write.md) — How encryption is used in transactions
- [send_shielded_transaction()](send-shielded-transaction.md) — Send encrypted transactions
