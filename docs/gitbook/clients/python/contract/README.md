---
description: Instantiating contracts and interacting through shielded and transparent namespaces
icon: file-lock
---

# Contract Instance

***

### Instantiation

```python
contract = w3.seismic.contract(address="0x...", abi=ABI)
```

The ABI works the same as in `web3.py`. If your contract uses shielded types (`suint256`, `sbool`, `saddress`), the SDK remaps them to their standard counterparts for parameter encoding while keeping the original shielded names for function selector computation.

***

### Namespaces

`ShieldedContract` gives you five namespaces:

| Namespace | What it does | On-chain visibility |
|-----------|-------------|-------------------|
| `.write` | Encrypted transaction (`TxSeismic` type `0x4a`) | Calldata hidden |
| `.read` | Encrypted signed `eth_call` | Calldata + result hidden |
| `.twrite` | Standard `eth_sendTransaction` | Calldata visible |
| `.tread` | Standard `eth_call` | Calldata visible |
| `.dwrite` | Debug write — like `.write` but returns plaintext + encrypted views | Calldata hidden |

```python
# Shielded write — encrypted calldata, returns tx hash
tx_hash = contract.write.setNumber(42)

# Shielded read — encrypted signed call, returns raw bytes
result = contract.read.getNumber()

# Transparent write — standard send_transaction
tx_hash = contract.twrite.setNumber(42)

# Transparent read — standard eth_call
result = contract.tread.getNumber()

# Debug write — returns plaintext + encrypted views + tx hash
debug = contract.dwrite.setNumber(42)
debug.plaintext_tx.data  # unencrypted calldata
debug.shielded_tx.data   # encrypted calldata
debug.tx_hash            # transaction hash
```

Write namespaces accept optional keyword arguments for transaction parameters:

```python
tx_hash = contract.write.deposit(value=10**18, gas=100_000, gas_price=10**9)
```

***

### Encoding calldata manually

If you need to encode calldata outside of a contract call — for example, to pass it to the [low-level API](shielded-write.md#low-level-api) — you can use `encode_shielded_calldata`. This computes the function selector using the original shielded type names (like `suint256`) but encodes the parameters using standard types (like `uint256`):

```python
from seismic_web3.contract.abi import encode_shielded_calldata

data = encode_shielded_calldata(abi, "setNumber", [42])
```
