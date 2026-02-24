---
description: Encrypted transactions — lifecycle, security parameters, and the low-level API
icon: shield-halved
---

# Shielded Write

A shielded write encrypts calldata, builds a `TxSeismic` (type `0x4a`), signs it, and broadcasts it.

## How it works

When you call `contract.write.someMethod(...)`, the SDK:

1. Fetches your nonce and the latest block hash from the node
2. Builds a `TxSeismic` (type `0x4a`) with encryption metadata
3. Encrypts the calldata using AES-GCM with a shared key derived via ECDH between your ephemeral keypair and the node's TEE public key
4. Signs and broadcasts the transaction

The encrypted calldata is bound to the transaction context (chain ID, nonce, block hash, expiry) via AES-GCM additional authenticated data, so it can't be replayed or tampered with.

## Basic example

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)

tx_hash = token.write.transfer("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", 1000)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
print(receipt["status"])  # 1
```

## Security parameters

Every shielded transaction includes a block-hash freshness check and an expiry window. The defaults are sane (100-block window, random nonce, latest block hash), but you can override them per-call:

```python
from seismic_web3 import SeismicSecurityParams

params = SeismicSecurityParams(
    blocks_window=50,       # expires after 50 blocks instead of 100
    encryption_nonce=None,  # random (default)
    recent_block_hash=None, # latest block (default)
    expires_at_block=None,  # computed from blocks_window (default)
)

tx_hash = token.write.transfer(
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    1000,
    security=params,
)
```

Security parameters work on both `.write` and `.read` calls.

## Low-level API

When you need manual control over calldata — contract deployments, pre-encoded data, or bypassing the `ShieldedContract` abstraction:

```python
from seismic_web3.contract.abi import encode_shielded_calldata

data = encode_shielded_calldata(SRC20_ABI, "transfer", [
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    1000,
])

tx_hash = w3.seismic.send_shielded_transaction(
    to="0xYourTokenAddress",
    data=data,
    value=0,
    gas=100000,
    gas_price=10**9,
)
```

## Debug mode

`.dwrite` sends a real transaction and also returns plaintext/encrypted views for debugging:

```python
result = token.dwrite.transfer("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", 1000)

print(result.tx_hash.hex())
print(result.plaintext_tx.data.hex())
print(result.shielded_tx.data.hex())
```

## Async variant

```python
w3 = await SEISMIC_TESTNET.async_wallet_client(pk)
token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)

tx_hash = await token.write.transfer("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", 1000)
receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
```

## See Also

- [ShieldedContract](../contract/shielded-contract.md) — `.write`, `.dwrite` namespace reference
- [SeismicSecurityParams](../api-reference/transaction-types/seismic-security-params.md) — Security parameter details
- [Signed Reads](signed-reads.md) — Encrypted reads using the same encryption flow
