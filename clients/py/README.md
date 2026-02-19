# seismic-web3

Python SDK for [Seismic](https://seismic.systems), built on [web3.py](https://github.com/ethereum/web3.py).

## Install

Requires **Python 3.10+**.

```bash
pip install seismic-web3
```

Or with [uv](https://docs.astral.sh/uv/):

```bash
uv add seismic-web3
```

## Quick Start

```python
from seismic_web3 import create_shielded_web3, PrivateKey

w3 = create_shielded_web3(
    "http://127.0.0.1:8545",
    private_key=PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX")),
)

contract = w3.seismic.contract(address="0x...", abi=ABI)

# Shielded write — calldata is encrypted (TxSeismic type 0x4a)
tx_hash = contract.write.setNumber(42)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Shielded read — signed, encrypted eth_call
result = contract.read.getNumber()
```

## Shielded Contracts

`ShieldedContract` exposes five namespaces:

| Namespace | What it does | On-chain visibility |
|-----------|-------------|-------------------|
| `.write` | Encrypted transaction (`TxSeismic` type `0x4a`) | Calldata hidden |
| `.read` | Encrypted signed `eth_call` | Calldata + result hidden |
| `.twrite` | Standard `eth_sendTransaction` | Calldata visible |
| `.tread` | Standard `eth_call` | Calldata visible |
| `.dwrite` | Debug write — like `.write` but returns plaintext + encrypted views | Calldata hidden |

```python
contract = w3.seismic.contract(address="0x...", abi=ABI)

# Shielded write — encrypted calldata, returns tx hash
tx_hash = contract.write.setNumber(42)

# Shielded read — encrypted signed call, returns raw bytes
result = contract.read.getNumber()

# Transparent write — standard send_transaction, returns tx hash
tx_hash = contract.twrite.setNumber(42)

# Transparent read — standard eth_call, returns raw bytes
result = contract.tread.getNumber()

# Debug write — sends encrypted tx, returns plaintext + encrypted views + hash
debug = contract.dwrite.setNumber(42)
debug.plaintext_tx.data  # unencrypted calldata
debug.shielded_tx.data   # encrypted calldata
debug.tx_hash            # transaction hash
```

Write namespaces accept optional keyword arguments:

```python
tx_hash = contract.write.deposit(value=10**18, gas=100_000, gas_price=10**9)
```

## Async Support

```python
from seismic_web3 import create_async_shielded_web3, PrivateKey

# HTTP
w3 = await create_async_shielded_web3(
    "http://127.0.0.1:8545",
    private_key=PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX")),
)

# WebSocket (persistent connection, supports subscriptions)
w3 = await create_async_shielded_web3(
    "ws://127.0.0.1:8545",
    private_key=PrivateKey(bytes.fromhex("YOUR_PRIVATE_KEY_HEX")),
    use_websocket=True,
)

contract = w3.seismic.contract(address="0x...", abi=ABI)
tx_hash = await contract.write.setNumber(42)
result = await contract.read.getNumber()
```

## Chain Configuration

Pre-defined chain configs:

```python
from seismic_web3 import SEISMIC_TESTNET, SANVIL, make_seismic_testnet, ChainConfig

SEISMIC_TESTNET.rpc_url  # "https://gcp-1.seismictest.net/rpc"
SEISMIC_TESTNET.ws_url   # "wss://gcp-1.seismictest.net/ws"
SEISMIC_TESTNET.chain_id # 5124

SANVIL.rpc_url   # "http://127.0.0.1:8545"
SANVIL.chain_id  # 31337

# Factory for other GCP testnet instances
testnet_2 = make_seismic_testnet(2)  # gcp-2.seismictest.net

# Custom chain config
custom = ChainConfig(
    chain_id=1234,
    rpc_url="https://my-node.example.com/rpc",
    ws_url="wss://my-node.example.com/ws",
    name="My Network",
)
```

## Low-Level API

For cases where you need manual control over calldata (e.g. pre-encoded calldata, contract deployments, or non-ABI interactions), the `w3.seismic` namespace exposes two lower-level methods:

```python
from hexbytes import HexBytes

# Shielded transaction with raw calldata
tx_hash = w3.seismic.send_shielded_transaction(
    to="0x...",
    data=HexBytes("0x..."),  # pre-encoded calldata
    value=0,
    gas=100_000,
    gas_price=10**9,
)

# Signed read with raw calldata
result = w3.seismic.signed_call(
    to="0x...",
    data=HexBytes("0x..."),
    gas=30_000_000,
)

# Debug shielded transaction — sends + returns plaintext/encrypted views
debug = w3.seismic.debug_send_shielded_transaction(
    to="0x...",
    data=HexBytes("0x..."),
)
```

You can also encode calldata for shielded types separately:

```python
from seismic_web3.contract.abi import encode_shielded_calldata

# Encodes selector (using original shielded type names like suint256)
# and parameters (using standard types like uint256)
data = encode_shielded_calldata(abi, "setNumber", [42])
```

## SRC20 Tokens

The SDK ships with `SRC20_ABI`, the ABI for Seismic's [SRC20 token standard](https://docs.seismic.systems) — a privacy-preserving ERC20 where balances and transfer amounts use shielded types (`suint256`).

Key differences from ERC20:

- **`balanceOf()` takes no arguments** — the contract uses `msg.sender` internally, so you must use a **signed read** (`.read`, not `.tread`). A plain `eth_call` zeros out the `from` field, which means `msg.sender` would be `0x0` and return the wrong balance. A signed read sends a valid Seismic transaction (type `0x4a`) to the `eth_call` endpoint, proving your identity so the contract sees your real address.
- **Amounts are `suint256`** — transfer values, approvals, and balances are shielded. The SDK's `ShieldedContract` handles the type remapping automatically.

```python
from seismic_web3 import create_shielded_web3, SRC20_ABI, PrivateKey

w3 = create_shielded_web3("http://127.0.0.1:8545", private_key=PrivateKey(...))

token = w3.seismic.contract(address="0x...", abi=SRC20_ABI)

# Metadata — plain reads (no privacy needed, no msg.sender dependency)
name = token.tread.name()         # b"TestToken"
symbol = token.tread.symbol()     # b"TEST"
decimals = token.tread.decimals() # b'\x12' (18)

# Balance — MUST use signed read (.read), not .tread
# balanceOf() relies on msg.sender; a plain eth_call zeros the from field
raw = token.read.balanceOf()
balance = int.from_bytes(raw, "big")

# Transfer — shielded write (amount is suint256)
tx_hash = token.write.transfer("0xRecipient...", 100)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Approve — shielded write
tx_hash = token.write.approve("0xSpender...", 500)
```

## Deposit Contract

The SDK provides first-class action methods for Seismic's Eth2-style validator deposit contract. Validators deposit ETH (minimum 1 ETH, typically 32 ETH) along with their public keys, signatures, and withdrawal credentials.

### Direct actions (recommended)

Call deposit actions directly on `w3.seismic` — no ABI or contract instantiation needed:

```python
from seismic_web3 import (
    create_shielded_web3,
    compute_deposit_data_root,
    make_withdrawal_credentials,
    PrivateKey,
)

w3 = create_shielded_web3("http://127.0.0.1:8545", private_key=PrivateKey(...))

# Build withdrawal credentials from your Ethereum address
withdrawal_creds = make_withdrawal_credentials("0xYourAddress...")

# Compute the deposit data root (required by the contract)
amount_gwei = 32_000_000_000  # 32 ETH
deposit_data_root = compute_deposit_data_root(
    node_pubkey=node_pk,          # 32-byte ED25519 public key
    consensus_pubkey=consensus_pk, # 48-byte BLS12-381 public key
    withdrawal_credentials=withdrawal_creds,
    node_signature=node_sig,       # 64-byte ED25519 signature
    consensus_signature=consensus_sig, # 96-byte BLS12-381 signature
    amount_gwei=amount_gwei,
)

# Deposit 32 ETH
tx_hash = w3.seismic.deposit(
    node_pk, consensus_pk, withdrawal_creds,
    node_sig, consensus_sig, deposit_data_root,
    value=32 * 10**18,
)
receipt = w3.eth.wait_for_transaction_receipt(tx_hash)

# Read deposit count and root
count = w3.seismic.get_deposit_count()   # int
root = w3.seismic.get_deposit_root()     # bytes (32 bytes)
```

All three methods accept an optional `address` keyword to target a custom deployment instead of the genesis contract.

### Manual contract approach

For advanced use cases (custom ABI, event filtering), you can use the `ShieldedContract` pattern. The deposit contract uses **no shielded types** — use transparent namespaces (`.twrite` / `.tread`):

```python
from seismic_web3 import DEPOSIT_CONTRACT_ABI, DEPOSIT_CONTRACT_ADDRESS

deposit = w3.seismic.contract(
    address=DEPOSIT_CONTRACT_ADDRESS, abi=DEPOSIT_CONTRACT_ABI,
)

tx_hash = deposit.twrite.deposit(
    node_pk, consensus_pk, withdrawal_creds,
    node_sig, consensus_sig, deposit_data_root,
    value=32 * 10**18,
)
count_bytes = deposit.tread.get_deposit_count()   # raw bytes
root = deposit.tread.get_deposit_root()           # raw bytes
```

| Export | Kind | Description |
|--------|------|-------------|
| `DEPOSIT_CONTRACT_ABI` | const | Deposit contract ABI (4 functions + 1 event) |
| `DEPOSIT_CONTRACT_ADDRESS` | const | Genesis address (`0x00000000219ab540356cBB839Cbe05303d7705Fa`) |
| `compute_deposit_data_root` | function | Compute SHA-256 SSZ hash tree root for deposit data |
| `make_withdrawal_credentials` | function | Format an Ethereum address into 32-byte withdrawal credentials |

## Precompiles

Call Mercury EVM precompiles directly via `eth_call`. No encryption state needed — just a `Web3` instance connected to a Seismic node.

```python
from seismic_web3.precompiles import rng, ecdh, aes_gcm_encrypt, aes_gcm_decrypt, hkdf, secp256k1_sign
from seismic_web3 import Bytes32, PrivateKey, CompressedPublicKey

# On-chain random number generation (0x64)
random_val = rng(w3, num_bytes=32)

# On-chain ECDH key exchange (0x65)
shared_secret = ecdh(w3, sk=my_private_key, pk=their_public_key)

# On-chain AES-GCM encrypt/decrypt (0x66 / 0x67)
ciphertext = aes_gcm_encrypt(w3, aes_key=key, nonce=1, plaintext=b"secret")
plaintext = aes_gcm_decrypt(w3, aes_key=key, nonce=1, ciphertext=bytes(ciphertext))

# On-chain HKDF key derivation (0x68)
derived_key = hkdf(w3, b"input key material")

# On-chain secp256k1 signing (0x69)
signature = secp256k1_sign(w3, sk=my_private_key, message="hello")
```

All functions have async variants (`async_rng`, `async_ecdh`, etc.).

| Precompile | Address | Function | Returns |
|---|---|---|---|
| RNG | `0x64` | `rng(w3, num_bytes=, pers=)` | `int` |
| ECDH | `0x65` | `ecdh(w3, sk=, pk=)` | `Bytes32` |
| AES Encrypt | `0x66` | `aes_gcm_encrypt(w3, aes_key=, nonce=, plaintext=)` | `HexBytes` |
| AES Decrypt | `0x67` | `aes_gcm_decrypt(w3, aes_key=, nonce=, ciphertext=)` | `HexBytes` |
| HKDF | `0x68` | `hkdf(w3, ikm)` | `Bytes32` |
| secp256k1 Sign | `0x69` | `secp256k1_sign(w3, sk=, message=)` | `HexBytes` |

## Security Parameters

Override per-transaction security defaults with `SeismicSecurityParams`:

```python
from seismic_web3 import SeismicSecurityParams

params = SeismicSecurityParams(
    blocks_window=50,          # Tx expires after 50 blocks (default: 100)
    encryption_nonce=None,     # Random nonce (default)
    recent_block_hash=None,    # Use latest block (default)
    expires_at_block=None,     # Computed from blocks_window (default)
)

tx_hash = contract.write.setNumber(42, security=params)
result = contract.read.getNumber(security=params)
```

## All Exports

Everything importable from `seismic_web3`:

| Export | Kind | Description |
|--------|------|-------------|
| `create_shielded_web3` | function | Create sync `Web3` with `w3.seismic` namespace |
| `create_async_shielded_web3` | function | Create async `AsyncWeb3` with `w3.seismic` namespace |
| `get_encryption` | function | Derive `EncryptionState` from a TEE public key |
| `make_seismic_testnet` | function | Factory for GCP testnet `ChainConfig` |
| `compute_deposit_data_root` | function | Compute SHA-256 deposit data root hash |
| `make_withdrawal_credentials` | function | Build 32-byte withdrawal credentials from address |
| `ShieldedContract` | class | Sync contract with `.write`/`.read`/`.twrite`/`.tread`/`.dwrite` |
| `AsyncShieldedContract` | class | Async version of `ShieldedContract` |
| `SeismicNamespace` | class | Sync namespace (`w3.seismic`) — includes `deposit`, `get_deposit_root`, `get_deposit_count` |
| `AsyncSeismicNamespace` | class | Async namespace (`w3.seismic`) — async versions of all methods |
| `EncryptionState` | class | ECDH-derived AES key + keypair |
| `ChainConfig` | class | Immutable chain configuration (chain_id, rpc_url, ws_url) |
| `SeismicSecurityParams` | class | Per-transaction security overrides |
| `SRC20_ABI` | const | ISRC20 interface ABI (7 functions) |
| `DEPOSIT_CONTRACT_ABI` | const | Deposit contract ABI (4 functions + 1 event) |
| `DEPOSIT_CONTRACT_ADDRESS` | const | Deposit contract genesis address |
| `SEISMIC_TESTNET` | const | Default testnet config (chain 5124) |
| `SANVIL` | const | Local sanvil config (chain 31337) |
| `SEISMIC_TX_TYPE` | const | Transaction type byte (`0x4a` / `74`) |
| `PrivateKey` | type | 32-byte secp256k1 private key |
| `Bytes32` | type | 32-byte value (hashes, AES keys) |
| `CompressedPublicKey` | type | 33-byte compressed secp256k1 public key |
| `EncryptionNonce` | type | 12-byte AES-GCM nonce |
| `SeismicElements` | type | Seismic-specific tx fields (encryption params, expiry) |
| `UnsignedSeismicTx` | type | Complete unsigned `TxSeismic` |
| `TxSeismicMetadata` | type | Transaction metadata (used as AES-GCM AAD) |
| `LegacyFields` | type | Standard EVM tx fields (chain_id, nonce, to, value) |
| `Signature` | type | ECDSA signature components (v, r, s) |
| `PlaintextTx` | type | Unencrypted transaction view (for debug writes) |
| `DebugWriteResult` | type | Result from `.dwrite` (plaintext tx + shielded tx + hash) |

---

## Development

### Prerequisites

- **Python 3.10+**
- **[uv](https://docs.astral.sh/uv/)** — install with `curl -LsSf https://astral.sh/uv/install.sh | sh`

### Commands

| Command | Description |
|---------|-------------|
| `make install` | Install all dependencies into venv |
| `make fmt` | Format code with ruff |
| `make fmt-check` | Check formatting without changes |
| `make lint` | Run ruff linter |
| `make typecheck` | Run ty type checker |
| `make test` | Run unit tests (no node required) |
| `make test-integration-anvil` | Run integration tests against sanvil |
| `make test-integration-reth` | Run integration tests against seismic-reth |
| `make test-all` | Run all tests (unit + integration) |
| `make ci` | Run all CI checks (fmt-check + lint + typecheck + unit tests) |

### Running CI Locally

```bash
./ci.sh                    # Full CI: lint, typecheck, unit + integration tests
./ci.sh --no-anvil         # Skip anvil integration tests
./ci.sh --no-reth          # Skip reth integration tests
./ci.sh --no-integration   # Skip all integration tests (unit only)
```

### Publishing to PyPI

```bash
# Build
uv build

# Publish (requires authentication)
uv publish

# Or with token
uv publish --token <PYPI_TOKEN>
```

### Project Structure

```
clients/py/
├── pyproject.toml
├── Makefile
├── ci.sh
├── .python-version
├── src/
│   └── seismic_web3/
│       ├── __init__.py              # Public API exports
│       ├── _types.py                # PrivateKey, Bytes32, etc.
│       ├── chains.py                # ChainConfig, SEISMIC_TESTNET, SANVIL
│       ├── client.py                # create_shielded_web3, EncryptionState
│       ├── module.py                # SeismicNamespace (w3.seismic)
│       ├── transaction_types.py     # SeismicSecurityParams, TxSeismic types
│       ├── py.typed                 # PEP 561 type marker
│       ├── abis/
│       │   ├── __init__.py           # Re-exports ABI constants
│       │   ├── deposit_contract.py   # Deposit contract ABI + helpers
│       │   └── src20.py              # ISRC20 interface ABI
│       ├── contract/
│       │   ├── abi.py               # ABI encoding, shielded type remapping
│       │   └── shielded.py          # ShieldedContract (5-namespace pattern)
│       ├── crypto/
│       │   ├── aes.py               # AES-GCM encryption
│       │   ├── ecdh.py              # ECDH key agreement
│       │   ├── nonce.py             # Nonce generation
│       │   └── secp.py              # secp256k1 utilities
│       ├── precompiles/
│       │   ├── _base.py             # Precompile framework (call, gas helpers)
│       │   ├── rng.py               # RNG precompile (0x64)
│       │   ├── ecdh.py              # ECDH precompile (0x65)
│       │   ├── aes.py               # AES encrypt/decrypt (0x66, 0x67)
│       │   ├── hkdf.py              # HKDF precompile (0x68)
│       │   └── secp256k1.py         # secp256k1 sign precompile (0x69)
│       └── transaction/
│           ├── aead.py              # AAD construction
│           ├── metadata.py          # Transaction metadata
│           ├── send.py              # send_shielded_transaction, signed_call
│           └── serialize.py         # RLP serialization
└── tests/
    ├── test_abi.py
    ├── test_chains.py
    ├── test_client.py
    ├── test_contract.py
    ├── test_crypto.py
    ├── test_encryption.py
    ├── test_module.py
    ├── test_rpc.py
    ├── test_send.py
    ├── test_serialize.py
    ├── test_transaction_types.py
    ├── test_precompiles.py
    ├── test_types.py
    ├── test_deposit_helpers.py
    └── integration/
        ├── conftest.py
        ├── contracts.py
        ├── artifacts/                # Compiled contract JSON
        ├── test_client_factory.py
        ├── test_namespace.py
        ├── test_precompiles.py
        ├── test_seismic_counter.py
        ├── test_deposit_contract.py
        ├── test_src20_token.py
        └── test_transparent_counter.py
```
