---
description: Transparent read namespace for standard contract calls
icon: eye
---

# .tread Namespace

The `.tread` namespace provides standard (non-encrypted) contract read operations using `eth_call`. Use this for public queries where privacy is not required.

***

## Overview

When you call `contract.tread.functionName(...)`, the SDK:

1. Encodes your function call using the contract ABI
2. Constructs a standard `eth_call` request
3. Executes the call against the node's state
4. ABI-decodes the result and returns Python values

**No encryption** is applied — calldata and results are visible to anyone observing the request.

***

## Usage Pattern

```python
result = contract.tread.functionName(arg1, arg2, ...)
```

- **Sync**: Returns decoded Python value immediately
- **Async**: Returns decoded Python value (must `await`)

Return values are **automatically ABI-decoded**:
- **Single output** (e.g. `returns (uint256)`): returns the value directly (`int`, `bool`, `str`, etc.)
- **Multiple outputs** (e.g. `returns (uint256, bool)`): returns a `tuple`
- **No outputs defined in ABI**: returns `None`

***

## Parameters

### Function Arguments

Pass function arguments as positional parameters:

```python
# No arguments
result = contract.tread.totalSupply()

# Single argument
result = contract.tread.balanceOf(address)

# Multiple arguments
result = contract.tread.allowance(owner, spender)
```

### No Transaction Options

Unlike `.read`, `.tread` does **not** accept keyword arguments like `value`, `gas`, or `security`.

The call uses default parameters:
- No value transfer
- Reasonable gas limit (set by node)
- No special security parameters

***

## Examples

### Sync Usage

```python
from seismic_web3 import create_public_client

# Create client and contract
w3 = create_public_client(...)
contract = w3.seismic.contract(address="0x...", abi=ABI)

# Results are automatically decoded
total_supply = contract.tread.totalSupply()  # int
print(f"Total supply: {total_supply}")
```

### Async Usage

```python
from seismic_web3 import create_async_public_client

# Create async client and contract
w3 = await create_async_public_client(...)
contract = w3.seismic.contract(address="0x...", abi=ABI)

# Results are automatically decoded
total_supply = await contract.tread.totalSupply()
print(f"Total supply: {total_supply}")
```

### Reading with Arguments

```python
# Single argument — returns decoded value directly
balance = contract.tread.balanceOf("0x1234...")  # int

# Multiple arguments
allowance = contract.tread.allowance("0x1234...", "0x5678...")  # int
```

### Multiple Return Values

```python
# Function returns multiple values — returned as tuple
name, balance, active = contract.tread.getUserInfo(user_address)

print(f"Name: {name}")
print(f"Balance: {balance}")
print(f"Active: {active}")
```

### Array and Struct Returns

```python
# Array return — returned as list
holders = contract.tread.getHolders()  # list[str]

# Struct-like return (multiple outputs) — returned as tuple
max_deposit, fee_rate, paused = contract.tread.getConfig()
```

***

## Return Value

Returns the **ABI-decoded Python value**:

- **Single output** (e.g. `returns (uint256)`): the value directly — `int`, `bool`, `str`, `bytes`, etc.
- **Multiple outputs** (e.g. `returns (uint112, uint112, uint32)`): a `tuple` of decoded values
- **No outputs defined in ABI**: `None`

```python
# Single return value — unwrapped from tuple
total_supply = contract.tread.totalSupply()  # int
name = contract.tread.name()                  # str

# Multiple return values — tuple
max_deposit, fee_rate, paused = contract.tread.getConfig()

# Array return
holders = contract.tread.getHolders()         # list
```

***

## Key Limitation: No msg.sender

Standard `eth_call` does **not** prove your identity. The contract sees `msg.sender` as `0x0000000000000000000000000000000000000000`.

### Problem Example

SRC20's `balanceOf()` takes no arguments and returns the caller's balance:

```solidity
// SRC20 balanceOf — uses msg.sender internally
function balanceOf() external view returns (suint256) {
    return balances[msg.sender];
}
```

With **transparent read** (`.tread`):
```python
# msg.sender is 0x0 — returns 0x0's balance (int)
balance = contract.tread.balanceOf()  # Almost always 0
```

With **signed read** (`.read`):
```python
# msg.sender is your address — returns your balance (int)
balance = contract.read.balanceOf()  # Your actual balance
```

### When .tread Works

Use `.tread` when the contract function:
- Does **not** check `msg.sender`
- Is purely computational (no authentication)
- Returns public data

**Examples**:
```solidity
// These work fine with .tread
function totalSupply() external view returns (uint256);
function balanceOf(address account) external view returns (uint256);
function name() external view returns (string);
function getConfig() external view returns (uint256, uint256, bool);
```

### When .tread Fails

Use `.read` when the contract function:
- Checks `msg.sender` for permissions
- Returns caller-specific data
- Has any access control

**Examples**:
```solidity
// These require .read (signed read) — they use msg.sender internally
function balanceOf() external view returns (suint256);   // SRC20
function allowance() external view returns (suint256);    // SRC20
```

***

## Privacy Implications

### What's Visible

**Everything is visible:**
- Your IP address (to the node you're querying)
- Contract address (what contract you're reading)
- Function selector (which function you're calling)
- All function arguments
- The result returned

### Example

```python
# This call is completely visible
result = contract.tread.balanceOf("0x1234...")
```

The node (and anyone monitoring) can see:
- You're checking the balance of `0x1234...`
- Which contract you're querying
- The result (the balance amount)

***

## When to Use .tread

### Good Use Cases

- **Public data queries** — Total supply, token name, public balances
- **No authentication needed** — Function doesn't check `msg.sender`
- **Cost optimization** — Free (no gas cost, no encryption overhead)
- **Public explorers** — Block explorers, analytics, public dashboards
- **Testing** — Quick tests where privacy doesn't matter

### Examples

```python
# Public token metadata — auto-decoded
name = contract.tread.name()            # str
symbol = contract.tread.symbol()        # str
decimals = contract.tread.decimals()    # int

# Public balances
balance = contract.tread.balanceOf(address)  # int

# Public configuration
item_count = contract.tread.getItemCount()   # int
```

***

## When NOT to Use .tread

### Use `.read` Instead When

- Function checks `msg.sender`
- Caller-specific data is needed
- Privacy is required
- Access control is involved

### Examples (Use `.read` for these)

```python
# SRC20 balanceOf uses msg.sender — use .read
my_balance = contract.read.balanceOf()

# SRC20 allowance uses msg.sender — use .read
my_allowance = contract.read.allowance(spender_address)
```

***

## Comparison with Other Namespaces

| Namespace | Encryption | Proves Identity | Gas Cost | Use Case |
|-----------|-----------|----------------|----------|----------|
| `.read` | Encrypted | Yes (`msg.sender` is your address) | None | Access-controlled reads |
| `.tread` | No encryption | No (`msg.sender` is `0x0`) | None | Public reads |

***

## Error Handling

```python
try:
    balance = contract.tread.balanceOf(address)
    print(f"Balance: {balance}")

except Exception as e:
    print(f"Call failed: {e}")
```

***

## Best Practices

### When to Choose .tread

**Checklist**:
- [ ] Function does not check `msg.sender`
- [ ] No authentication required
- [ ] Data is public anyway
- [ ] Privacy is not a concern
- [ ] Want zero overhead (no encryption)

### When to Choose .read

**Checklist**:
- [ ] Function checks `msg.sender`
- [ ] Caller-specific data needed
- [ ] Access control involved
- [ ] Privacy required
- [ ] Need to prove identity

### Common Mistakes

**Mistake**: Using `.tread` for caller-specific functions
```python
# BAD: SRC20 balanceOf uses msg.sender — returns 0x0's balance
balance = contract.tread.balanceOf()
```

**Fix**: Use `.read` to prove your identity
```python
# GOOD: Returns your actual balance
balance = contract.read.balanceOf()
```

***

## Async Patterns

### Concurrent Reads

```python
import asyncio

# Read multiple values concurrently — each is auto-decoded
total_supply, name, symbol = await asyncio.gather(
    contract.tread.totalSupply(),
    contract.tread.name(),
    contract.tread.symbol(),
)
```

### Read with Timeout

```python
import asyncio

try:
    result = await asyncio.wait_for(
        contract.tread.totalSupply(),
        timeout=5.0,  # 5 seconds
    )
except asyncio.TimeoutError:
    print("Read timed out")
```

***

## Standard Web3.py Behavior

The `.tread` namespace uses standard `eth_call` under the hood. All web3.py call features work:

### Block Number

You can specify a block number (though not directly via `.tread`):

```python
# Use low-level eth_call for historical reads
from seismic_web3.contract.abi import encode_shielded_calldata

data = encode_shielded_calldata(abi, "totalSupply", [])
result = w3.eth.call(
    {"to": contract_address, "data": data},
    block_identifier=12345678,  # Historical block
)
```

***

## Low-Level Alternative

Direct `eth_call`:

```python
result = w3.eth.call({
    "to": contract_address,
    "data": "0x...",  # Encoded calldata
})
```

The `.tread` namespace is more convenient as it handles ABI encoding automatically.

***

## Performance

`.tread` is the **fastest** read method:
- No encryption overhead
- No signature computation
- Direct `eth_call` to node
- Can be cached/memoized easily

Use it for:
- Public data that changes infrequently
- High-frequency queries (price feeds, balances)
- Analytics and dashboards

***

## See Also

- [.read Namespace](read.md) — Encrypted signed reads with `msg.sender`
- [.twrite Namespace](twrite.md) — Transparent writes
- [.write Namespace](write.md) — Encrypted writes
- [Contract Instance](../README.md) — Contract wrapper reference
- [Web3.py Documentation](https://web3py.readthedocs.io/) — Standard Ethereum calls
