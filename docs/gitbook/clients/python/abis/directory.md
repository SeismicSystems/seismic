---
description: Directory contract ABI and address constants for viewing key management
icon: address-book
---

# Directory Contract

ABI and address constants for Seismic's Directory contract, which manages per-user AES-256 encryption keys.

## Overview

The Directory contract is deployed at a fixed genesis address on all Seismic networks. It stores per-user viewing keys using shielded storage (`suint256`), enabling privacy-preserving balance queries and event decryption.

Users register their AES-256 encryption keys in the Directory, which are then used by contracts to encrypt sensitive data (like token balances and amounts) so only the key holder can decrypt them.

## Constants

### DIRECTORY_ADDRESS

The canonical Directory contract address, deployed at a fixed genesis address on all Seismic networks.

```python
from seismic_web3 import DIRECTORY_ADDRESS

# Genesis address for Directory contract
DIRECTORY_ADDRESS: str = "0x1000000000000000000000000000000000000004"
```

### DIRECTORY_ABI

Complete ABI for the Directory contract, including only the four functions needed by the Python SDK.

```python
from seismic_web3 import DIRECTORY_ABI

DIRECTORY_ABI: list[dict[str, Any]]
```

## Import

```python
from seismic_web3 import (
    DIRECTORY_ABI,
    DIRECTORY_ADDRESS,
)
```

## ABI Contents

### Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `checkHasKey(address)` | `_addr` | `bool` | Check if address has registered a viewing key (view) |
| `keyHash(address)` | `to` | `bytes32` | Get hash of address's viewing key (view) |
| `getKey()` | None | `uint256` | Get caller's viewing key (view, shielded) |
| `setKey(suint256)` | `_key` | None | Set caller's viewing key (nonpayable, shielded) |

### Events

None included in this ABI subset.

## Full ABI

```python
DIRECTORY_ABI: list[dict[str, Any]] = [
    {
        "type": "function",
        "name": "checkHasKey",
        "inputs": [{"name": "_addr", "type": "address"}],
        "outputs": [{"name": "", "type": "bool"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "keyHash",
        "inputs": [{"name": "to", "type": "address"}],
        "outputs": [{"name": "", "type": "bytes32"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "getKey",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
        "stateMutability": "view",
    },
    {
        "type": "function",
        "name": "setKey",
        "inputs": [{"name": "_key", "type": "suint256"}],
        "outputs": [],
        "stateMutability": "nonpayable",
    },
]
```

## Usage Examples

### Creating a Contract Instance

```python
from seismic_web3 import (
    create_wallet_client,
    DIRECTORY_ABI,
    DIRECTORY_ADDRESS,
)

# Create client
w3 = create_wallet_client(
    rpc_url="https://sepolia.seismic.foundation",
    private_key="0x...",
)

# Create directory contract instance
directory = w3.eth.contract(
    address=DIRECTORY_ADDRESS,
    abi=DIRECTORY_ABI,
)
```

### Check if User Has Key

```python
# Check if address has registered a viewing key
user_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"

has_key = directory.functions.checkHasKey(user_address).call()

if has_key:
    print(f"{user_address} has registered a viewing key")
else:
    print(f"{user_address} has not registered a viewing key")
```

### Get Key Hash

```python
# Get hash of user's viewing key
user_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0"

key_hash = directory.functions.keyHash(user_address).call()
print(f"Key hash: {key_hash.hex()}")
```

### Register Viewing Key

```python
from seismic_web3 import Suint256
import secrets

# Generate a random 256-bit AES key
key_bytes = secrets.token_bytes(32)
key_int = int.from_bytes(key_bytes, byteorder="big")

# Wrap as shielded uint256
viewing_key = Suint256(key_int)

# Register the key (uses shielded storage)
tx_hash = directory.functions.setKey(viewing_key).transact()
print(f"Key registration tx: {tx_hash.hex()}")

# Save key locally for decryption
with open("viewing_key.bin", "wb") as f:
    f.write(key_bytes)
```

### Get Your Own Key

```python
# Retrieve your own viewing key (shielded read)
# This requires a signed call to prove your identity

from web3.contract import Contract

# Create shielded contract wrapper for signed reads
directory_shielded = w3.seismic.contract(
    address=DIRECTORY_ADDRESS,
    abi=DIRECTORY_ABI,
)

# Get your key using signed read
result = directory_shielded.read.getKey()

if result:
    from eth_abi import decode
    key_value = decode(['uint256'], result)[0]
    key_bytes = key_value.to_bytes(32, byteorder="big")
    print(f"Your viewing key: {key_bytes.hex()}")
else:
    print("No key registered")
```

### Update Viewing Key

```python
from seismic_web3 import Suint256
import secrets

# Generate new key
new_key_bytes = secrets.token_bytes(32)
new_key_int = int.from_bytes(new_key_bytes, byteorder="big")
new_viewing_key = Suint256(new_key_int)

# Update key (replaces existing key)
tx_hash = directory.functions.setKey(new_viewing_key).transact()
print(f"Key update tx: {tx_hash.hex()}")
```

### Using with SRC20 Tokens

```python
from seismic_web3 import SRC20_ABI, Suint256

# First, register your viewing key
viewing_key = Suint256(...)
directory.functions.setKey(viewing_key).transact()

# Now SRC20 contracts can encrypt data for you
token = w3.seismic.contract(address="0x...", abi=SRC20_ABI)

# Your encrypted balance can be decrypted with your viewing key
encrypted_balance = token.read.balanceOf()
# ... decrypt using your viewing key ...
```

## How Viewing Keys Work

### Registration

1. User generates a 256-bit AES-GCM encryption key
2. User calls `setKey()` with shielded key value
3. Contract stores key in shielded storage (encrypted at rest)
4. Key is associated with user's address

### Key Storage

- Keys are stored using `suint256` (shielded uint256)
- Only accessible to the key owner via signed reads
- Contracts can compute key hash without revealing key value

### Key Hash

- Public identifier for encryption key
- Used by contracts to tag encrypted data
- Computed as SHA-256 hash of key value
- Visible to everyone (not sensitive)

### Encryption Flow

1. Contract checks if recipient has registered key (`checkHasKey`)
2. Contract retrieves recipient's key hash (`keyHash`)
3. Contract encrypts sensitive data using key
4. Contract emits event with `encryptKeyHash` + `encryptedAmount`
5. Recipient decrypts using their locally-stored key

## Privacy Features

### What Gets Encrypted
- Viewing key value (stored as `suint256`)
- Data encrypted by contracts using the viewing key

### What Remains Visible
- Whether an address has a key registered (`checkHasKey`)
- Hash of the viewing key (`keyHash`)
- Contract interactions (transactions)

### Decryption

To decrypt data encrypted with your viewing key:

```python
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

# Your viewing key (retrieved from local storage)
key_bytes = bytes.fromhex("...")

# Encrypted data from event
nonce = bytes.fromhex("...")  # 12-byte nonce
ciphertext = bytes.fromhex("...")  # Encrypted data

# Decrypt
aesgcm = AESGCM(key_bytes)
plaintext = aesgcm.decrypt(nonce, ciphertext, associated_data=b"")

# Parse decrypted value
amount = int.from_bytes(plaintext, byteorder="big")
```

## Security Considerations

### Key Generation

- Use cryptographically secure random number generator
- Never reuse keys across networks
- Store keys securely (encrypted local storage recommended)

```python
import secrets

# Good: cryptographically secure
key_bytes = secrets.token_bytes(32)

# Bad: predictable
key_bytes = b"my-secret-key-12345678901234567"  # Never do this!
```

### Key Storage

- Never commit viewing keys to version control
- Use encrypted storage for production keys
- Consider hardware security modules (HSM) for high-value accounts
- Back up keys securely (losing key = losing ability to decrypt)

### Key Rotation

- Consider rotating keys periodically
- Old encrypted data remains encrypted with old key
- New data uses new key after rotation
- Keep old keys to decrypt historical data

### Privacy Tradeoffs

- Key hash is public (reveals key identity, not value)
- `checkHasKey` reveals whether user has registered
- Transaction patterns visible on-chain
- Key registration is a public action

## When to Use

Use the Directory contract when:
- Registering your viewing key for privacy features
- Checking if a user can receive encrypted data
- Building privacy-preserving applications
- Implementing encrypted balance queries
- Creating tools for viewing key management

## Comparison with Other Contracts

| Contract | Purpose | Key Storage | Privacy Level |
|----------|---------|-------------|---------------|
| Directory | Viewing key management | Shielded | High (keys encrypted) |
| SRC20 | Token transfers | N/A | High (amounts encrypted) |
| Deposit | Validator deposits | N/A | None (public deposits) |

## Contract Source

This ABI matches the TypeScript `DirectoryAbi` in seismic-viem and the Solidity contract at:
```
contracts/src/seismic-std-lib/Directory.sol
```

Note: Only the four functions needed by the Python SDK are included in this ABI. The full contract may have additional functions.

## See Also

- [SRC20_ABI](src20-abi.md) - Privacy token standard that uses Directory keys
- [Suint256 Type](../api-reference/types/suint256.md) - Shielded integer type
- [ShieldedContract](../contract/) - Contract interaction patterns
- [DEPOSIT_CONTRACT_ABI](deposit-contract.md) - Validator deposits
- [AES-GCM Encrypt](../precompiles/aes-gcm-encrypt.md) - On-chain encryption
- [AES-GCM Decrypt](../precompiles/aes-gcm-decrypt.md) - On-chain decryption
