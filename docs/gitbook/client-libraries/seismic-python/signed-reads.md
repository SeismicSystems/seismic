---
icon: signature
---

# Signed Reads

A signed read (`.read`) constructs a `TxSeismic` like a shielded write, but targets `eth_call` instead of broadcasting a transaction. The node decrypts the calldata inside the TEE, executes the call, encrypts the result, and returns it.

## Why signed reads?

A plain `eth_call` zeroes out the `from` field. Contracts that check `msg.sender` -- like `balanceOf()` -- would see the zero address and return the wrong result.

Signed reads prove your identity to the contract without submitting an on-chain transaction.

## Example

```python
# This proves msg.sender -- returns YOUR balance
balance = contract.read.balanceOf()

# This does NOT prove identity -- returns zero address's balance
balance = contract.tread.balanceOf()
```

## Privacy

Both the request and response are encrypted. An observer can see which contract you interacted with, but cannot determine which function was called or what data was returned.

## Low-level API

```python
from hexbytes import HexBytes

result = w3.seismic.signed_call(
    to="0x...",
    data=HexBytes("0x..."),
    gas=30_000_000,
)
```

## Async

```python
result = await contract.read.balanceOf()
```
