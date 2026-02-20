---
icon: file-contract
---

# Contract Instance

Create a contract instance with `w3.seismic.contract()`. The SDK automatically remaps shielded types (`suint256`, `sbool`, `saddress`) to standard equivalents for ABI encoding while preserving the original names for function selectors.

## Creating a contract

```python
from seismic_web3.abis import SRC20_ABI

contract = w3.seismic.contract(address="0x...", abi=SRC20_ABI)
```

## Five namespaces

Each contract exposes five operation modes:

### `.write` -- Shielded write

Sends an encrypted `TxSeismic` (type `0x4A`). Calldata is hidden from observers.

```python
tx_hash = contract.write.transfer(to, amount)
```

### `.read` -- Signed read

Sends an encrypted `eth_call` that proves `msg.sender`. Both the request and response are encrypted.

```python
balance = contract.read.balanceOf()
```

### `.twrite` -- Transparent write

Sends a standard transaction with visible calldata.

```python
tx_hash = contract.twrite.approve(spender, amount)
```

### `.tread` -- Transparent read

Sends a standard `eth_call`. No identity proof.

```python
name = contract.tread.name()
symbol = contract.tread.symbol()
```

### `.dwrite` -- Debug write

Returns plaintext, encrypted, and transaction hash for inspection.

```python
debug = contract.dwrite.transfer(to, amount)
print(debug.plaintext_tx.data)
print(debug.shielded_tx.data)
print(debug.tx_hash)
```

## Transaction parameters

Write methods accept optional keyword arguments:

```python
contract.write.deposit(value=10**18, gas=100_000, gas_price=10**9)
```

## Manual calldata encoding

For advanced use cases, encode calldata externally using shielded type selectors:

```python
from seismic_web3 import encode_shielded_calldata

data = encode_shielded_calldata(abi, "transfer", [to, amount])
```
