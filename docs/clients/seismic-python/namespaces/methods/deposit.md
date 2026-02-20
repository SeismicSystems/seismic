---
description: Submit a validator deposit to the Seismic deposit contract
icon: coins
---

# deposit

Submit a validator deposit to the Seismic deposit contract. This registers a new validator with the network and stakes the required ETH.

***

## Overview

To become a Seismic validator, you must submit a deposit containing:

* **Node keys**: ED25519 keypair for execution layer validation
* **Consensus keys**: BLS12-381 keypair for consensus layer participation
* **Deposit amount**: 32 ETH (in wei)
* **Signatures**: Proofs of key ownership
* **Withdrawal credentials**: Where to send your stake when you exit

This method encodes and broadcasts a transparent deposit transaction to the deposit contract.

***

## Signatures

<table><thead><tr><th width="400">Sync</th><th>Async</th></tr></thead><tbody><tr><td><pre class="language-python"><code class="lang-python">def deposit(
    *,
    node_pubkey: bytes,
    consensus_pubkey: bytes,
    withdrawal_credentials: bytes,
    node_signature: bytes,
    consensus_signature: bytes,
    deposit_data_root: bytes,
    value: int,
    address: str = DEPOSIT_CONTRACT_ADDRESS,
) -> HexBytes
</code></pre></td><td><pre class="language-python"><code class="lang-python">async def deposit(
    *,
    node_pubkey: bytes,
    consensus_pubkey: bytes,
    withdrawal_credentials: bytes,
    node_signature: bytes,
    consensus_signature: bytes,
    deposit_data_root: bytes,
    value: int,
    address: str = DEPOSIT_CONTRACT_ADDRESS,
) -> HexBytes
</code></pre></td></tr></tbody></table>

***

## Parameters

All parameters are **keyword-only** to prevent mix-ups between the many `bytes` arguments.

| Parameter                | Type    | Default                                                      | Description                                                                                                                                                      |
| ------------------------ | ------- | ------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `node_pubkey`            | `bytes` | Required                                                     | 32-byte ED25519 public key identifying the validator node on the execution layer                                                                                 |
| `consensus_pubkey`       | `bytes` | Required                                                     | 48-byte BLS12-381 public key for block proposals and attestations on the consensus layer                                                                         |
| `withdrawal_credentials` | `bytes` | Required                                                     | 32-byte credential specifying where staked ETH is sent upon exit. Generate with [`make_withdrawal_credentials()`](../../abis/make-withdrawal-credentials.md)     |
| `node_signature`         | `bytes` | Required                                                     | 64-byte ED25519 signature over the deposit data, proving ownership of the node key                                                                               |
| `consensus_signature`    | `bytes` | Required                                                     | 96-byte BLS12-381 signature over the deposit data, proving ownership of the consensus key                                                                        |
| `deposit_data_root`      | `bytes` | Required                                                     | 32-byte Merkle root of the deposit data for consensus-layer verification. Generate with [`compute_deposit_data_root()`](../../abis/compute-deposit-data-root.md) |
| `value`                  | `int`   | Required                                                     | Deposit amount in wei. Typically `32 * 10**18` (32 ETH) for a full validator                                                                                     |
| `address`                | `str`   | [`DEPOSIT_CONTRACT_ADDRESS`](../../abis/deposit-contract.md) | Address of the deposit contract to call                                                                                                                          |

### Parameter Details

#### node\_pubkey (32 bytes)

The ED25519 public key for your validator node. Used for execution layer operations.

#### consensus\_pubkey (48 bytes)

The BLS12-381 public key for consensus participation. Used for block proposals and attestations.

#### withdrawal\_credentials (32 bytes)

Specifies where to send your stake when you exit. Can be:

* **Execution address**: `0x01` + 11 zero bytes + 20-byte address
* **BLS credentials**: `0x00` + BLS public key hash

#### node\_signature (64 bytes)

ED25519 signature over the deposit data using your node private key. Proves you own the node key.

#### consensus\_signature (96 bytes)

BLS12-381 signature over the deposit data using your consensus private key. Proves you own the consensus key.

#### deposit\_data\_root (32 bytes)

Merkle root of the deposit data. Used for verification by the consensus layer.

#### value (int)

Deposit amount in wei. Typically:

* **Full validator**: 32 ETH = `32 * 10**18` wei
* **Partial deposit**: Any amount ≥ minimum (check network rules)

***

## Returns

**Type:** `HexBytes`

The transaction hash from the deposit transaction.

```python
# Sync
tx_hash = w3.seismic.deposit(...)
assert isinstance(tx_hash, HexBytes)

# Async
tx_hash = await w3.seismic.deposit(...)
assert isinstance(tx_hash, HexBytes)
```

***

## Examples

### Sync Usage

```python
from seismic_web3 import create_wallet_client, PrivateKey

# Create wallet client
w3 = create_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=PrivateKey(b"...32 bytes..."),
)

# Prepare deposit parameters (these would come from your validator setup)
node_pubkey = bytes.fromhex("a1b2c3d4...")  # 32 bytes
consensus_pubkey = bytes.fromhex("e5f6g7h8...")  # 48 bytes
withdrawal_credentials = bytes.fromhex("01000000...")  # 32 bytes
node_signature = bytes.fromhex("1234abcd...")  # 64 bytes
consensus_signature = bytes.fromhex("5678ef01...")  # 96 bytes
deposit_data_root = bytes.fromhex("deadbeef...")  # 32 bytes

# Submit deposit
tx_hash = w3.seismic.deposit(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    deposit_data_root=deposit_data_root,
    value=32 * 10**18,  # 32 ETH in wei
)

print(f"Deposit transaction: {tx_hash.hex()}")

# Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)
print(f"Status: {'success' if receipt['status'] == 1 else 'failed'}")
```

### Async Usage

```python
from seismic_web3 import create_async_wallet_client, PrivateKey

# Create async wallet client
w3 = await create_async_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=PrivateKey(b"...32 bytes..."),
)

# Submit deposit (parameters same as sync)
tx_hash = await w3.seismic.deposit(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    deposit_data_root=deposit_data_root,
    value=32 * 10**18,
)

print(f"Deposit transaction: {tx_hash.hex()}")

# Wait for confirmation
receipt = await w3.eth.wait_for_transaction_receipt(tx_hash)
print(f"Status: {'success' if receipt['status'] == 1 else 'failed'}")
```

### Custom Contract Address

```python
# Deposit to a different contract (e.g., testnet)
tx_hash = w3.seismic.deposit(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    deposit_data_root=deposit_data_root,
    value=32 * 10**18,
    address="0x1234567890123456789012345678901234567890",
)
```

### Verifying Deposit

```python
# Check deposit count before and after
before_count = w3.seismic.get_deposit_count()
print(f"Deposits before: {before_count}")

# Submit deposit
tx_hash = w3.seismic.deposit(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    deposit_data_root=deposit_data_root,
    value=32 * 10**18,
)

# Wait for confirmation
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Verify deposit was included
after_count = w3.seismic.get_deposit_count()
print(f"Deposits after: {after_count}")

if after_count == before_count + 1:
    print("Deposit successful!")
    print(f"Your deposit index: {after_count - 1}")
else:
    print("WARNING: Deposit count mismatch!")
```

### With Balance Check

```python
# Check balance before deposit
balance = w3.eth.get_balance(w3.eth.default_account)
required = 32 * 10**18

if balance < required:
    print(f"Insufficient balance: {balance / 10**18} ETH")
    print(f"Required: {required / 10**18} ETH")
    exit(1)

# Submit deposit
tx_hash = w3.seismic.deposit(
    node_pubkey=node_pubkey,
    consensus_pubkey=consensus_pubkey,
    withdrawal_credentials=withdrawal_credentials,
    node_signature=node_signature,
    consensus_signature=consensus_signature,
    deposit_data_root=deposit_data_root,
    value=32 * 10**18,
)

print(f"Deposit submitted: {tx_hash.hex()}")
```

***

## Implementation Details

### Contract Call

This method calls the deposit contract's `deposit()` function:

```solidity
function deposit(
    bytes calldata node_pubkey,
    bytes calldata consensus_pubkey,
    bytes calldata withdrawal_credentials,
    bytes calldata node_signature,
    bytes calldata consensus_signature,
    bytes32 deposit_data_root
) external payable;
```

The SDK:

1. Validates all parameter lengths (raises `ValueError` if wrong)
2. Encodes the function call using the deposit contract ABI
3. Sends a transaction via `eth_sendTransaction` with the deposit value
4. Returns the transaction hash

### Validation

The method validates parameter lengths before sending:

| Parameter                | Expected Length | Error if Wrong |
| ------------------------ | --------------- | -------------- |
| `node_pubkey`            | 32 bytes        | `ValueError`   |
| `consensus_pubkey`       | 48 bytes        | `ValueError`   |
| `withdrawal_credentials` | 32 bytes        | `ValueError`   |
| `node_signature`         | 64 bytes        | `ValueError`   |
| `consensus_signature`    | 96 bytes        | `ValueError`   |
| `deposit_data_root`      | 32 bytes        | `ValueError`   |

### Transparent Transaction

This is a **transparent** (unencrypted) transaction, not a shielded one. Deposit data is public information required by the consensus layer.

The transaction uses standard `eth_sendTransaction`, not `TxSeismic`.

### Genesis Contract

The default deposit contract address is [`DEPOSIT_CONTRACT_ADDRESS`](../../abis/deposit-contract.md), deployed at genesis on all Seismic networks.

***

## Deposit Data Generation

### Overview

The deposit parameters (keys, signatures, root) must be generated using the Seismic validator tooling. The SDK does **not** generate these for you.

### Typical Workflow

1.  **Generate keys** using the Seismic CLI or validator client:

    ```bash
    seismic-cli validator generate-keys \
      --output-dir ./validator_keys
    ```
2. **Create deposit data**:
   * The tool generates deposit data JSON files
   * Each file contains all required parameters
   * Signatures are pre-computed
3.  **Load deposit data** in your script:

    ```python
    import json

    with open("validator_keys/deposit_data.json") as f:
        deposit_data = json.load(f)

    # Extract parameters
    node_pubkey = bytes.fromhex(deposit_data["node_pubkey"])
    consensus_pubkey = bytes.fromhex(deposit_data["consensus_pubkey"])
    # ... etc
    ```
4. **Submit deposit** using this method

### Security Note

**Never manually construct deposit parameters** unless you fully understand the cryptographic requirements. Use official tooling to generate deposit data.

***

## Error Handling

```python
try:
    tx_hash = w3.seismic.deposit(
        node_pubkey=node_pubkey,
        consensus_pubkey=consensus_pubkey,
        withdrawal_credentials=withdrawal_credentials,
        node_signature=node_signature,
        consensus_signature=consensus_signature,
        deposit_data_root=deposit_data_root,
        value=32 * 10**18,
    )

    receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

    if receipt['status'] == 0:
        print("Deposit transaction failed!")
    else:
        print("Deposit successful!")

except ValueError as e:
    print(f"Invalid parameter: {e}")
    # Check parameter lengths

except Exception as e:
    print(f"Deposit failed: {e}")
    # Possible causes:
    # - Insufficient balance
    # - Invalid signatures
    # - Contract reverted
    # - Network error
```

***

## Security Considerations

### Key Management

* Store private keys securely
* Never expose private keys in code or logs
* Use hardware wallets for large deposits
* Backup keys in multiple secure locations

### Withdrawal Credentials

* Double-check withdrawal credentials before depositing
* Once deposited, credentials cannot be changed (until withdrawals)
* Use execution addresses (`0x01`) for easier withdrawals
* Test with small deposits first on testnet

### Signature Verification

* Verify signatures match your keys before depositing
* Use official tooling to generate signatures
* Never accept deposit data from untrusted sources

### Amount Verification

```python
# Always verify the deposit amount
deposit_eth = 32
deposit_wei = deposit_eth * 10**18

print(f"Depositing {deposit_eth} ETH ({deposit_wei} wei)")
input("Confirm deposit amount (Ctrl+C to cancel)...")

tx_hash = w3.seismic.deposit(
    # ... parameters
    value=deposit_wei,
)
```

***

## Use Cases

### Becoming a Validator

Submit your initial deposit to become a validator:

```python
# Load deposit data from validator setup
with open("deposit_data.json") as f:
    data = json.load(f)

# Submit deposit
tx_hash = w3.seismic.deposit(
    node_pubkey=bytes.fromhex(data["node_pubkey"]),
    consensus_pubkey=bytes.fromhex(data["consensus_pubkey"]),
    withdrawal_credentials=bytes.fromhex(data["withdrawal_credentials"]),
    node_signature=bytes.fromhex(data["node_signature"]),
    consensus_signature=bytes.fromhex(data["consensus_signature"]),
    deposit_data_root=bytes.fromhex(data["deposit_data_root"]),
    value=32 * 10**18,
)

print(f"Validator deposit submitted: {tx_hash.hex()}")
```

### Batch Deposits

Submit multiple validator deposits:

```python
import json
import time

# Load multiple deposit data files
with open("deposits.json") as f:
    deposits = json.load(f)

for i, data in enumerate(deposits):
    print(f"Submitting deposit {i+1}/{len(deposits)}...")

    tx_hash = w3.seismic.deposit(
        node_pubkey=bytes.fromhex(data["node_pubkey"]),
        consensus_pubkey=bytes.fromhex(data["consensus_pubkey"]),
        withdrawal_credentials=bytes.fromhex(data["withdrawal_credentials"]),
        node_signature=bytes.fromhex(data["node_signature"]),
        consensus_signature=bytes.fromhex(data["consensus_signature"]),
        deposit_data_root=bytes.fromhex(data["deposit_data_root"]),
        value=32 * 10**18,
    )

    print(f"Transaction: {tx_hash.hex()}")

    # Wait for confirmation before next deposit
    receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

    if receipt['status'] == 1:
        print(f"Deposit {i+1} confirmed!")
    else:
        print(f"WARNING: Deposit {i+1} failed!")
        break

    time.sleep(2)  # Rate limit
```

***

## Notes

### Requires Wallet Client

This method is only available on wallet clients created with `create_wallet_client()`. It requires a private key to sign the deposit transaction.

### Transparent Transaction

Unlike shielded transactions, deposits are transparent and visible on-chain. This is required for consensus layer verification.

### Minimum Deposit

Check the network's minimum deposit requirement. Most networks require 32 ETH, but testnets may have different requirements.

### Activation Time

After depositing:

1. Transaction must be confirmed (usually 1-2 blocks)
2. Consensus layer must process the deposit (varies by network)
3. Validator enters activation queue
4. Activation can take hours to days depending on queue length

***

## See Also

* [get\_deposit\_count()](get-deposit-count.md) — Read total deposits
* [get\_deposit\_root()](get-deposit-root.md) — Read deposit Merkle root
* [Deposit Contract ABI](../../../../gitbook/client-libraries/seismic-python/api-reference/abis/deposit-contract.md) — Full contract interface
* [Validator Guide](../../../../gitbook/client-libraries/seismic-python/guides/validator-deposits.md) — Complete guide to becoming a validator
* [Seismic CLI Documentation](https://docs.seismic.systems/cli) — Validator key generation tools
