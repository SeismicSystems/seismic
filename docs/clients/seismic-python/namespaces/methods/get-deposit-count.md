---
description: Read the total number of validator deposits from the deposit contract
icon: hashtag
---

# get\_deposit\_count

Read the total number of validator deposits that have been made to the Seismic deposit contract. This count is used by the consensus layer to track deposit history.

***

## Overview

The Seismic deposit contract maintains a counter that increments with each validator deposit. This counter is used by the consensus layer to:

* Track the total number of deposits
* Verify deposit indices
* Coordinate deposit processing

This method queries the current count.

***

## Signatures

<table><thead><tr><th width="200">Sync</th><th>Async</th></tr></thead><tbody><tr><td><pre class="language-python"><code class="lang-python">def get_deposit_count(
    *,
    address: str = DEPOSIT_CONTRACT_ADDRESS,
) -> int
</code></pre></td><td><pre class="language-python"><code class="lang-python">async def get_deposit_count(
    *,
    address: str = DEPOSIT_CONTRACT_ADDRESS,
) -> int
</code></pre></td></tr></tbody></table>

***

## Parameters

| Parameter | Type  | Default                                                      | Description                              |
| --------- | ----- | ------------------------------------------------------------ | ---------------------------------------- |
| `address` | `str` | [`DEPOSIT_CONTRACT_ADDRESS`](../../abis/deposit-contract.md) | Address of the deposit contract to query |

The default address is the genesis deposit contract ([`DEPOSIT_CONTRACT_ADDRESS`](../../abis/deposit-contract.md)).

***

## Returns

**Type:** `int`

The total number of deposits as a Python integer.

```python
# Sync
count = w3.seismic.get_deposit_count()
assert isinstance(count, int)

# Async
count = await w3.seismic.get_deposit_count()
assert isinstance(count, int)
```

***

## Examples

### Sync Usage

```python
from seismic_web3 import create_public_client

# Create public client
w3 = create_public_client("https://gcp-1.seismictest.net/rpc")

# Get deposit count
count = w3.seismic.get_deposit_count()

print(f"Total deposits: {count}")
print(f"Type: {type(count)}")
```

### Async Usage

```python
from seismic_web3 import create_async_public_client

# Create async public client
w3 = await create_async_public_client("https://gcp-1.seismictest.net/rpc")

# Get deposit count
count = await w3.seismic.get_deposit_count()

print(f"Total deposits: {count}")
```

### With Wallet Client

```python
from seismic_web3 import create_wallet_client, PrivateKey

# Wallet clients also have access to public methods
w3 = create_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=PrivateKey(b"..."),
)

# Get deposit count
count = w3.seismic.get_deposit_count()
print(f"Total deposits: {count}")
```

### Custom Contract Address

```python
# Query a different deposit contract
count = w3.seismic.get_deposit_count(
    address="0x1234567890123456789012345678901234567890",
)

print(f"Custom contract deposits: {count}")
```

### Monitoring New Deposits

```python
import time

# Monitor for new deposits
previous_count = w3.seismic.get_deposit_count()
print(f"Starting count: {previous_count}")

while True:
    current_count = w3.seismic.get_deposit_count()

    if current_count > previous_count:
        new_deposits = current_count - previous_count
        print(f"New deposits detected: {new_deposits}")
        print(f"Total deposits: {current_count}")
        previous_count = current_count

    time.sleep(12)  # Check every block (~12 seconds)
```

### Verifying Deposit

```python
# Check count before and after deposit
before_count = w3.seismic.get_deposit_count()
print(f"Deposits before: {before_count}")

# Make deposit
tx_hash = w3.seismic.deposit(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    deposit_data_root=deposit_data_root,
    value=32 * 10**18,
)

# Wait for transaction
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Verify count increased
after_count = w3.seismic.get_deposit_count()
print(f"Deposits after: {after_count}")
assert after_count == before_count + 1
```

### With Deposit Root

```python
# Get both count and root
count = w3.seismic.get_deposit_count()
root = w3.seismic.get_deposit_root()

print(f"Total deposits: {count}")
print(f"Deposit root: {root.hex()}")
print(f"Estimated total ETH staked: {count * 32} ETH")
```

***

## Implementation Details

### Contract Call

This method calls the deposit contract's `get_deposit_count()` function:

```solidity
function get_deposit_count() external view returns (bytes memory);
```

The contract returns an 8-byte little-endian encoded integer. The SDK:

1. Encodes the function call using the deposit contract ABI
2. Calls `eth_call` to the deposit contract
3. Extracts bytes 64-72 from the response (the 8-byte count)
4. Decodes as a little-endian integer
5. Returns as a Python `int`

### Encoding Details

The on-chain value is stored as:

* **Format**: 8-byte little-endian integer
* **Location**: Bytes 64-72 of the contract response
* **Range**: 0 to 2^64 - 1 (18,446,744,073,709,551,615)

The SDK handles the conversion automatically.

### Genesis Contract

The default deposit contract address ([`DEPOSIT_CONTRACT_ADDRESS`](../../abis/deposit-contract.md)) is deployed at genesis on all Seismic networks.

***

## Use Cases

### Network Statistics

Track network growth by monitoring deposits:

```python
# Get current network stats
count = w3.seismic.get_deposit_count()
total_staked = count * 32  # Assuming 32 ETH per deposit

print(f"Active validators: {count}")
print(f"Total ETH staked: {total_staked:,} ETH")
print(f"Total value (USD): ${total_staked * eth_price:,.2f}")
```

### Validator Onboarding

Check network size before deciding to become a validator:

```python
# Check current validator count
count = w3.seismic.get_deposit_count()

print(f"Current validators: {count}")

if count < 1000:
    print("Early network - lower competition")
elif count < 10000:
    print("Growing network - moderate competition")
else:
    print("Mature network - high competition")
```

### Deposit Verification

Verify your deposit was included:

```python
# Record count before deposit
before_count = w3.seismic.get_deposit_count()

# Make deposit
tx_hash = w3.seismic.deposit(...)

# Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Verify inclusion
after_count = w3.seismic.get_deposit_count()

if after_count == before_count + 1:
    print("Deposit successful!")
    print(f"Your deposit index: {after_count - 1}")
else:
    print("WARNING: Deposit count mismatch!")
```

### Rate Limiting

Implement rate limiting for deposit processing:

```python
import time

# Process deposits at most once per minute
last_count = w3.seismic.get_deposit_count()
last_check = time.time()

while True:
    now = time.time()

    if now - last_check >= 60:  # Check every minute
        current_count = w3.seismic.get_deposit_count()

        if current_count > last_count:
            # Process new deposits
            process_deposits(last_count, current_count)
            last_count = current_count

        last_check = now

    time.sleep(1)
```

***

## Notes

### Public Method

`get_deposit_count()` is available on both:

* **Public clients** (`create_public_client`) — Read-only, no private key
* **Wallet clients** (`create_wallet_client`) — Full capabilities

### Read-Only

This is a `view` function that does not modify state. It does not consume gas or require a transaction.

### Monotonic Counter

The deposit count is monotonically increasing — it only goes up, never down. Each deposit increments the count by exactly 1.

### Deposit Index

The deposit count also represents the next deposit index (0-indexed):

* First deposit: index 0, count becomes 1
* Second deposit: index 1, count becomes 2
* Third deposit: index 2, count becomes 3

***

## Error Handling

```python
try:
    count = w3.seismic.get_deposit_count()
    print(f"Total deposits: {count}")
except Exception as e:
    print(f"Failed to fetch deposit count: {e}")
    # Possible causes:
    # - Node is offline
    # - Invalid contract address
    # - RPC error
```

***

## Performance

### Caching

If you're querying the deposit count frequently, consider caching:

```python
import time

class DepositCounter:
    def __init__(self, w3, cache_seconds=12):
        self.w3 = w3
        self.cache_seconds = cache_seconds
        self._count = None
        self._last_fetch = 0

    def get_count(self):
        now = time.time()

        if now - self._last_fetch > self.cache_seconds:
            self._count = self.w3.seismic.get_deposit_count()
            self._last_fetch = now

        return self._count

# Use cached counter
counter = DepositCounter(w3)
count = counter.get_count()  # Fetches from chain
count = counter.get_count()  # Returns cached value (if within 12 seconds)
```

***

## See Also

* [get\_deposit\_root()](get-deposit-root.md) — Read the deposit Merkle root
* [deposit()](deposit.md) — Submit a validator deposit
* [Deposit Contract ABI](../../../../gitbook/client-libraries/seismic-python/api-reference/abis/deposit-contract.md) — Full contract interface
* [Validator Guide](../../../../gitbook/client-libraries/seismic-python/guides/validator-deposits.md) — Guide to making deposits
