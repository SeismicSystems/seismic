---
description: Generate ETH1 withdrawal credentials for validator deposits
icon: key
---

# make_withdrawal_credentials

Build 32-byte ETH1 withdrawal credentials from an Ethereum address.

## Overview

`make_withdrawal_credentials()` generates the 32-byte withdrawal credentials required for validator deposits. These credentials determine which Ethereum address can withdraw a validator's funds when they exit the validator set.

The function creates ETH1-style withdrawal credentials with the format: `0x01` + 11 zero bytes + 20-byte Ethereum address.

## Signature

```python
def make_withdrawal_credentials(address: str) -> bytes
```

## Import

```python
from seismic_web3 import make_withdrawal_credentials
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `address` | `str` | Yes | Hex-encoded Ethereum address (with or without `0x` prefix) |

## Returns

| Type | Description |
|------|-------------|
| `bytes` | 32-byte withdrawal credentials in ETH1 format |

## Examples

### Basic Usage

```python
from seismic_web3 import make_withdrawal_credentials

# Generate withdrawal credentials from address
address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
withdrawal_credentials = make_withdrawal_credentials(address)

print(f"Withdrawal credentials: {withdrawal_credentials.hex()}")
# Output: 010000000000000000000000742d35cc6634c0532925a3b844bc9e7595f0beb0
```

### With and Without 0x Prefix

```python
from seismic_web3 import make_withdrawal_credentials

# Both formats work
address_with_prefix = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
address_without_prefix = "742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"

creds1 = make_withdrawal_credentials(address_with_prefix)
creds2 = make_withdrawal_credentials(address_without_prefix)

assert creds1 == creds2  # Same result
```

### Complete Deposit Flow

```python
from seismic_web3 import (
    create_wallet_client,
    DEPOSIT_CONTRACT_ABI,
    DEPOSIT_CONTRACT_ADDRESS,
    make_withdrawal_credentials,
    compute_deposit_data_root,
)

# Create client
w3 = create_wallet_client(
    rpc_url="https://sepolia.seismic.foundation",
    private_key="0x...",
)

# Use your wallet address for withdrawals
withdrawal_address = w3.eth.default_account
withdrawal_credentials = make_withdrawal_credentials(withdrawal_address)

# Prepare keys and signatures
node_pubkey = bytes.fromhex("...")  # 32 bytes
consensus_pubkey = bytes.fromhex("...")  # 48 bytes
node_signature = bytes.fromhex("...")  # 64 bytes
consensus_signature = bytes.fromhex("...")  # 96 bytes

# Compute deposit data root
deposit_data_root = compute_deposit_data_root(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,  # Generated credentials
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    amount_gwei=32_000_000_000,
)

# Make deposit
deposit_contract = w3.eth.contract(
    address=DEPOSIT_CONTRACT_ADDRESS,
    abi=DEPOSIT_CONTRACT_ABI,
)

tx_hash = deposit_contract.functions.deposit(
    node_pubkey,
    consensus_pubkey,
    withdrawal_credentials,
    node_signature,
    consensus_signature,
    deposit_data_root,
).transact({'value': 32 * 10**18})

print(f"Deposit tx: {tx_hash.hex()}")
```

### Using Separate Withdrawal Address

```python
from seismic_web3 import make_withdrawal_credentials

# Use different address for withdrawals (e.g., cold wallet)
deposit_address = w3.eth.default_account  # Hot wallet for deposit
withdrawal_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"  # Cold wallet

# Generate credentials for cold wallet
withdrawal_credentials = make_withdrawal_credentials(withdrawal_address)

# Deposit from hot wallet, withdrawals go to cold wallet
# ... proceed with deposit ...
```

### Checksum Addresses

```python
from seismic_web3 import make_withdrawal_credentials
from web3 import Web3

# Convert to checksum address (EIP-55)
address = "0x742d35cc6634c0532925a3b844bc9e7595f0beb0"  # lowercase
checksum_address = Web3.to_checksum_address(address)

withdrawal_credentials = make_withdrawal_credentials(checksum_address)

# Note: Case doesn't matter for withdrawal credentials
# Both lowercase and checksum produce same result
```

### Verifying Credentials Format

```python
from seismic_web3 import make_withdrawal_credentials

address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
credentials = make_withdrawal_credentials(address)

# Verify format
assert len(credentials) == 32, "Must be 32 bytes"
assert credentials[0] == 0x01, "First byte must be 0x01 (ETH1 type)"
assert credentials[1:12] == b'\x00' * 11, "Bytes 1-11 must be zero"
assert credentials[12:] == bytes.fromhex(address[2:]), "Last 20 bytes are address"

print("Credentials format verified!")
```

## How It Works

The function constructs withdrawal credentials using the ETH1 format:

```
+-------+----------------+------------------------+
| 0x01  | 11 zero bytes  | 20-byte address        |
+-------+----------------+------------------------+
 1 byte   11 bytes         20 bytes
                           = 32 bytes total
```

**Breakdown**:
1. **Type byte** (`0x01`): Indicates ETH1-style withdrawal credentials
2. **Padding** (11 bytes of `0x00`): Reserved space
3. **Address** (20 bytes): Ethereum address that can withdraw funds

**Example**:
```python
address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"

credentials = make_withdrawal_credentials(address)
# Result: 0x010000000000000000000000742d35cc6634c0532925a3b844bc9e7595f0beb0
#         ^^ type
#           ^^^^^^^^^^^^^^^^^^^^^^ padding (11 bytes)
#                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ address (20 bytes)
```

## Credential Types

Ethereum supports two types of withdrawal credentials:

### ETH1 Withdrawal (0x01)

- **Type byte**: `0x01`
- **Format**: `0x01` + 11 zero bytes + 20-byte address
- **Created by**: `make_withdrawal_credentials()`
- **Withdrawals**: Direct to Ethereum address
- **Best for**: Most validators (simple and direct)

### BLS Withdrawal (0x00)

- **Type byte**: `0x00`
- **Format**: `0x00` + 31-byte BLS pubkey hash
- **Created by**: Custom (not provided by this function)
- **Withdrawals**: Requires BLS signature
- **Best for**: Advanced setups (e.g., multi-sig validators)

**This function only creates ETH1 credentials (0x01).**

## Error Handling

```python
from seismic_web3 import make_withdrawal_credentials

try:
    credentials = make_withdrawal_credentials("0x742d...")
    print(f"Credentials: {credentials.hex()}")

except ValueError as e:
    print(f"Invalid address: {e}")
    # Error: "address must be 20 bytes, got X"
```

### Common Errors

```python
# Too short
address = "0x742d35"  # Only 3 bytes
credentials = make_withdrawal_credentials(address)
# ValueError: address must be 20 bytes, got 3

# Too long
address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0FFFF"  # 22 bytes
credentials = make_withdrawal_credentials(address)
# ValueError: address must be 20 bytes, got 22

# Invalid hex
address = "0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG"
credentials = make_withdrawal_credentials(address)
# ValueError: non-hexadecimal number found in fromhex() arg
```

## Validation

The function validates:
- Address must decode to exactly **20 bytes**
- Hex string must be valid (0-9, a-f, A-F)
- Length check after removing `0x` prefix

The function does NOT validate:
- Checksum correctness (EIP-55)
- Whether address exists on-chain
- Whether address is a contract or EOA

## Security Considerations

### Choose Withdrawal Address Carefully

The withdrawal address **controls validator withdrawals**:

```python
# DO NOT use temporary or test addresses
withdrawal_address = "0x0000000000000000000000000000000000000000"  # BAD!

# DO use a secure address you control
withdrawal_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"  # Good

withdrawal_credentials = make_withdrawal_credentials(withdrawal_address)
```

**Once set, withdrawal credentials typically cannot be changed.**

### Cold Wallet Recommended

Use a secure cold wallet address for withdrawals:

```python
# Hot wallet for deposits (needs gas and signing)
deposit_wallet = w3.eth.default_account

# Cold wallet for withdrawals (secure storage)
cold_wallet = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"

# Use cold wallet for withdrawal credentials
withdrawal_credentials = make_withdrawal_credentials(cold_wallet)
```

### Multi-Sig Considerations

Consider using a multi-sig wallet for high-value validators:

```python
# Gnosis Safe or other multi-sig address
multisig_address = "0x1234567890123456789012345678901234567890"

withdrawal_credentials = make_withdrawal_credentials(multisig_address)

# Withdrawals will go to multi-sig, requiring multiple signatures
```

### Verify Before Depositing

Always double-check withdrawal credentials before depositing:

```python
from seismic_web3 import make_withdrawal_credentials

withdrawal_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
credentials = make_withdrawal_credentials(withdrawal_address)

# Verify the address is embedded correctly
extracted_address = "0x" + credentials[12:].hex()

if extracted_address.lower() == withdrawal_address.lower():
    print(f"✓ Withdrawal address verified: {withdrawal_address}")
else:
    print(f"✗ ERROR: Address mismatch!")
    print(f"  Expected: {withdrawal_address}")
    print(f"  Got: {extracted_address}")
```

## Common Use Cases

### Standard Validator Setup

```python
from seismic_web3 import make_withdrawal_credentials

# Use validator operator's address
validator_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
withdrawal_credentials = make_withdrawal_credentials(validator_address)
```

### Staking Service

```python
from seismic_web3 import make_withdrawal_credentials

# User provides withdrawal address
user_withdrawal_address = "0x1234567890123456789012345678901234567890"

# Staking service generates credentials for user
withdrawal_credentials = make_withdrawal_credentials(user_withdrawal_address)

# Service handles deposit, but withdrawals go to user
```

### Validator Pool

```python
from seismic_web3 import make_withdrawal_credentials

# Pool contract address receives all withdrawals
pool_contract = "0x9876543210987654321098765432109876543210"

withdrawal_credentials = make_withdrawal_credentials(pool_contract)

# Pool contract distributes to participants
```

## Alternative: BLS Withdrawal Credentials

If you need BLS-style credentials (type `0x00`), implement manually:

```python
import hashlib

def make_bls_withdrawal_credentials(bls_pubkey: bytes) -> bytes:
    """Generate BLS withdrawal credentials (0x00 type)."""
    if len(bls_pubkey) != 48:
        raise ValueError(f"BLS pubkey must be 48 bytes, got {len(bls_pubkey)}")

    # SHA-256 hash of pubkey
    pubkey_hash = hashlib.sha256(bls_pubkey).digest()

    # 0x00 + first 31 bytes of hash
    return b'\x00' + pubkey_hash[:31]

# Usage
bls_pubkey = bytes.fromhex("...")  # 48-byte BLS pubkey
bls_credentials = make_bls_withdrawal_credentials(bls_pubkey)
```

## When to Use

Use `make_withdrawal_credentials()` when:
- Making a validator deposit
- Generating ETH1 withdrawal credentials
- Building validator onboarding tools
- Implementing staking services
- Creating deposit data for testing

## Related Functions

### Deposit Flow

```python
from seismic_web3 import (
    make_withdrawal_credentials,
    compute_deposit_data_root,
)

# 1. Generate withdrawal credentials
withdrawal_credentials = make_withdrawal_credentials(address)

# 2. Compute deposit data root
deposit_data_root = compute_deposit_data_root(
    node_pubkey=...,
    consensus_pubkey=...,
    withdrawal_credentials=withdrawal_credentials,  # From step 1
    node_signature=...,
    consensus_signature=...,
    amount_gwei=...,
)

# 3. Make deposit
deposit_contract.functions.deposit(
    ...,
    withdrawal_credentials,  # Pass credentials
    ...,
    deposit_data_root,       # Pass computed root
).transact({'value': ...})
```

## See Also

- [compute_deposit_data_root()](compute-deposit-data-root.md) - Compute deposit data root
- [DEPOSIT_CONTRACT_ABI](deposit-contract.md) - Deposit contract ABI and address
- [Deposit Contract Functions](deposit-contract.md#usage-examples) - Making deposits
- [Client Documentation](../client/) - Client creation
