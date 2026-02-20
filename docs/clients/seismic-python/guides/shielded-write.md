---
description: Encrypted transactions — lifecycle, security parameters, and the low-level API
icon: shield-halved
---

# Shielded Write

***

### How it works

When you call `contract.write.someMethod(...)`, the SDK:

1. Fetches your nonce and the latest block hash from the node
2. Builds a `TxSeismic` (type `0x4a`) with encryption metadata
3. Encrypts the calldata using AES-GCM with a shared key derived via ECDH between your ephemeral keypair and the node's TEE public key
4. Signs and broadcasts the transaction

The encrypted calldata is bound to the transaction context (chain ID, nonce, block hash, expiry) via AES-GCM additional authenticated data, so it can't be replayed or tampered with.

***

### Security parameters

Every shielded transaction includes a block-hash freshness check and an expiry window. The defaults are sane (100-block window, random nonce, latest block hash), but you can override them per-call:

```python
from seismic_web3 import SeismicSecurityParams

params = SeismicSecurityParams(
    blocks_window=50,          # expires after 50 blocks instead of 100
    encryption_nonce=None,     # random (default)
    recent_block_hash=None,    # latest block (default)
    expires_at_block=None,     # computed from blocks_window (default)
)

tx_hash = contract.write.setNumber(42, security=params)
result = contract.read.getNumber(security=params)
```

***

### Low-level API

When you need manual control over calldata — contract deployments, pre-encoded data, or bypassing the `ShieldedContract` abstraction:

```python
from hexbytes import HexBytes

# Shielded transaction with raw calldata
tx_hash = w3.seismic.send_shielded_transaction(
    to="0x...",
    data=HexBytes("0x..."),
    value=0,
    gas=100_000,
    gas_price=10**9,
)

# Debug shielded transaction
debug = w3.seismic.debug_send_shielded_transaction(
    to="0x...",
    data=HexBytes("0x..."),
)
```

***

### EIP-712 typed data

{% hint style="info" %}
Most Python users sign locally with a private key and don't need this. EIP-712 is relevant if you're integrating with MetaMask, WalletConnect, or other external signers that use `eth_signTypedData_v4`.
{% endhint %}

```python
from seismic_web3 import sign_seismic_tx_eip712, build_seismic_typed_data

# Sign using EIP-712 (same RLP output, different ECDSA message hash)
signed_tx = sign_seismic_tx_eip712(unsigned_tx, private_key)

# Get the typed data dict (matches eth_signTypedData_v4 format)
typed_data = build_seismic_typed_data(unsigned_tx)
typed_data["domain"]       # {"name": "Seismic Transaction", "version": "2", ...}
typed_data["primaryType"]  # "TxSeismic"
typed_data["message"]      # all 13 transaction fields
```
