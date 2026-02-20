---
description: CLAUDE.md template for Python dapp development with seismic-web3
icon: snake
---

# Seismic Python

Use this template when your project uses `seismic-web3` (the Seismic Python SDK) to interact with Seismic contracts. This SDK extends `web3.py` with a `w3.seismic` namespace for shielded transactions and signed reads.

## The template

Copy the entire block below and save it as `CLAUDE.md` in your project root.

````markdown
# [Your Project Name]

## Seismic Overview

Seismic is an EVM-compatible L1 with on-chain privacy. Nodes run inside TEEs (Intel TDX). The Solidity compiler adds shielded types (`suint256`, `saddress`, `sbool`) that are invisible outside the TEE. Client libraries handle transaction encryption and signed reads automatically.

## Key Concepts

- **Shielded types**: `suint256`, `saddress`, `sbool` — on-chain private state, only readable via signed reads
- **TxSeismic (type 0x4A)**: Encrypts calldata before broadcast. The SDK handles this automatically.
- **Signed reads**: `eth_call` zeroes `msg.sender` on Seismic. Use `.read` namespace on ShieldedContract for reads that need a valid sender.
- **Encryption pubkeys**: 33-byte compressed secp256k1 keys. The client fetches and manages these on construction.
- **Legacy gas**: Seismic transactions use `gas_price` + `gas_limit`, NOT EIP-1559.

## SDK: seismic-web3

### Install

```bash
pip install seismic-web3
# or with uv
uv add seismic-web3
```

The PyPI package is `seismic-web3`. The import is `from seismic_web3 import ...`.

### Key imports

```python
from seismic_web3 import (
    SEISMIC_TESTNET,        # Chain config for devnet
    SANVIL,                 # Chain config for local sanvil
    PrivateKey,
    create_wallet_client,
    create_public_client,
    create_async_wallet_client,
    create_async_public_client,
)
```

## Core Patterns

### Create a wallet client (sync)

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))
w3 = SEISMIC_TESTNET.wallet_client(pk)

# Standard web3.py methods work
block = w3.eth.get_block("latest")
```

### Create a wallet client (async)

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))

# HTTP
w3 = await SEISMIC_TESTNET.async_wallet_client(pk)

# WebSocket
w3 = await SEISMIC_TESTNET.async_wallet_client(pk, ws=True)
```

### Create a public client (read-only)

```python
from seismic_web3 import SEISMIC_TESTNET

# Sync
public = SEISMIC_TESTNET.public_client()

# Async
public = await SEISMIC_TESTNET.async_public_client()
```

### Create a client with a URL

```python
from seismic_web3 import create_wallet_client, create_public_client, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))

w3 = create_wallet_client("http://127.0.0.1:8545", private_key=pk)
public = create_public_client("http://127.0.0.1:8545")
```

### Shielded contract interaction

```python
contract = w3.seismic.contract(
    address="0xCONTRACT_ADDRESS",
    abi=my_contract_abi,
)

# Shielded write — sends a type 0x4A transaction with encrypted calldata
tx_hash = contract.write.transfer(recipient_address, amount)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Signed read — proves identity so msg.sender is set correctly
balance = contract.read.getBalance(user_address)

# Transparent write (standard tx, no encryption)
tx_hash = contract.twrite.setPublicValue(42)

# Transparent read (standard eth_call, no signing)
value = contract.tread.getPublicValue()
```

### Contract namespaces

| Namespace | Description                              | When to use                                |
| --------- | ---------------------------------------- | ------------------------------------------ |
| `.write`  | Shielded write (encrypted calldata)      | Functions that write shielded state        |
| `.read`   | Signed read (proves msg.sender)          | View functions that check msg.sender       |
| `.twrite` | Transparent write (standard tx)          | Functions that only write public state     |
| `.tread`  | Transparent read (standard call)         | View functions that don't check msg.sender |
| `.dwrite` | Debug write (shielded + verbose logging) | Development/debugging                      |

### Encryption

The wallet client automatically handles encryption. On creation, it fetches the node's TEE public key and derives a shared AES-GCM key via ECDH. The encryption state is accessible at `w3.seismic.encryption` if needed.

## Common Mistakes

1. **Using standard web3.py contract calls** — `contract.functions.transfer(...).transact()` sends a standard Ethereum transaction. It won't encrypt calldata. Use `contract.write.transfer(...)` from the Seismic contract instance.
2. **Wrong package name** — The PyPI package is `seismic-web3`, not `seismic-python` or `seismic-py`. The import is `from seismic_web3 import ...`.
3. **Using EIP-1559 gas params** — Seismic uses legacy gas. Do NOT pass `maxFeePerGas`/`maxPriorityFeePerGas`.
4. **Using `.tread` when `.read` is needed** — If the contract function checks `msg.sender`, you must use `.read` (signed read). Using `.tread` zeroes the sender and the `require` will fail.
5. **Forgetting `await` on async clients** — `SEISMIC_TESTNET.async_wallet_client(pk)` is async. Missing `await` gives a coroutine, not a client.
6. **PrivateKey format** — Use `PrivateKey(bytes.fromhex("..."))`, not a hex string directly. The key should NOT include the `0x` prefix when using `fromhex`.

## Networks

| Network        | Chain ID | Chain config      | RPC URL                             |
| -------------- | -------- | ----------------- | ----------------------------------- |
| Devnet         | 5124     | `SEISMIC_TESTNET` | `https://node-2.seismicdev.net/rpc` |
| Devnet (WS)    | 5124     | `SEISMIC_TESTNET` | `wss://node-2.seismicdev.net/ws`    |
| Local (sanvil) | 31337    | `SANVIL`          | `http://127.0.0.1:8545`             |

Faucet: https://faucet-2.seismicdev.net/

## Links

- [Client Documentation](https://docs.seismic.systems/client-libraries/seismic-python/)
- [ShieldedContract](https://docs.seismic.systems/client-libraries/seismic-python/contract/shielded-contract)
- [Contract Namespaces](https://docs.seismic.systems/client-libraries/seismic-python/contract/namespaces/)
- [Shielded Write Guide](https://docs.seismic.systems/client-libraries/seismic-python/guides/shielded-write)
- [Signed Reads Guide](https://docs.seismic.systems/client-libraries/seismic-python/guides/signed-reads)
- [GitHub: seismic-py](https://github.com/SeismicSystems/seismic-py)
````

## What this teaches Claude

- **Correct package and import names** — Claude will use `seismic-web3` for pip and `seismic_web3` for imports
- **Chain config objects** — Claude will use `SEISMIC_TESTNET` and `SANVIL` instead of constructing configs manually
- **Contract namespaces** — Claude will pick `.write`/`.read`/`.twrite`/`.tread` correctly based on whether the operation needs encryption or signing
- **Client construction** — Claude will use the chain config factory methods instead of raw Web3 constructors
- **PrivateKey type** — Claude will wrap keys in `PrivateKey(bytes.fromhex(...))` correctly

## Customizing

After pasting the template:

- Replace `[Your Project Name]` with your project name
- Add your contract ABIs and addresses
- Specify sync vs. async preference if your project uses one consistently
- Add any project-specific environment variable names for keys and RPC URLs
