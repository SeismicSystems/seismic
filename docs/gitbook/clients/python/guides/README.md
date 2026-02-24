---
description: Step-by-step guides for common workflows
icon: book-open
---

# Guides

These guides walk through the core interaction patterns in the Python SDK, from wallet setup to shielded transactions and private token operations.

## Available Guides

### [Shielded Write](shielded-write.md)

Send encrypted transactions with `TxSeismic`. Covers the encryption lifecycle, security parameters, the low-level API, and debug mode.

### [Signed Reads](signed-reads.md)

Execute encrypted `eth_call` reads that prove your identity via `msg.sender`. Covers when to use `.read` vs `.tread`, what gets encrypted, and the low-level API.

### [SRC20 Workflow](src20-workflow.md)

End-to-end token workflow: read metadata, check balances, approve, and transfer using the SRC20 standard.

### [Async Patterns](async-patterns.md)

Concurrent operations, resource cleanup, and async best practices.

## Before You Start

Both shielded writes and signed reads require a wallet client:

```python
import os
from seismic_web3 import PrivateKey, SEISMIC_TESTNET

pk = PrivateKey.from_hex_str(os.environ["PRIVATE_KEY"])

# Sync
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Async
w3 = await SEISMIC_TESTNET.async_wallet_client(pk)
```

To verify your connection:

```python
print(f"Chain ID: {w3.eth.chain_id}")
print(f"Block: {w3.eth.block_number}")
print(f"Address: {w3.eth.default_account}")
print(f"TEE public key: {w3.seismic.get_tee_public_key().to_0x_hex()}")
```

## See Also

- [Client Setup](../client/) — Full client configuration reference
- [Contract Interaction](../contract/) — Contract wrapper patterns
- [SRC20 Tokens](../src20/) — Token standard documentation
- [API Reference](../api-reference/) — Complete API documentation
