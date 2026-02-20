---
description: Complete wallet client setup with both sync and async variants
icon: wallet
---

# Basic Wallet Setup

This example demonstrates how to set up Seismic wallet clients in both synchronous and asynchronous variants. A wallet client provides full capabilities including shielded writes, signed reads, and deposits.

## Prerequisites

```bash
# Install the SDK
pip install seismic-web3

# Set your private key (32 bytes hex, without 0x prefix)
export PRIVATE_KEY="your_64_char_hex_private_key"
```

## Synchronous Wallet Client

The sync client is simpler and suitable for scripts, CLI tools, and applications that don't need concurrent operations.

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey, SEISMIC_TESTNET

# Load private key from environment
private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

# Method 1: Using chain config (recommended)
w3 = SEISMIC_TESTNET.wallet_client(private_key)

# Method 2: Using RPC URL directly
# w3 = create_wallet_client(
#     "https://gcp-1.seismictest.net/rpc",
#     private_key=private_key,
# )

# Verify connection
print(f"Connected to chain ID: {w3.eth.chain_id}")
print(f"Current block number: {w3.eth.block_number}")
print(f"Your address: {w3.eth.default_account}")

# Get TEE public key (required for encryption)
tee_pubkey = w3.seismic.get_tee_public_key()
print(f"TEE public key: {tee_pubkey.to_0x_hex()}")

# Check your balance
balance = w3.eth.get_balance(w3.eth.default_account)
print(f"Balance: {w3.from_wei(balance, 'ether')} ETH")
```

## Asynchronous Wallet Client

The async client enables concurrent operations, WebSocket connections, and is ideal for high-performance applications.

```python
import asyncio
import os
from seismic_web3 import create_async_wallet_client, PrivateKey, SEISMIC_TESTNET


async def main():
    # Load private key
    private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

    # Method 1: Using chain config with HTTP
    w3 = await SEISMIC_TESTNET.async_wallet_client(private_key)

    # Method 2: Using WebSocket (for event streaming)
    # w3 = await create_async_wallet_client(
    #     "wss://gcp-1.seismictest.net/ws",
    #     private_key=private_key,
    #     ws=True,
    # )

    try:
        # Verify connection
        chain_id = await w3.eth.chain_id
        block_number = await w3.eth.block_number
        print(f"Connected to chain ID: {chain_id}")
        print(f"Current block number: {block_number}")

        # Get TEE public key
        tee_pubkey = await w3.seismic.get_tee_public_key()
        print(f"TEE public key: {tee_pubkey.to_0x_hex()}")

        # Check balance
        address = w3.eth.default_account
        balance = await w3.eth.get_balance(address)
        print(f"Balance: {w3.from_wei(balance, 'ether')} ETH")

    finally:
        # Clean up async resources
        await w3.provider.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
```

## Custom Encryption Key

By default, a random ephemeral encryption key is generated. You can provide a deterministic key for reproducibility:

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey

signing_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
encryption_key = PrivateKey.from_hex_str(os.environ["ENCRYPTION_KEY"])

w3 = create_wallet_client(
    "https://gcp-1.seismictest.net/rpc",
    private_key=signing_key,
    encryption_sk=encryption_key,
)

# Access encryption state
print(f"Client encryption pubkey: {w3.seismic.encryption.encryption_pubkey.to_0x_hex()}")
print(f"Derived AES key: {w3.seismic.encryption.aes_key.to_0x_hex()}")
```

## Error Handling

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey
from web3.exceptions import Web3Exception


def create_client_with_retry(rpc_url: str, private_key: PrivateKey, max_retries: int = 3):
    """Create wallet client with connection retry logic."""
    for attempt in range(max_retries):
        try:
            w3 = create_wallet_client(rpc_url, private_key=private_key)
            # Verify connection by fetching block number
            _ = w3.eth.block_number
            print(f"Connected successfully on attempt {attempt + 1}")
            return w3
        except Web3Exception as e:
            print(f"Attempt {attempt + 1} failed: {e}")
            if attempt == max_retries - 1:
                raise
            # Wait before retry (exponential backoff)
            import time
            time.sleep(2 ** attempt)

    raise RuntimeError("Failed to create wallet client after all retries")


# Usage
private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = create_client_with_retry("https://gcp-1.seismictest.net/rpc", private_key)
```

## Expected Output

```
Connected to chain ID: 31337
Current block number: 12345
Your address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
TEE public key: 0x028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0
Balance: 100.0 ETH
```

## Common Variations

### Using SANVIL Testnet

```python
import os
from seismic_web3 import SANVIL, PrivateKey

private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = SANVIL.wallet_client(private_key)
```

### Multiple Clients (Multi-Account)

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey

pk1 = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY_1"])
pk2 = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY_2"])

# Create separate clients for each account
w3_account1 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=pk1)
w3_account2 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=pk2)

# Each client uses its own signing and encryption keys
print(f"Account 1: {w3_account1.eth.default_account}")
print(f"Account 2: {w3_account2.eth.default_account}")
```

### Connection Verification

```python
import os
from seismic_web3 import create_wallet_client, PrivateKey


def verify_client_ready(w3):
    """Verify client is properly initialized and connected."""
    try:
        # Check standard web3 connection
        assert w3.is_connected(), "Web3 provider not connected"

        # Check chain ID
        chain_id = w3.eth.chain_id
        assert chain_id > 0, "Invalid chain ID"

        # Check TEE public key is available
        tee_pk = w3.seismic.get_tee_public_key()
        assert len(tee_pk.value) == 33, "Invalid TEE public key length"

        # Check encryption state is initialized
        assert w3.seismic.encryption is not None, "Encryption not initialized"
        assert len(w3.seismic.encryption.aes_key.value) == 32, "Invalid AES key"

        print("Client verification passed")
        return True
    except AssertionError as e:
        print(f"Client verification failed: {e}")
        return False


private_key = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])
w3 = create_wallet_client("https://gcp-1.seismictest.net/rpc", private_key=private_key)
verify_client_ready(w3)
```

## Next Steps

- [Shielded Write Complete](shielded-write-complete.md) - Send encrypted transactions
- [Signed Read Pattern](signed-read-pattern.md) - Execute authenticated calls
- [SRC20 Workflow](src20-workflow.md) - Work with private tokens
- [Async Patterns](async-patterns.md) - Advanced async techniques

## See Also

- [create_wallet_client](../client/create-wallet-client.md) - API reference
- [create_async_wallet_client](../client/create-async-wallet-client.md) - Async API reference
- [Chains Configuration](../chains/) - Pre-configured chain constants
- [EncryptionState](../client/encryption-state.md) - Encryption internals
