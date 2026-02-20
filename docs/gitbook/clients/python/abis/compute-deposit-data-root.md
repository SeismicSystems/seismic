---
description: Compute deposit data root hash for validator deposits
icon: calculator
---

# compute_deposit_data_root

Compute the deposit data root hash (SHA-256 SSZ hash tree root) for validator deposits.

## Overview

`compute_deposit_data_root()` computes the 32-byte deposit data root hash that must be passed to the deposit contract's `deposit()` function. This hash is computed using SSZ (Simple Serialize) hash tree root over the deposit data structure and mirrors the on-chain verification logic in `DepositContract.sol`.

The function validates all input lengths and raises `ValueError` if any parameter has the wrong byte length.

## Signature

```python
def compute_deposit_data_root(
    *,
    node_pubkey: bytes,
    consensus_pubkey: bytes,
    withdrawal_credentials: bytes,
    node_signature: bytes,
    consensus_signature: bytes,
    amount_gwei: int,
) -> bytes
```

## Import

```python
from seismic_web3 import compute_deposit_data_root
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `node_pubkey` | `bytes` | Yes | 32-byte ED25519 public key for node identity |
| `consensus_pubkey` | `bytes` | Yes | 48-byte BLS12-381 public key for consensus |
| `withdrawal_credentials` | `bytes` | Yes | 32-byte withdrawal credentials (use `make_withdrawal_credentials()`) |
| `node_signature` | `bytes` | Yes | 64-byte ED25519 signature over deposit data |
| `consensus_signature` | `bytes` | Yes | 96-byte BLS12-381 signature over deposit data |
| `amount_gwei` | `int` | Yes | Deposit amount in gwei (e.g., `32_000_000_000` for 32 ETH) |

**Note**: All parameters are keyword-only. You must use `name=value` syntax when calling.

## Returns

| Type | Description |
|------|-------------|
| `bytes` | 32-byte deposit data root hash (SHA-256 SSZ hash tree root) |

## Examples

### Basic Usage

```python
from seismic_web3 import (
    compute_deposit_data_root,
    make_withdrawal_credentials,
)

# Prepare deposit data
node_pubkey = bytes.fromhex(
    "a1b2c3d4e5f6..."  # 32-byte ED25519 pubkey
)
consensus_pubkey = bytes.fromhex(
    "1a2b3c4d5e6f..."  # 48-byte BLS12-381 pubkey
)
withdrawal_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
withdrawal_credentials = make_withdrawal_credentials(withdrawal_address)

# Signatures from signing process
node_signature = bytes.fromhex("...")  # 64 bytes
consensus_signature = bytes.fromhex("...")  # 96 bytes

# Compute root for 32 ETH deposit
root = compute_deposit_data_root(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    amount_gwei=32_000_000_000,  # 32 ETH in gwei
)

print(f"Deposit data root: {root.hex()}")
```

### Complete Deposit Flow

```python
from seismic_web3 import (
    create_wallet_client,
    DEPOSIT_CONTRACT_ABI,
    DEPOSIT_CONTRACT_ADDRESS,
    compute_deposit_data_root,
    make_withdrawal_credentials,
)

# Create client
w3 = create_wallet_client(
    rpc_url="https://sepolia.seismic.foundation",
    private_key="0x...",
)

# Prepare keys
node_pubkey = bytes.fromhex("...")  # 32 bytes
consensus_pubkey = bytes.fromhex("...")  # 48 bytes

# Generate withdrawal credentials
withdrawal_address = w3.eth.default_account  # Use your address
withdrawal_credentials = make_withdrawal_credentials(withdrawal_address)

# Sign deposit data (external process)
node_signature = bytes.fromhex("...")  # 64 bytes
consensus_signature = bytes.fromhex("...")  # 96 bytes

# Compute deposit data root
amount_gwei = 32_000_000_000  # 32 ETH
deposit_data_root = compute_deposit_data_root(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    amount_gwei=amount_gwei,
)

# Create deposit contract instance
deposit_contract = w3.eth.contract(
    address=DEPOSIT_CONTRACT_ADDRESS,
    abi=DEPOSIT_CONTRACT_ABI,
)

# Make deposit
tx_hash = deposit_contract.functions.deposit(
    node_pubkey,
    consensus_pubkey,
    withdrawal_credentials,
    node_signature,
    consensus_signature,
    deposit_data_root,  # Computed root
).transact({
    'value': 32 * 10**18,  # 32 ETH in wei
    'gas': 500_000,
})

print(f"Deposit tx: {tx_hash.hex()}")
```

### Varying Deposit Amounts

```python
from seismic_web3 import compute_deposit_data_root

# 1 ETH deposit (minimum)
root_1eth = compute_deposit_data_root(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    amount_gwei=1_000_000_000,  # 1 ETH
)

# 32 ETH deposit (standard)
root_32eth = compute_deposit_data_root(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    amount_gwei=32_000_000_000,  # 32 ETH
)

# Note: Signatures must match the amount being deposited
```

### Verifying Deposit Data

```python
from seismic_web3 import compute_deposit_data_root

# Compute root from deposit data
computed_root = compute_deposit_data_root(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    amount_gwei=amount_gwei,
)

# Compare with expected root
expected_root = bytes.fromhex("...")

if computed_root == expected_root:
    print("Deposit data verified!")
else:
    print("WARNING: Deposit data mismatch!")
    print(f"Expected: {expected_root.hex()}")
    print(f"Computed: {computed_root.hex()}")
```

## How It Works

The function computes the SSZ hash tree root using the following structure:

1. **Public Key Root**:
   ```
   pubkey_root = SHA256(
       node_pubkey + SHA256(consensus_pubkey + padding)
   )
   ```

2. **Signature Root**:
   ```
   signature_root = SHA256(
       SHA256(node_signature) +
       SHA256(SHA256(consensus_sig[0:64]) + SHA256(consensus_sig[64:96] + padding))
   )
   ```

3. **Final Root**:
   ```
   root = SHA256(
       SHA256(pubkey_root + withdrawal_credentials) +
       SHA256(amount + padding + signature_root)
   )
   ```

Where:
- `amount` is encoded as 8-byte little-endian gwei value
- `padding` extends data to chunk boundaries (16/32/64 bytes)
- All hashes are SHA-256

## Parameter Requirements

### node_pubkey (32 bytes)

ED25519 public key for node identity:
```python
# Generate with cryptography library
from cryptography.hazmat.primitives.asymmetric import ed25519

private_key = ed25519.Ed25519PrivateKey.generate()
node_pubkey = private_key.public_key().public_bytes(
    encoding=serialization.Encoding.Raw,
    format=serialization.PublicFormat.Raw,
)
```

### consensus_pubkey (48 bytes)

BLS12-381 public key for consensus:
```python
# Generate with py_ecc library
from py_ecc.bls import G2ProofOfPossession as bls

private_key = int.from_bytes(secrets.token_bytes(32), byteorder="big")
consensus_pubkey = bls.SkToPk(private_key)  # Returns 48 bytes
```

### withdrawal_credentials (32 bytes)

ETH1-style withdrawal credentials:
```python
from seismic_web3 import make_withdrawal_credentials

withdrawal_credentials = make_withdrawal_credentials(
    "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"
)
```

Format: `0x01` + 11 zero bytes + 20-byte address

### node_signature (64 bytes)

ED25519 signature over deposit message:
```python
from cryptography.hazmat.primitives.asymmetric import ed25519

# Sign deposit message
deposit_message = ...  # Construct deposit message
signature = private_key.sign(deposit_message)  # 64 bytes
```

### consensus_signature (96 bytes)

BLS12-381 signature over deposit message:
```python
from py_ecc.bls import G2ProofOfPossession as bls

# Sign deposit message
deposit_message_hash = ...  # Hash of deposit message
signature = bls.Sign(private_key, deposit_message_hash)  # Returns 96 bytes
```

### amount_gwei (integer)

Deposit amount in gwei (10⁹ wei):
```python
# 1 ETH = 1,000,000,000 gwei
amount_gwei = 1_000_000_000

# 32 ETH = 32,000,000,000 gwei
amount_gwei = 32_000_000_000

# Convert from ETH to gwei
eth_amount = 32.0
amount_gwei = int(eth_amount * 10**9)

# Convert from wei to gwei
wei_amount = 32 * 10**18
amount_gwei = wei_amount // 10**9
```

## Error Handling

```python
from seismic_web3 import compute_deposit_data_root

try:
    root = compute_deposit_data_root(
        node_pubkey=node_pubkey,
        consensus_pubkey=consensus_pubkey,
        withdrawal_credentials=withdrawal_credentials,
        node_signature=node_signature,
        consensus_signature=consensus_signature,
        amount_gwei=amount_gwei,
    )
    print(f"Root: {root.hex()}")

except ValueError as e:
    print(f"Invalid deposit data: {e}")
    # Common errors:
    # - "node_pubkey must be 32 bytes, got 33"
    # - "consensus_pubkey must be 48 bytes, got 96"
    # - "withdrawal_credentials must be 32 bytes, got 20"
    # - etc.
```

## Validation

The function validates all input lengths:

| Parameter | Expected Length | Error if Wrong |
|-----------|----------------|----------------|
| `node_pubkey` | 32 bytes | `ValueError` |
| `consensus_pubkey` | 48 bytes | `ValueError` |
| `withdrawal_credentials` | 32 bytes | `ValueError` |
| `node_signature` | 64 bytes | `ValueError` |
| `consensus_signature` | 96 bytes | `ValueError` |
| `amount_gwei` | Any `int` | No validation |

## Security Considerations

### Signature Verification

- Signatures must be valid for the deposit data
- On-chain contract will verify signatures match pubkeys
- Invalid signatures will cause deposit to revert

### Amount Consistency

- `amount_gwei` in root computation must match transaction value
- Contract receives value in **wei** (not gwei)
- Convert correctly: `value_wei = amount_gwei * 10**9`

### Root Uniqueness

- Different deposit data produces different roots
- Changing any parameter changes the root
- Root acts as commitment to all deposit parameters

### Key Reuse

- Never reuse validator keys across networks
- Each network should have unique keys
- Reusing keys can lead to slashing

## Common Pitfalls

### Wrong Unit Conversion

```python
# Wrong: passing wei instead of gwei
amount_gwei = 32 * 10**18  # This is wei, not gwei!

# Correct: use gwei
amount_gwei = 32 * 10**9   # 32 ETH in gwei
```

### Mismatched Transaction Value

```python
# Wrong: root uses gwei, transaction uses different value
root = compute_deposit_data_root(..., amount_gwei=32_000_000_000)
tx = deposit_contract.functions.deposit(...).transact({
    'value': 1 * 10**18,  # Only 1 ETH! Mismatch!
})

# Correct: match the amounts
amount_eth = 32
amount_gwei = amount_eth * 10**9
root = compute_deposit_data_root(..., amount_gwei=amount_gwei)
tx = deposit_contract.functions.deposit(...).transact({
    'value': amount_eth * 10**18,
})
```

### Wrong Byte Length

```python
# Wrong: passing hex string instead of bytes
node_pubkey = "a1b2c3d4..."  # String, not bytes!

# Correct: convert to bytes
node_pubkey = bytes.fromhex("a1b2c3d4...")
```

## When to Use

Use `compute_deposit_data_root()` when:
- Making a validator deposit
- Verifying deposit data before submitting
- Building validator onboarding tools
- Testing deposit contract interactions
- Implementing custom deposit flows

## See Also

- [make_withdrawal_credentials()](make-withdrawal-credentials.md) - Generate withdrawal credentials
- [DEPOSIT_CONTRACT_ABI](deposit-contract.md) - Deposit contract ABI and address
- [Deposit Contract Functions](deposit-contract.md#usage-examples) - Making deposits
- [Client Documentation](../client/) - Client creation
