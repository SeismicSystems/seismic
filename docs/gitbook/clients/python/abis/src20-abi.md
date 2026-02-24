---
description: SRC20 token interface ABI constant
icon: coins
---

# SRC20\_ABI

ABI for the `ISRC20` interface — Seismic's privacy-preserving ERC20 standard. Token amounts use shielded types (`suint256`) and `balanceOf()` takes no arguments (uses `msg.sender` internally).

```python
from seismic_web3 import SRC20_ABI

SRC20_ABI: list[dict[str, Any]]
```

## Functions

| Function | Parameters | Returns | Mutability | Description |
| --- | --- | --- | --- | --- |
| `name()` | — | `string` | `view` | Token name |
| `symbol()` | — | `string` | `view` | Token symbol |
| `decimals()` | — | `uint8` | `view` | Token decimals |
| `balanceOf()` | — | `uint256` | `view` | Caller's balance (no address arg) |
| `approve` | `spender: address`, `amount: suint256` | `bool` | `nonpayable` | Approve shielded amount |
| `transfer` | `to: address`, `amount: suint256` | `bool` | `nonpayable` | Transfer shielded amount |
| `transferFrom` | `from: address`, `to: address`, `amount: suint256` | `bool` | `nonpayable` | Transfer from approved account |

## Events

| Event | Indexed Parameters | Data |
| --- | --- | --- |
| `Transfer` | `from: address`, `to: address`, `encryptKeyHash: bytes32` | `encryptedAmount: bytes` |
| `Approval` | `owner: address`, `spender: address`, `encryptKeyHash: bytes32` | `encryptedAmount: bytes` |

## balanceOf — SRC20 vs ERC20

Standard ERC20:

```solidity
function balanceOf(address account) external view returns (uint256);
```

SRC20:

```solidity
function balanceOf() external view returns (uint256);
```

In SRC20, `balanceOf()` always returns the **caller's** balance. Use `.read` (a signed read that proves identity), not `.tread`.

## Example

```python
import os
from eth_abi import decode
from seismic_web3 import PrivateKey, SEISMIC_TESTNET, SRC20_ABI

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SEISMIC_TESTNET.wallet_client(pk)

token = w3.seismic.contract("0x00000000219ab540356cBB839Cbe05303d7705Fa", SRC20_ABI)

# Transparent metadata (no auth needed)
name = decode(["string"], bytes(token.tread.name()))[0]
symbol = decode(["string"], bytes(token.tread.symbol()))[0]

# Shielded balance (signed read)
balance = decode(["uint256"], bytes(token.read.balanceOf()))[0]

# Shielded transfer
tx_hash = token.write.transfer("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", 100)
w3.eth.wait_for_transaction_receipt(tx_hash)
```

## See Also

- [SRC20](../src20/) — Token usage guide
- [Event Watching](../src20/event-watching/) — Decrypt Transfer/Approval events
- [ShieldedContract](../contract/shielded-contract.md) — `.read`, `.write`, `.tread` namespaces
