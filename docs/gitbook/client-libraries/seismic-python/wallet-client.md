---
icon: wallet
---

# Wallet Client

The wallet client provides full capabilities: shielded writes, signed reads, and deposits. It requires a private key.

## Sync

```python
from seismic_web3 import create_wallet_client
from seismic_web3.chains import SEISMIC_TESTNET
from eth_account import Account

w3 = create_wallet_client(
    chain=SEISMIC_TESTNET,
    private_key=Account.from_key("0x...").key,
)
```

## Async

```python
from seismic_web3 import create_async_wallet_client
from seismic_web3.chains import SEISMIC_TESTNET
from eth_account import Account

w3 = await create_async_wallet_client(
    chain=SEISMIC_TESTNET,
    private_key=Account.from_key("0x...").key,
)
```

Pass `ws=True` for WebSocket connections (automatic URL selection from the chain config).

## Automatic encryption

The wallet client automatically handles encryption setup:

1. Fetches the node's TEE public key
2. Derives an AES-GCM key via ECDH between an ephemeral keypair and the TEE public key
3. Encrypts calldata for all shielded operations

Access the encryption state via `w3.seismic.encryption`.

## Available methods

The `w3.seismic` namespace on a wallet client provides:

| Method                              | Purpose                                           |
| ----------------------------------- | ------------------------------------------------- |
| `send_shielded_transaction()`       | Send an encrypted `TxSeismic` (type `0x4A`)       |
| `debug_send_shielded_transaction()` | Same, but returns plaintext + encrypted + tx hash |
| `signed_call()`                     | Encrypted `eth_call` that proves `msg.sender`     |
| `get_tee_public_key()`              | Fetch the node's TEE public key                   |
| `contract()`                        | Create a shielded contract wrapper                |
