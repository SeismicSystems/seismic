---
description: Read the current deposit Merkle root from the deposit contract
icon: tree
---

# get_deposit_root()

Read the current deposit Merkle root from the Seismic deposit contract. The deposit root is used by the consensus layer to verify validator deposits.

***

## Overview

The Seismic deposit contract maintains a Merkle tree of all validator deposits. The root of this tree is used by the consensus layer to:
- Verify that deposits have been included
- Validate deposit proofs
- Track the deposit history

This method queries the current root hash.

***

## Signatures

<table>
<thead>
<tr>
<th width="200">Sync</th>
<th>Async</th>
</tr>
</thead>
<tbody>
<tr>
<td>

```python
def get_deposit_root(
    *,
    address: str = DEPOSIT_CONTRACT_ADDRESS,
) -> bytes
```

</td>
<td>

```python
async def get_deposit_root(
    *,
    address: str = DEPOSIT_CONTRACT_ADDRESS,
) -> bytes
```

</td>
</tr>
</tbody>
</table>

***

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `address` | `str` | [`DEPOSIT_CONTRACT_ADDRESS`](../../abis/deposit-contract.md) | Address of the deposit contract to query |

The default address is the genesis deposit contract ([`DEPOSIT_CONTRACT_ADDRESS`](../../abis/deposit-contract.md)).

***

## Returns

**Type:** `bytes` (32 bytes)

A 32-byte hash representing the current Merkle root of all deposits.

```python
# Sync
root = w3.seismic.get_deposit_root()
assert len(root) == 32

# Async
root = await w3.seismic.get_deposit_root()
assert len(root) == 32
```

***

## Examples

### Sync Usage

```python
from seismic_web3 import create_public_client

# Create public client
w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

# Get deposit root
root = w3.seismic.get_deposit_root()

print(f"Deposit root: {root.hex()}")
print(f"Root length: {len(root)} bytes")
```

### Async Usage

```python
from seismic_web3 import create_async_public_client

# Create async public client
w3 = create_async_public_client("https://gcp-1.seismictest.net/rpc")

# Get deposit root
root = await w3.seismic.get_deposit_root()

print(f"Deposit root: {root.hex()}")
```

### With Wallet Client

```python
from seismic_web3 import create_wallet_client, PrivateKey

# Wallet clients also have access to public methods
w3 = create_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=PrivateKey(b"..."),
)

# Get deposit root
root = w3.seismic.get_deposit_root()
print(f"Deposit root: {root.hex()}")
```

### Custom Contract Address

```python
# Query a different deposit contract
root = w3.seismic.get_deposit_root(
    address="0x1234567890123456789012345678901234567890",
)

print(f"Custom contract root: {root.hex()}")
```

### Monitoring Deposits

```python
import time

# Monitor deposit root changes
previous_root = None

while True:
    root = w3.seismic.get_deposit_root()

    if root != previous_root:
        print(f"Deposit root changed: {root.hex()}")
        count = w3.seismic.get_deposit_count()
        print(f"New deposit count: {count}")
        previous_root = root

    time.sleep(12)  # Check every block (~12 seconds)
```

### Comparing with Deposit Count

```python
# Get both root and count
root = w3.seismic.get_deposit_root()
count = w3.seismic.get_deposit_count()

print(f"Deposit root: {root.hex()}")
print(f"Total deposits: {count}")
```

***

## Implementation Details

### Contract Call

This method calls the deposit contract's `get_deposit_root()` function:

```solidity
function get_deposit_root() external view returns (bytes32);
```

The call is made via `eth_call` with the contract address and encoded function selector.

### Encoding

The SDK:
1. Encodes the function call using the deposit contract ABI
2. Calls `eth_call` to the deposit contract
3. Extracts the first 32 bytes of the response
4. Returns them as `bytes`

### Genesis Contract

The default deposit contract address ([`DEPOSIT_CONTRACT_ADDRESS`](../../abis/deposit-contract.md)) is deployed at genesis on all Seismic networks.

***

## Merkle Tree Structure

### Deposit Tree

The deposit contract maintains a Merkle tree where each leaf is:

```
hash(deposit_data)
```

The tree is built incrementally as deposits are made. The root is updated after each deposit.

### Verification

The consensus layer uses the deposit root to:
1. Verify that a deposit is included in the tree
2. Validate the deposit count matches the tree size
3. Ensure deposit data hasn't been tampered with

***

## Use Cases

### Validator Deposits

Before making a deposit, you might want to verify the current root:

```python
# Get current state
root = w3.seismic.get_deposit_root()
count = w3.seismic.get_deposit_count()

print(f"Current root: {root.hex()}")
print(f"Current count: {count}")

# Make deposit
tx_hash = w3.seismic.deposit(
    node_pubkey=...,
    consensus_pubkey=...,
    # ... other params
    value=32 * 10**18,
)

# Wait for transaction
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Verify deposit was included
new_root = w3.seismic.get_deposit_root()
new_count = w3.seismic.get_deposit_count()

print(f"New root: {new_root.hex()}")
print(f"New count: {new_count}")
assert new_count == count + 1
assert new_root != root
```

### Consensus Verification

Consensus clients can query the deposit root to ensure they have the latest deposit data:

```python
# Query deposit root from execution layer
el_root = w3.seismic.get_deposit_root()

# Compare with consensus layer state
# (pseudocode - actual consensus client integration varies)
cl_root = consensus_client.get_deposit_root()

if el_root == cl_root:
    print("Deposit roots match")
else:
    print("WARNING: Deposit root mismatch!")
    print(f"Execution layer: {el_root.hex()}")
    print(f"Consensus layer: {cl_root.hex()}")
```

### Monitoring Tools

Build monitoring tools that track deposit activity:

```python
# Track deposits over time
deposits = []

for block_num in range(start_block, end_block):
    # Get root at specific block
    root = w3.seismic.get_deposit_root(
        # Note: would need block parameter support
    )
    count = w3.seismic.get_deposit_count()

    deposits.append({
        'block': block_num,
        'root': root.hex(),
        'count': count,
    })

# Analyze deposit rate
print(f"Total deposits: {deposits[-1]['count'] - deposits[0]['count']}")
```

***

## Notes

### Public Method

`get_deposit_root()` is available on both:
- **Public clients** (`create_public_client`) — Read-only, no private key
- **Wallet clients** (`create_wallet_client`) — Full capabilities

### Read-Only

This is a `view` function that does not modify state. It does not consume gas or require a transaction.

### Deposit Contract

The deposit contract is a special system contract deployed at genesis. It is used for:
- Accepting validator deposits
- Tracking deposit history
- Providing deposit proofs to the consensus layer

***

## Error Handling

```python
try:
    root = w3.seismic.get_deposit_root()
    print(f"Deposit root: {root.hex()}")
except Exception as e:
    print(f"Failed to fetch deposit root: {e}")
    # Possible causes:
    # - Node is offline
    # - Invalid contract address
    # - RPC error
```

***

## See Also

- [get_deposit_count()](get-deposit-count.md) — Read the total number of deposits
- [deposit()](deposit.md) — Submit a validator deposit
- [Deposit Contract ABI](../../api-reference/abis/deposit-contract.md) — Full contract interface
- [Validator Guide](../../guides/validator-deposits.md) — Guide to making deposits
