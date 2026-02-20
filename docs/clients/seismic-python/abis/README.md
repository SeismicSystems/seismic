---
description: Built-in contract ABIs and helpers
icon: file-code
---

# ABIs

The Seismic Python SDK includes built-in ABIs for core protocol contracts and helper functions for common operations.

## Built-in ABIs

| Constant | Description |
|----------|-------------|
| `SRC20_ABI` | Standard SRC20 token contract ABI |
| `DEPOSIT_CONTRACT_ABI` | Deposit contract ABI for ETH/token deposits |
| `DEPOSIT_CONTRACT_ADDRESS` | Canonical deposit contract address |
| `DIRECTORY_ABI` | Viewing key directory contract ABI |
| `DIRECTORY_ADDRESS` | Canonical directory contract address |

## Helper Functions

| Function | Description |
|----------|-------------|
| `compute_deposit_data_root()` | Compute merkle root for deposit data |
| `make_withdrawal_credentials()` | Generate withdrawal credentials for deposits |

## Quick Examples

### Using SRC20_ABI

```python
from seismic_web3 import SRC20_ABI, create_wallet_client

w3 = create_wallet_client(...)
token = w3.seismic.contract("0x...", SRC20_ABI)

# Use shielded methods
await token.write.transfer(recipient, amount)
```

### Using Deposit Contract

```python
from seismic_web3 import (
    DEPOSIT_CONTRACT_ABI,
    DEPOSIT_CONTRACT_ADDRESS,
    create_wallet_client,
)

w3 = create_wallet_client(...)
deposit_contract = w3.eth.contract(
    address=DEPOSIT_CONTRACT_ADDRESS,
    abi=DEPOSIT_CONTRACT_ABI,
)

# Query deposit count
count = deposit_contract.functions.get_deposit_count().call()
```

### Using Directory Contract

```python
from seismic_web3 import (
    DIRECTORY_ABI,
    DIRECTORY_ADDRESS,
    create_wallet_client,
)

w3 = create_wallet_client(...)
directory = w3.eth.contract(
    address=DIRECTORY_ADDRESS,
    abi=DIRECTORY_ABI,
)

# Check if viewing key registered
has_key = directory.functions.hasKey(user_address, token_address).call()
```

### Compute Deposit Data Root

```python
from seismic_web3 import compute_deposit_data_root

# Prepare deposit data
deposit_data = {
    'pubkey': pubkey_bytes,
    'withdrawal_credentials': withdrawal_creds,
    'amount': amount_gwei,
    'signature': signature_bytes,
}

# Compute root for verification
root = compute_deposit_data_root(deposit_data)
```

### Make Withdrawal Credentials

```python
from seismic_web3 import make_withdrawal_credentials

# Generate BLS withdrawal credentials
withdrawal_creds = make_withdrawal_credentials(withdrawal_address)

# Use in deposit
w3.seismic.deposit(
    pubkey=validator_pubkey,
    withdrawal_credentials=withdrawal_creds,
    signature=signature,
    deposit_data_root=root,
    value=32_000_000_000,  # 32 ETH in gwei
)
```

## See Also

- [SRC20 Documentation](../src20/) - SRC20 token standard
- [Client Documentation](../client/) - Client creation
- [Contract Documentation](../contract/) - Contract interaction patterns
