---
icon: coins
---

# SRC20

SRC20 is Seismic's privacy-focused token standard. It extends ERC20 with shielded balances and transfer amounts using `suint256`, hiding transaction details from external observers.

## Setup

The SDK includes `SRC20_ABI` built-in:

```python
from seismic_web3.abis import SRC20_ABI

token = w3.seismic.contract(address="0x...", abi=SRC20_ABI)
```

## Metadata (transparent)

Token metadata is public and uses transparent reads:

```python
name = token.tread.name()
symbol = token.tread.symbol()
decimals = token.tread.decimals()
```

## Balances (signed read)

Balances require a signed read because the contract checks `msg.sender`. Using `.tread` would send the zero address as the caller and return the wrong result.

```python
# Correct -- proves your identity
balance = token.read.balanceOf()

# Wrong -- returns zero address's balance
balance = token.tread.balanceOf()
```

## Transfers

```python
# Direct transfer (shielded)
tx_hash = token.write.transfer(to_address, amount)

# Approve a spender
tx_hash = token.write.approve(spender_address, amount)

# Check allowance (transparent -- allowances are public)
allowance = token.tread.allowance(owner, spender)

# Transfer on behalf of owner (requires prior approval)
tx_hash = token.write.transferFrom(from_address, to_address, amount)
```
