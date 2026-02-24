---
description: SRC20 token usage with shielded reads/writes
icon: coins
---

# SRC20

SRC20 is Seismic's privacy-preserving ERC20. Balances and transfer amounts use shielded types (`suint256`), so they're hidden from external observers. The SDK ships with `SRC20_ABI` built in.

## Example

```python
import os
from eth_abi import decode
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract("0x00000000219ab540356cBB839Cbe05303d7705Fa", SRC20_ABI)

# Transparent metadata reads
name = decode(["string"], bytes(token.tread.name()))[0]
symbol = decode(["string"], bytes(token.tread.symbol()))[0]
decimals = decode(["uint8"], bytes(token.tread.decimals()))[0]

# Shielded balance read (signed read — uses msg.sender)
balance = decode(["uint256"], bytes(token.read.balanceOf()))[0]

# Shielded transfer
tx_hash = token.write.transfer("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", 100)
w3.eth.wait_for_transaction_receipt(tx_hash)
```

## Notes

- `balanceOf()` in `SRC20_ABI` takes **no address argument** — the contract uses `msg.sender` internally, so you must use `.read` (a signed read), not `.tread`
- Use `.read` for shielded reads and `.write` for shielded writes
- Use `.tread` for transparent metadata reads (`name`, `symbol`, `decimals`, `allowance`)

## See Also

- [Event Watching](event-watching/) — Watch and decrypt SRC20 Transfer/Approval events
- [Types](types/) — Decrypted event data structures
- [Intelligence Providers](intelligence-providers/) — Viewing key management
