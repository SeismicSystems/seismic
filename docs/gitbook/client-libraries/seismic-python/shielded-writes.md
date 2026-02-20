---
icon: pen
---

# Shielded Writes

A shielded write encrypts calldata so it is never visible outside the TEE -- not in the mempool, not in block data, not in transaction traces.

## How it works

When you call `contract.write.someMethod(...)`, the SDK:

1. Retrieves the nonce and current block hash from the node
2. Constructs a `TxSeismic` (type `0x4A`) with encryption metadata
3. Encrypts calldata using AES-GCM with a key derived via ECDH between an ephemeral keypair and the TEE public key
4. Signs and broadcasts the transaction

The encrypted calldata is bound to the transaction context (chain ID, nonce, block hash, expiry) via AES-GCM additional authenticated data, so it cannot be replayed or tampered with.

## Using the contract wrapper

```python
contract = w3.seismic.contract(address="0x...", abi=abi)

tx_hash = contract.write.transfer(to, amount)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
```

## Low-level API

For contract deployments or pre-encoded calldata:

```python
from hexbytes import HexBytes

tx_hash = w3.seismic.send_shielded_transaction(
    to="0x...",
    data=HexBytes("0x..."),
)
```

## Security parameters

Default settings provide a 100-block expiration window. Customize with `SeismicSecurityParams`:

```python
from seismic_web3 import SeismicSecurityParams

params = SeismicSecurityParams(
    blocks_window=50,
    encryption_nonce=None,      # random by default
    recent_block_hash=None,     # latest by default
    expires_at_block=None,      # computed from blocks_window
)

tx_hash = contract.write.transfer(to, amount, seismic_params=params)
```

## Debug mode

Inspect the plaintext and encrypted forms of a transaction:

```python
debug = contract.dwrite.transfer(to, amount)
print(debug.plaintext_tx.data)   # unencrypted calldata
print(debug.shielded_tx.data)    # encrypted calldata
print(debug.tx_hash)             # transaction hash
```

## EIP-712 support

For external wallet integration, the SDK supports EIP-712 typed data signing, matching the `eth_signTypedData_v4` standard.
