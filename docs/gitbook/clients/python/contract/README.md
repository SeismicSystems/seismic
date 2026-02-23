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

### Example Contract

All code snippets in the Python SDK docs reference the interface below. Only the interface is shown — no deployment needed.

```solidity
interface IExampleVault {
    // ── Public reads (no msg.sender dependency → .tread) ─────
    function getNumber()       external view returns (uint256);
    function isOdd()           external view returns (bool);
    function isActive()        external view returns (bool);
    function getName()         external view returns (string memory);
    function getConfig()       external view returns (uint256 maxDeposit, uint256 feeRate, bool paused);
    function getHolders()      external view returns (address[] memory);
    function getItemCount()    external view returns (uint256);
    function getItems(uint256 offset, uint256 limit)
        external view returns (uint256[] memory);
    function getUserInfo(address user)
        external view returns (string memory name, uint256 balance, bool active);

    // ── Shielded reads (use msg.sender → require .read) ──────
    function getSecretBalance()  external view returns (suint256);

    // ── Writes ───────────────────────────────────────────────
    function setNumber(uint256 value)  external;
    function deposit()                 external payable;
    function withdraw(suint256 amount) external;
    function batchTransfer(
        address[] calldata recipients,
        suint256[] calldata amounts
    ) external;
}
```

Token examples use the real [SRC20](https://github.com/SeismicSystems/src-20) / ERC20 specs (`balanceOf`, `transfer`, `approve`, `allowance`, `totalSupply`, etc.).

***

### Namespaces

`ShieldedContract` gives you five namespaces:

| Namespace | What it does | On-chain visibility |
|-----------|-------------|-------------------|
| `.write` | Encrypted transaction (`TxSeismic` type `0x4a`) | Calldata encrypted |
| `.read` | Encrypted signed `eth_call` | Calldata + result encrypted |
| `.twrite` | Standard `eth_sendTransaction` | Calldata plaintext |
| `.tread` | Standard `eth_call` | Calldata plaintext |
| `.dwrite` | Debug write — like `.write` but returns plaintext + encrypted views | Calldata encrypted |

```python
# Shielded write — encrypted calldata, returns tx hash
tx_hash = contract.write.setNumber(42)

# Shielded read — encrypted signed call, auto-decoded
number = contract.read.getNumber()       # int
is_odd = contract.read.isOdd()           # bool

# Transparent write — standard send_transaction
tx_hash = contract.twrite.setNumber(42)

# Transparent read — standard eth_call, auto-decoded
number = contract.tread.getNumber()      # int

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

If you need to encode calldata outside of a contract call — for example, to pass it to the [low-level API](../guides/shielded-write.md#low-level-api) — you can use [`encode_shielded_calldata`](../namespaces/methods/encode-shielded-calldata.md). This computes the function selector using the original shielded type names (like `suint256`) but encodes the parameters using standard types (like `uint256`):

```python
from seismic_web3.contract.abi import encode_shielded_calldata

data = encode_shielded_calldata(abi, "setNumber", [42])
```
