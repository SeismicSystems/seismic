---
description: Read metadata, check balance, approve, and transfer with SRC20
icon: coins
---

# SRC20 Workflow

End-to-end example using the SRC20 token standard — Seismic's privacy-preserving ERC20.

```python
import os
from eth_abi import decode
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract("0xYourTokenAddress", SRC20_ABI)

# 1) Read token metadata (transparent reads — no auth needed)
name = decode(["string"], bytes(token.tread.name()))[0]
symbol = decode(["string"], bytes(token.tread.symbol()))[0]
decimals = decode(["uint8"], bytes(token.tread.decimals()))[0]
print(name, symbol, decimals)

# 2) Read your balance (signed read — proves identity)
balance = decode(["uint256"], bytes(token.read.balanceOf()))[0]
print("balance", balance)

# 3) Approve and transfer (shielded writes)
approve_tx = token.write.approve("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", 500)
w3.eth.wait_for_transaction_receipt(approve_tx)

transfer_tx = token.write.transfer("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", 100)
receipt = w3.eth.wait_for_transaction_receipt(transfer_tx)
print("transfer status", receipt["status"])
```

## Notes

- `balanceOf()` takes no arguments — it uses `msg.sender`, so you must use `.read` (signed read), not `.tread`
- `approve` and `transfer` amounts use shielded types (`suint256`) — encrypted in the transaction calldata
- Transparent metadata like `name()`, `symbol()`, and `decimals()` can use `.tread` since they don't depend on the caller

## See Also

- [SRC20 Documentation](../src20/) — Full token standard reference
- [Signed Reads](signed-reads.md) — Why `.read` is needed for `balanceOf()`
- [Shielded Write](shielded-write.md) — How `.write` encrypts calldata
