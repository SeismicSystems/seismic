---
description: SRC20 token standard ABI constant
icon: file-contract
---

# SRC20_ABI

ABI definition for Seismic's privacy-preserving SRC20 token standard.

## Overview

`SRC20_ABI` is a Python list containing the complete ABI for the `ISRC20` interface, which defines Seismic's privacy-preserving ERC20-compatible token standard. Unlike standard ERC20, SRC20 uses shielded types (`suint256`) for amounts to preserve balance and transfer privacy.

The SRC20 standard maintains the familiar ERC20 API while adding privacy features:
- Token amounts are shielded using `suint256` types
- `balanceOf()` takes no arguments and returns the caller's own balance
- Transfer and approval events emit encrypted amounts

## Import

```python
from seismic_web3 import SRC20_ABI
```

## Type

```python
SRC20_ABI: list[dict[str, Any]]
```

## ABI Contents

The ABI includes the following functions and events:

### Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `name()` | None | `string` | Token name (view) |
| `symbol()` | None | `string` | Token symbol (view) |
| `decimals()` | None | `uint8` | Token decimals (view) |
| `balanceOf()` | None | `uint256` | Caller's token balance (view) |
| `approve(address, suint256)` | `spender`, `amount` | `bool` | Approve spender for shielded amount |
| `transfer(address, suint256)` | `to`, `amount` | `bool` | Transfer shielded amount to recipient |
| `transferFrom(address, address, suint256)` | `from`, `to`, `amount` | `bool` | Transfer shielded amount from approved account |

### Events

| Event | Parameters | Description |
|-------|-----------|-------------|
| `Transfer` | `from` (indexed), `to` (indexed), `encryptKeyHash` (indexed), `encryptedAmount` | Emitted on transfer with encrypted amount |
| `Approval` | `owner` (indexed), `spender` (indexed), `encryptKeyHash` (indexed), `encryptedAmount` | Emitted on approval with encrypted amount |

## Full ABI

```python
SRC20_ABI: list[dict[str, Any]] = [
    {
        "type": "function",
        "name": "name",
        "inputs": [],
        "outputs": [{"name": "", "type": "string", "internalType": "string"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "symbol",
        "inputs": [],
        "outputs": [{"name": "", "type": "string", "internalType": "string"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "decimals",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint8", "internalType": "uint8"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "balanceOf",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256", "internalType": "uint256"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "approve",
        "inputs": [
            {"name": "spender", "type": "address", "internalType": "address"},
            {"name": "amount", "type": "suint256", "internalType": "suint256"},
        ],
        "outputs": [{"name": "", "type": "bool", "internalType": "bool"}],
        "stateMutability": "nonpayable",
    },
    {
        "type": "function",
        "name": "transfer",
        "inputs": [
            {"name": "to", "type": "address", "internalType": "address"},
            {"name": "amount", "type": "suint256", "internalType": "suint256"},
        ],
        "outputs": [{"name": "", "type": "bool", "internalType": "bool"}],
        "stateMutability": "nonpayable",
    },
    {
        "type": "function",
        "name": "transferFrom",
        "inputs": [
            {"name": "from", "type": "address", "internalType": "address"},
            {"name": "to", "type": "address", "internalType": "address"},
            {"name": "amount", "type": "suint256", "internalType": "suint256"},
        ],
        "outputs": [{"name": "", "type": "bool", "internalType": "bool"}],
        "stateMutability": "nonpayable",
    },
    {
        "type": "event",
        "name": "Transfer",
        "inputs": [
            {
                "name": "from",
                "type": "address",
                "indexed": True,
                "internalType": "address",
            },
            {
                "name": "to",
                "type": "address",
                "indexed": True,
                "internalType": "address",
            },
            {
                "name": "encryptKeyHash",
                "type": "bytes32",
                "indexed": True,
                "internalType": "bytes32",
            },
            {
                "name": "encryptedAmount",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
        ],
        "anonymous": False,
    },
    {
        "type": "event",
        "name": "Approval",
        "inputs": [
            {
                "name": "owner",
                "type": "address",
                "indexed": True,
                "internalType": "address",
            },
            {
                "name": "spender",
                "type": "address",
                "indexed": True,
                "internalType": "address",
            },
            {
                "name": "encryptKeyHash",
                "type": "bytes32",
                "indexed": True,
                "internalType": "bytes32",
            },
            {
                "name": "encryptedAmount",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
        ],
        "anonymous": False,
    },
]
```

## Usage Examples

### Creating a Contract Instance

```python
from seismic_web3 import create_wallet_client, SRC20_ABI

# Create client
w3 = create_wallet_client(
    rpc_url="https://gcp-1.seismictest.net/rpc",
    private_key="0x...",
)

# Create contract instance
token = w3.seismic.contract(
    address="0x1234567890123456789012345678901234567890",
    abi=SRC20_ABI,
)
```

### Reading Token Metadata

```python
# Get token name, symbol, and decimals (transparent reads)
name = token.tread.name()
symbol = token.tread.symbol()
decimals = token.tread.decimals()

print(f"Token: {name} ({symbol})")
print(f"Decimals: {decimals}")
```

### Checking Balance

```python
# Get caller's balance (signed read - proves your identity)
balance = token.read.balanceOf()

from eth_abi import decode
balance_value = decode(['uint256'], balance)[0]
print(f"Balance: {balance_value}")
```

### Transferring Tokens

```python
from seismic_web3 import Suint256

# Transfer 100 tokens (with 18 decimals)
recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
amount = Suint256(100 * 10**18)

tx_hash = token.write.transfer(recipient, amount)
print(f"Transfer tx: {tx_hash.hex()}")
```

### Approving Spender

```python
from seismic_web3 import Suint256

# Approve spender for 1000 tokens
spender = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
amount = Suint256(1000 * 10**18)

tx_hash = token.write.approve(spender, amount)
print(f"Approval tx: {tx_hash.hex()}")
```

### Using transferFrom

```python
from seismic_web3 import Suint256

# Transfer from approved account
owner = "0x5B38Da6a701c568545dCfcB03FcB875f56beddC4"
recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
amount = Suint256(50 * 10**18)

tx_hash = token.write.transferFrom(owner, recipient, amount)
print(f"TransferFrom tx: {tx_hash.hex()}")
```

### Listening to Transfer Events

```python
# Get recent transfer events
events = token.events.Transfer.get_logs(from_block="latest" - 1000)

for event in events:
    print(f"Transfer from {event.args['from']} to {event.args['to']}")
    print(f"Encrypted amount: {event.args['encryptedAmount'].hex()}")
    print(f"Key hash: {event.args['encryptKeyHash'].hex()}")
```

## When to Use

Use `SRC20_ABI` when:
- Interacting with SRC20-compliant privacy tokens
- Building DeFi applications with shielded balances
- Deploying privacy-preserving token contracts
- Creating token swap or exchange interfaces
- Implementing token approval flows with privacy

## Key Differences from ERC20

### balanceOf() Takes No Arguments

**Standard ERC20**:
```solidity
function balanceOf(address account) external view returns (uint256);
```

**SRC20**:
```solidity
function balanceOf() external view returns (uint256);
```

In SRC20, `balanceOf()` always returns the caller's balance. Use signed reads (`.read`) to prove your identity.

### Shielded Amounts

All amounts are `suint256` (shielded uint256) instead of plain `uint256`:

```python
# Use Suint256 wrapper for shielded amounts
from seismic_web3 import Suint256

amount = Suint256(100 * 10**18)  # 100 tokens
token.write.transfer(recipient, amount)
```

### Encrypted Events

Events include encryption metadata:
- `encryptKeyHash` - Hash of the encryption key used
- `encryptedAmount` - AES-GCM encrypted amount

To decrypt events, you need access to the viewing key registered in the Directory contract.

## Privacy Features

### What Gets Encrypted
- Token amounts in all operations (transfer, approve, etc.)
- Event amounts (encrypted before emission)

### What Remains Visible
- Token metadata (name, symbol, decimals)
- Participant addresses (from, to, spender)
- Transaction existence and timing
- Function being called

### Viewing Keys

Users must register viewing keys in the Directory contract to decrypt their balances and amounts:

```python
from seismic_web3 import DIRECTORY_ABI, DIRECTORY_ADDRESS, Suint256

# Register viewing key
directory = w3.eth.contract(address=DIRECTORY_ADDRESS, abi=DIRECTORY_ABI)

viewing_key = Suint256(...)  # Your AES-256 key as suint256
tx_hash = directory.functions.setKey(viewing_key).transact()
```

## Contract Source

This ABI matches the `ISRC20` interface defined in:
```
contracts/src/seismic-std-lib/SRC20.sol
```

## See Also

- [SRC20 Token Guide](../src20/) - Complete SRC20 usage guide
- [Suint256 Type](../api-reference/types/suint256.md) - Shielded integer type
- [ShieldedContract](../contract/) - Contract interaction patterns
- [Directory Contract](directory.md) - Viewing key management
- [DEPOSIT_CONTRACT_ABI](deposit-contract.md) - Validator deposits
