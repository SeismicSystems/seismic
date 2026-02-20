---
description: Signed read namespace for encrypted contract queries
icon: signature
---

# .read Namespace

The `.read` namespace provides encrypted contract read operations using signed `eth_call`. Both calldata and return values are encrypted end-to-end, and the transaction is signed to prove your identity to the contract.

***

## Overview

When you call `contract.read.functionName(...)`, the SDK:

1. Encodes your function call using the contract ABI
2. Encrypts the calldata using AES-GCM with a shared key derived via ECDH
3. Constructs a full `TxSeismic` with encryption metadata (nonce, block hash, expiry)
4. Signs the transaction with your private key
5. Sends the signed transaction to `eth_call` endpoint
6. Node decrypts calldata inside TEE, executes the call, encrypts the result
7. Returns encrypted result (which the SDK can decrypt if needed)

Unlike standard `eth_call`, signed reads **prove your identity** to the contract via `msg.sender`, enabling access-controlled queries.

***

## Usage Pattern

```python
result = contract.read.functionName(arg1, arg2, ...)
```

- **Sync**: Returns decoded Python value immediately
- **Async**: Returns decoded Python value (must `await`)

Return values are **automatically ABI-decoded**:
- **Single output** (e.g. `returns (uint256)`): returns the value directly (`int`, `bool`, `str`, etc.)
- **Multiple outputs** (e.g. `returns (uint256, bool)`): returns a `tuple`
- **No outputs or empty result**: returns empty `HexBytes`

***

## Parameters

### Function Arguments

Pass function arguments as positional parameters:

```python
# No arguments
result = contract.read.getBalance()

# Single argument
result = contract.read.getTokenBalance(token_address)

# Multiple arguments
result = contract.read.getAllowance(owner, spender)
```

### Call Options (Keyword Arguments)

All call options are **optional** keyword arguments:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `value` | `int` | `0` | ETH value for the call (in wei) |
| `gas` | `int` | `30_000_000` | Gas limit for the call |
| `security` | [`SeismicSecurityParams`](../../api-reference/transaction-types/seismic-security-params.md) \| `None` | `None` | Custom security parameters (block expiry, nonce, etc.) |

**Note**: Unlike `.write`, there is no `gas_price` parameter because reads don't consume gas.

***

## Examples

### Sync Usage

```python
from seismic_web3 import create_wallet_client

# Create client and contract
w3 = create_wallet_client(...)
contract = w3.seismic.contract(address="0x...", abi=ABI)

# Results are automatically decoded
balance = contract.read.getBalance()  # int
print(f"Balance: {balance}")

is_active = contract.read.isActive()  # bool
name = contract.read.getName()        # str
```

### Async Usage

```python
from seismic_web3 import create_async_wallet_client

# Create async client and contract
w3 = await create_async_wallet_client(...)
contract = w3.seismic.contract(address="0x...", abi=ABI)

# Results are automatically decoded
balance = await contract.read.getBalance()
print(f"Balance: {balance}")
```

### Reading with Arguments

```python
# Single argument — returns decoded value directly
owner_balance = contract.read.balanceOf(owner_address)

# Multiple arguments
allowance = contract.read.allowance(owner_address, spender_address)
```

### Custom Gas Limit

```python
# Increase gas for complex reads
result = contract.read.complexCalculation(
    arg1,
    arg2,
    gas=50_000_000,
)
```

### Custom Security Parameters

```python
from seismic_web3 import SeismicSecurityParams

# Use longer expiry window
security = SeismicSecurityParams(blocks_window=200)

result = contract.read.getBalance(security=security)
```

### Simulating Value Transfer

```python
# Simulate sending 1 ETH with the read
result = contract.read.simulateDeposit(
    amount,
    value=10**18,  # 1 ETH in wei
)
```

***

## Return Value

Returns the **ABI-decoded Python value**:

- **Single output** (e.g. `returns (uint256)`): the value directly — `int`, `bool`, `str`, `bytes`, etc.
- **Multiple outputs** (e.g. `returns (uint256, bool, address)`): a `tuple` of decoded values
- **No outputs or empty result**: empty `HexBytes`

```python
# Single return value — unwrapped from tuple
balance = contract.read.getBalance()       # int
is_odd = contract.read.isOdd()             # bool
name = contract.read.getName()             # str

# Multiple return values — tuple
name, age, active = contract.read.getUserInfo(address)

# Array return
values = contract.read.getArray()          # list
```

***

## Why Use Signed Reads?

### Identity Matters

Many contracts use `msg.sender` to determine access or return personalized data. For example, SRC20's `balanceOf()` takes no arguments and returns the caller's balance:

```solidity
// SRC20 balanceOf — uses msg.sender internally
function balanceOf() external view returns (suint256) {
    return balances[msg.sender];
}
```

With **signed read** (`.read`):
```python
# Proves your identity — msg.sender is your address
balance = contract.read.balanceOf()  # Returns your balance (int)
```

With **transparent read** (`.tread`):
```python
# Does NOT prove identity — msg.sender is 0x0
balance = contract.tread.balanceOf()  # Returns 0x0's balance (usually 0)
```

### Common Use Cases

**Use signed reads when the contract function**:
- Checks `msg.sender` permissions
- Returns data specific to the caller
- Has access control (owner-only, role-based, etc.)
- Uses `msg.sender` in any way

***

## Privacy Guarantees

### What Gets Encrypted

- Function selector (4 bytes)
- All function arguments
- Return value (encrypted by node)

An observer watching the network can see:
- Your address (call sender)
- Contract address (call recipient)
- That you made a call (via signed transaction)

But **cannot** see:
- Which function you called
- What arguments you passed
- What data was returned

### What Remains Visible

These fields are **not** encrypted:
- `from` — Your wallet address
- `to` — Contract address
- Gas limit
- Transaction metadata (block hash, expiry, encryption nonce)

***

## Comparison with .tread

| Feature | `.read` (Signed) | `.tread` (Transparent) |
|---------|-----------------|----------------------|
| Calldata encryption | Yes | No |
| Result encryption | Yes | No |
| Proves identity | Yes (`msg.sender` is your address) | No (`msg.sender` is `0x0`) |
| Gas cost | None (doesn't broadcast) | None (doesn't broadcast) |
| Use case | Access-controlled reads | Public reads |

***

## Security Considerations

### Block Hash Freshness

Like shielded writes, signed reads include a recent block hash. The node validates:
1. Block hash corresponds to a real block
2. Block is recent (within freshness window)

### Transaction Expiry

Calls include an expiry block number:
- Default: **100 blocks** (~20 minutes)
- After expiry, node rejects the call

### Nonce Uniqueness

Each call uses a cryptographically random 12-byte encryption nonce. The SDK generates random nonces automatically.

### Private Key Security

Signed reads **require your private key** to sign the call. Never expose your private key or share it with untrusted parties.

***

## Error Handling

```python
try:
    balance = contract.read.getBalance()
    print(f"Balance: {balance}")

except ValueError as e:
    print(f"Call failed: {e}")
```

***

## Best Practices

### Use `.read` When

- Contract function checks `msg.sender`
- You need to prove your identity
- Data is access-controlled (requires authentication)
- Privacy is required (both query and result)

### Don't Use `.read` When

- Function is public and doesn't check `msg.sender`
- No privacy needed (use `.tread` for lower overhead)
- Function is a simple getter with no access control

### Production Checklist

- Verify contract function actually uses `msg.sender`
- Use appropriate gas limits for complex reads
- Results are auto-decoded — no manual `eth_abi.decode` needed
- Consider caching read results if called frequently

***

## Low-Level Alternative

If you need more control (e.g., pre-encoded calldata):

```python
from hexbytes import HexBytes

result = w3.seismic.signed_call(
    to="0x...",
    data=HexBytes("0x..."),
    gas=30_000_000,
)
```

See [Signed Read Guide](../../guides/signed-reads.md#low-level-api) for details.

***

## Async Patterns

### Concurrent Reads

```python
import asyncio

# Read multiple values concurrently — each is auto-decoded
balance, name, status = await asyncio.gather(
    contract.read.getBalance(),
    contract.read.getName(),
    contract.read.getStatus(),
)
```

### Read with Timeout

```python
import asyncio

try:
    result = await asyncio.wait_for(
        contract.read.getBalance(),
        timeout=10.0,  # 10 seconds
    )
except asyncio.TimeoutError:
    print("Read timed out")
```

***

## See Also

- [Signed Read Guide](../../guides/signed-reads.md) — Full workflow and examples
- [SeismicSecurityParams](../../api-reference/transaction-types/seismic-security-params.md) — Security parameter reference
- [.write Namespace](write.md) — Encrypted writes
- [.tread Namespace](tread.md) — Transparent reads without encryption
- [ShieldedContract](../README.md) — Contract wrapper reference
