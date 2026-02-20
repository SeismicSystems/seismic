---
description: Python SDK for Seismic, built on web3.py
icon: snake
---

# Seismic Python

Python SDK for [Seismic](https://seismic.systems), built on [web3.py](https://github.com/ethereum/web3.py). Requires **Python 3.10+**.

```bash
pip install seismic-web3
```

Or with [uv](https://docs.astral.sh/uv/):

```bash
uv add seismic-web3
```

## Quick Example

```python
from seismic_web3 import SEISMIC_TESTNET, PrivateKey

pk = PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX"))

# Wallet client — full capabilities (requires private key)
w3 = SEISMIC_TESTNET.wallet_client(pk)

contract = w3.seismic.contract(address="0x...", abi=ABI)

# Shielded write — calldata is encrypted
tx_hash = contract.write.setNumber(42)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Signed read — encrypted eth_call, proves your identity
result = contract.read.getNumber()
```

```python
# Public client — read-only (no private key needed)
public = SEISMIC_TESTNET.public_client()

contract = public.seismic.contract(address="0x...", abi=ABI)
result = contract.tread.getNumber()
```

## Documentation Navigation

### Getting Started

| Section                   | Description                                    |
| ------------------------- | ---------------------------------------------- |
| [**Client**](client/)     | Create sync/async wallet and public clients    |
| [**Chains**](chains/)     | Chain configuration (SEISMIC\_TESTNET, SANVIL) |
| [**Contract**](contract/) | Interact with shielded and public contracts    |

### Guides & Examples

| Section                   | Description                                 |
| ------------------------- | ------------------------------------------- |
| [**Guides**](guides/)     | Step-by-step tutorials for common workflows |
| [**Examples**](examples/) | Complete runnable code examples             |

### API Reference

| Section                                                                                                 | Description                                            |
| ------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| [**API Reference**](api-reference/)                                                                     | Complete API documentation for all types and functions |
| [**Types**](../../gitbook/client-libraries/seismic-python/api-reference/types/)                         | Primitive types (Bytes32, PrivateKey, etc.)            |
| [**Transaction Types**](../../gitbook/client-libraries/seismic-python/api-reference/transaction-types/) | Seismic transaction dataclasses                        |
| [**EIP-712**](../../gitbook/client-libraries/seismic-python/api-reference/eip712/)                      | EIP-712 typed data signing functions                   |

### Advanced Features

| Section                         | Description                                |
| ------------------------------- | ------------------------------------------ |
| [**Namespaces**](namespaces/)   | w3.seismic namespace methods               |
| [**Precompiles**](precompiles/) | Privacy-preserving cryptographic functions |
| [**SRC20**](src20/)             | SRC20 token standard support               |
| [**ABIs**](abis/)               | Built-in contract ABIs and helpers         |

## Quick Links

### By Task

* **Send a shielded transaction** → [Shielded Write Guide](guides/shielded-write.md)
* **Execute a signed read** → [Signed Reads Guide](guides/signed-reads.md)
* **Work with SRC20 tokens** → [SRC20 Documentation](src20/)
* **Use async client** → [Client Documentation](client/create-async-wallet-client.md)
* **Configure custom chain** → [Chains Documentation](chains/chain-config.md)

### By Component

* **Client factories** → [create\_wallet\_client](client/create-wallet-client.md), [create\_public\_client](client/create-public-client.md)
* **Contract wrappers** → [ShieldedContract](contract/shielded-contract.md), [PublicContract](contract/public-contract.md)
* **Encryption** → [EncryptionState](client/encryption-state.md), [Precompiles](precompiles/)
* **Transaction types** → [UnsignedSeismicTx](api-reference/signature/unsigned-seismic-tx.md), [SeismicElements](api-reference/signature/seismic-elements.md)

## Features

* **🔒 Shielded Transactions** - Encrypt calldata with TEE public key
* **📝 Signed Reads** - Prove identity in eth\_call
* **🪙 SRC20 Support** - Built-in support for private tokens
* **⚡ Async/Await** - Full async support with AsyncWeb3
* **🔑 EIP-712** - Structured typed data signing
* **🛠️ Precompiles** - Access Mercury EVM cryptographic precompiles
* **🌐 Web3.py Compatible** - Standard Web3 instance with Seismic extensions

## Architecture

The SDK extends `web3.py` with a custom `w3.seismic` namespace:

```
Web3 (standard web3.py)
├── eth (standard)
├── net (standard)
└── seismic (Seismic-specific) ✨
    ├── send_shielded_transaction()
    ├── signed_call()
    ├── get_tee_public_key()
    ├── deposit()
    └── contract() → ShieldedContract
                     ├── .write (shielded)
                     ├── .read (signed)
                     ├── .twrite (transparent)
                     ├── .tread (transparent)
                     └── .dwrite (debug)
```

## Next Steps

1. [**Install and setup a client**](client/) - Get connected to Seismic
2. [**Send your first shielded transaction**](guides/shielded-write.md) - Step-by-step guide
3. [**Explore the API reference**](api-reference/) - Deep dive into all types and functions
4. [**Check out examples**](examples/) - See complete runnable code
