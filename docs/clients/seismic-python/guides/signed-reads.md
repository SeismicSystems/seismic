---
description: Encrypted eth_call that proves your identity to the contract
icon: signature
---

# Signed Read

A signed read (`.read`) builds a full `TxSeismic` just like a [shielded write](shielded-write.md), but targets the `eth_call` endpoint instead of broadcasting a transaction. The node decrypts the calldata inside the TEE, executes the call, encrypts the result, and returns it.

***

### Why this matters

Any contract function that depends on `msg.sender` needs a signed read. A plain `eth_call` zeros out the `from` field, so the contract wouldn't know who's asking.

```python
# This proves your identity to the contract
result = contract.read.getBalance()

# This does NOT — msg.sender will be 0x0
result = contract.tread.getBalance()
```

A common example: a contract with a `balanceOf()` that takes no arguments and uses `msg.sender` internally to look up the caller's balance. If you call it with `.tread`, the contract sees the zero address as the sender and returns its balance — which is almost certainly zero.

***

### What gets encrypted

Both the calldata you send and the result you get back are encrypted. An observer watching the network can see that you made a call to a particular contract address, but not what function you called or what was returned.

***

### Low-level API

If you need to make a signed read with raw calldata (e.g., pre-encoded or non-ABI interactions):

```python
from hexbytes import HexBytes

result = w3.seismic.signed_call(
    to="0x...",
    data=HexBytes("0x..."),
    gas=30_000_000,
)
```

***

### Async

Signed reads work identically with the async client — just `await` the call:

```python
result = await contract.read.getBalance()
```
