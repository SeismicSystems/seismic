---
description: Validator deposit contract ABI and address constants
icon: vault
---

# Deposit Contract

ABI and address constants for Seismic's Eth2-style validator deposit contract.

## Overview

The deposit contract is a genesis contract that manages validator deposits for the Seismic network. Validators deposit ETH (minimum 1 ETH, typically 32 ETH) along with their public keys, signatures, and withdrawal credentials to join the validator set.

The contract maintains a Merkle tree of deposits and emits a `DepositEvent` for each successful deposit.

## Constants

### DEPOSIT_CONTRACT_ADDRESS

The canonical deposit contract address, deployed at a fixed genesis address on all Seismic networks.

```python
from seismic_web3 import DEPOSIT_CONTRACT_ADDRESS

# Genesis address matching Ethereum's deposit contract
DEPOSIT_CONTRACT_ADDRESS: str = "0x00000000219ab540356cBB839Cbe05303d7705Fa"
```

### DEPOSIT_CONTRACT_ABI

Complete ABI for the `IDepositContract` interface.

```python
from seismic_web3 import DEPOSIT_CONTRACT_ABI

DEPOSIT_CONTRACT_ABI: list[dict[str, Any]]
```

## Import

```python
from seismic_web3 import (
    DEPOSIT_CONTRACT_ABI,
    DEPOSIT_CONTRACT_ADDRESS,
)
```

## ABI Contents

### Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `deposit()` | `node_pubkey`, `consensus_pubkey`, `withdrawal_credentials`, `node_signature`, `consensus_signature`, `deposit_data_root` | None | Deposit ETH to become a validator (payable) |
| `get_deposit_count()` | None | `bytes` | Get total number of deposits (view) |
| `get_deposit_root()` | None | `bytes32` | Get Merkle root of all deposits (view) |
| `supportsInterface()` | `interfaceId` | `bool` | Check ERC-165 interface support (pure) |

### Events

| Event | Parameters | Description |
|-------|-----------|-------------|
| `DepositEvent` | `node_pubkey`, `consensus_pubkey`, `withdrawal_credentials`, `amount`, `node_signature`, `consensus_signature`, `index` | Emitted on successful deposit |

## Full ABI

```python
DEPOSIT_CONTRACT_ABI: list[dict[str, Any]] = [
    {
        "type": "function",
        "name": "deposit",
        "inputs": [
            {"name": "node_pubkey", "type": "bytes", "internalType": "bytes"},
            {"name": "consensus_pubkey", "type": "bytes", "internalType": "bytes"},
            {
                "name": "withdrawal_credentials",
                "type": "bytes",
                "internalType": "bytes",
            },
            {"name": "node_signature", "type": "bytes", "internalType": "bytes"},
            {"name": "consensus_signature", "type": "bytes", "internalType": "bytes"},
            {
                "name": "deposit_data_root",
                "type": "bytes32",
                "internalType": "bytes32",
            },
        ],
        "outputs": [],
        "stateMutability": "payable",
    },
    {
        "type": "function",
        "name": "get_deposit_count",
        "inputs": [],
        "outputs": [{"name": "", "type": "bytes", "internalType": "bytes"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "get_deposit_root",
        "inputs": [],
        "outputs": [{"name": "", "type": "bytes32", "internalType": "bytes32"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "supportsInterface",
        "inputs": [
            {"name": "interfaceId", "type": "bytes4", "internalType": "bytes4"},
        ],
        "outputs": [{"name": "", "type": "bool", "internalType": "bool"}],
        "stateMutability": "pure",
    },
    {
        "type": "event",
        "name": "DepositEvent",
        "inputs": [
            {
                "name": "node_pubkey",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
            {
                "name": "consensus_pubkey",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
            {
                "name": "withdrawal_credentials",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
            {
                "name": "amount",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
            {
                "name": "node_signature",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
            {
                "name": "consensus_signature",
                "type": "bytes",
                "indexed": False,
                "internalType": "bytes",
            },
            {
                "name": "index",
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
from seismic_web3 import (
    create_wallet_client,
    DEPOSIT_CONTRACT_ABI,
    DEPOSIT_CONTRACT_ADDRESS,
)

# Create client
w3 = create_wallet_client(
    rpc_url="https://gcp-1.seismictest.net/rpc",
    private_key="0x...",
)

# Create deposit contract instance
deposit_contract = w3.eth.contract(
    address=DEPOSIT_CONTRACT_ADDRESS,
    abi=DEPOSIT_CONTRACT_ABI,
)
```

### Query Deposit Count

```python
# Get total number of deposits
count_bytes = deposit_contract.functions.get_deposit_count().call()

# Decode as little-endian 8-byte integer
count = int.from_bytes(count_bytes[:8], byteorder="little")
print(f"Total deposits: {count}")
```

### Query Deposit Root

```python
# Get current Merkle root
root = deposit_contract.functions.get_deposit_root().call()
print(f"Deposit root: {root.hex()}")
```

### Making a Validator Deposit

```python
from seismic_web3 import (
    make_withdrawal_credentials,
    compute_deposit_data_root,
)

# Prepare deposit data
node_pubkey = bytes.fromhex("...")  # 32-byte ED25519 pubkey
consensus_pubkey = bytes.fromhex("...")  # 48-byte BLS12-381 pubkey
withdrawal_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"

# Generate withdrawal credentials
withdrawal_credentials = make_withdrawal_credentials(withdrawal_address)

# Sign deposit data (external signing process)
node_signature = bytes.fromhex("...")  # 64-byte ED25519 signature
consensus_signature = bytes.fromhex("...")  # 96-byte BLS12-381 signature

# Deposit amount (32 ETH in gwei)
amount_gwei = 32_000_000_000

# Compute deposit data root
deposit_data_root = compute_deposit_data_root(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    amount_gwei=amount_gwei,
)

# Make deposit (send 32 ETH)
tx_hash = deposit_contract.functions.deposit(
    node_pubkey,
    consensus_pubkey,
    withdrawal_credentials,
    node_signature,
    consensus_signature,
    deposit_data_root,
).transact({
    'value': 32 * 10**18,  # 32 ETH in wei
    'gas': 500_000,
})

print(f"Deposit tx: {tx_hash.hex()}")
```

### Listening to Deposit Events

```python
# Get recent deposit events
events = deposit_contract.events.DepositEvent.get_logs(
    from_block="latest" - 10000,
)

for event in events:
    node_pub = event.args['node_pubkey'].hex()
    consensus_pub = event.args['consensus_pubkey'].hex()
    amount_bytes = event.args['amount']
    index_bytes = event.args['index']

    # Decode amount (little-endian 8-byte gwei)
    amount_gwei = int.from_bytes(amount_bytes[:8], byteorder="little")
    amount_eth = amount_gwei / 10**9

    # Decode index (little-endian 8-byte integer)
    index = int.from_bytes(index_bytes[:8], byteorder="little")

    print(f"Deposit #{index}: {amount_eth} ETH")
    print(f"  Node pubkey: {node_pub[:16]}...")
    print(f"  Consensus pubkey: {consensus_pub[:16]}...")
```

### Check Interface Support

```python
# Check if contract supports ERC-165
interface_id = bytes.fromhex("01ffc9a7")  # ERC-165 interface ID
supported = deposit_contract.functions.supportsInterface(interface_id).call()
print(f"Supports ERC-165: {supported}")
```

## Deposit Requirements

### Minimum Deposit

- Minimum: 1 ETH
- Standard validator deposit: 32 ETH
- Amount must be specified in both gwei (for root computation) and wei (for transaction value)

### Required Keys

1. **Node Public Key** (32 bytes)
   - ED25519 public key for node identity
   - Used for node-to-node communication

2. **Consensus Public Key** (48 bytes)
   - BLS12-381 public key for consensus participation
   - Used for block signing and attestations

### Required Signatures

1. **Node Signature** (64 bytes)
   - ED25519 signature over deposit data
   - Proves ownership of node private key

2. **Consensus Signature** (96 bytes)
   - BLS12-381 signature over deposit data
   - Proves ownership of consensus private key

### Withdrawal Credentials

32-byte ETH1-style withdrawal credentials:
- Format: `0x01` + 11 zero bytes + 20-byte Ethereum address
- Use `make_withdrawal_credentials()` helper to generate

### Deposit Data Root

32-byte SHA-256 hash of the deposit data:
- Computed using SSZ hash tree root
- Use `compute_deposit_data_root()` helper to compute
- Must match on-chain verification

## Deposit Process

1. Generate ED25519 and BLS12-381 key pairs
2. Create withdrawal credentials from Ethereum address
3. Sign deposit message with both private keys
4. Compute deposit data root
5. Call `deposit()` with all parameters + ETH value
6. Contract verifies signatures and root
7. Contract adds deposit to Merkle tree
8. Emits `DepositEvent` with deposit details

## Security Considerations

### Key Management

- Never reuse validator keys across networks
- Store private keys securely (hardware wallet or HSM recommended)
- Back up withdrawal credentials separately
- Use different keys for node and consensus

### Withdrawal Address

- Set withdrawal credentials to a secure address you control
- Cannot be changed after deposit (in most configurations)
- Consider using a multi-sig or cold wallet

### Deposit Root Verification

- Always verify `deposit_data_root` matches expected value
- Contract will reject mismatched roots
- Use the provided `compute_deposit_data_root()` helper

### Gas Estimation

- Deposit function requires approximately 300-500k gas
- Set sufficient gas limit to avoid failed deposits
- Failed deposits will revert and refund ETH

## When to Use

Use the deposit contract when:
- Becoming a Seismic validator
- Increasing validator stake (additional deposits)
- Building validator onboarding tools
- Monitoring validator deposits
- Verifying deposit Merkle tree

## Contract Source

This ABI matches the `IDepositContract` interface defined in:
```
contracts/src/seismic-std-lib/DepositContract.sol
```

## See Also

- [compute_deposit_data_root()](compute-deposit-data-root.md) - Compute deposit data root
- [make_withdrawal_credentials()](make-withdrawal-credentials.md) - Generate withdrawal credentials
- [DIRECTORY_ABI](directory.md) - Viewing key directory
- [SRC20_ABI](src20-abi.md) - Privacy token standard
- [Client Documentation](../client/) - Client creation and configuration
