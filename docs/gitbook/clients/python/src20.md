---
description: Privacy-preserving ERC20 tokens with shielded balances
icon: coins
---

# SRC20 Tokens

SRC20 is Seismic's privacy-preserving ERC20. Balances and transfer amounts use shielded types (`suint256`), so they're hidden from external observers. The SDK ships with `SRC20_ABI` built in.

```python
from seismic_web3 import create_wallet_client, SRC20_ABI, PrivateKey

w3 = create_wallet_client("http://127.0.0.1:8545", private_key=PrivateKey(...))

token = w3.seismic.contract(address="0x...", abi=SRC20_ABI)
```

***

### Metadata

Token metadata isn't shielded, so you can use plain transparent reads:

```python
name = token.tread.name()         # b"TestToken"
symbol = token.tread.symbol()     # b"TEST"
decimals = token.tread.decimals() # b'\x12' (18)
```

***

### Balances

```python
raw = token.read.balanceOf()
balance = int.from_bytes(raw, "big")
```

{% hint style="warning" %}
`balanceOf()` takes **no arguments**. The contract uses `msg.sender` internally, so you must use `.read` (a [signed read](signed-reads.md)) — not `.tread`. A plain `eth_call` zeros out the `from` field, which means the contract sees the zero address as the sender and returns its balance — which is almost certainly zero.
{% endhint %}

***

### Approvals

Approve a spender to transfer tokens on your behalf, then have them call `transferFrom`:

```python
# Approve
tx_hash = token.write.approve("0xSpender...", 500)
w3.eth.wait_for_transaction_receipt(tx_hash)

# Check allowance — transparent read, no signed read needed
raw = token.tread.allowance("0xOwner...", "0xSpender...")
allowance = int.from_bytes(raw, "big")
```

***

### Transfers

```python
# Direct transfer
tx_hash = token.write.transfer("0xRecipient...", 100)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Transfer on behalf of owner (called by the approved spender)
tx_hash = token.write.transferFrom("0xOwner...", "0xRecipient...", 100)
```
