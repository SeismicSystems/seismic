# Development

## Prerequisites

- **Python 3.10+**
- **[uv](https://docs.astral.sh/uv/)** — install with `curl -LsSf https://astral.sh/uv/install.sh | sh`

## Environment Variables for Integration Tests

Integration tests start a local sanvil or seismic-reth node and need to know where the binaries are. Set **one** of the following:

| Variable | Description |
|----------|-------------|
| `SEISMIC_WORKSPACE` | Parent directory containing `seismic-foundry/` and `seismic-reth/` repos |
| `SFOUNDRY_ROOT` | Path to seismic-foundry repo root (overrides `SEISMIC_WORKSPACE` for sanvil) |
| `SRETH_ROOT` | Path to seismic-reth repo root (overrides `SEISMIC_WORKSPACE` for reth) |

The binaries are resolved as:
- **sanvil**: `$SFOUNDRY_ROOT/target/debug/sanvil` or `$SEISMIC_WORKSPACE/seismic-foundry/target/debug/sanvil`
- **seismic-reth**: `$SRETH_ROOT/target/debug/seismic-reth` or `$SEISMIC_WORKSPACE/seismic-reth/target/debug/seismic-reth`

Quick setup:

```bash
# Option 1: Set workspace root (works for both)
export SEISMIC_WORKSPACE=~/code/seismic-workspace

# Option 2: Set individual roots
export SFOUNDRY_ROOT=~/code/seismic-workspace/seismic-foundry
export SRETH_ROOT=~/code/seismic-workspace/seismic-reth
```

See `.env.example` for a template you can copy to `.env`.

## Commands

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

## Running CI Locally

```bash
./ci.sh                    # Full CI: lint, typecheck, unit + integration tests
./ci.sh --no-anvil         # Skip anvil integration tests
./ci.sh --no-reth          # Skip reth integration tests
./ci.sh --no-integration   # Skip all integration tests (unit only)
```

## Publishing to PyPI

```bash
# Build
uv build

# Publish (requires authentication)
uv publish

# Or with token
uv publish --token <PYPI_TOKEN>
```

## Project Structure

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
│       │   ├── __init__.py           # Re-exports SRC20_ABI, DIRECTORY_ABI
│       │   ├── deposit_contract.py   # Deposit contract ABI + helpers
│       │   ├── directory.py          # Directory genesis contract ABI
│       │   └── src20.py              # ISRC20 interface ABI + events
│       ├── contract/
│       │   ├── abi.py               # ABI encoding, shielded type remapping
│       │   └── shielded.py          # ShieldedContract (5-namespace pattern)
│       ├── crypto/
│       │   ├── aes.py               # AES-GCM encryption
│       │   ├── ecdh.py              # ECDH key agreement
│       │   ├── nonce.py             # Nonce generation
│       │   └── secp.py              # secp256k1 utilities
│       ├── src20/
│       │   ├── __init__.py           # Re-exports public API
│       │   ├── types.py              # DecryptedTransferLog, DecryptedApprovalLog
│       │   ├── crypto.py             # AES-GCM decryption of encrypted amounts
│       │   ├── directory.py          # Directory contract helpers (viewing keys)
│       │   └── watch.py              # SRC20EventWatcher, factory functions
│       ├── precompiles/
│       │   ├── _base.py             # Precompile framework (call, gas helpers)
│       │   ├── rng.py               # RNG precompile (0x64)
│       │   ├── ecdh.py              # ECDH precompile (0x65)
│       │   ├── aes.py               # AES encrypt/decrypt (0x66, 0x67)
│       │   ├── hkdf.py              # HKDF precompile (0x68)
│       │   └── secp256k1.py         # secp256k1 sign precompile (0x69)
│       └── transaction/
│           ├── aead.py              # AAD construction
│           ├── eip712.py            # EIP-712 typed data signing
│           ├── metadata.py          # Transaction metadata
│           ├── send.py              # send_shielded_transaction, signed_call
│           └── serialize.py         # RLP serialization
└── tests/
    ├── test_abi.py
    ├── test_chains.py
    ├── test_client.py
    ├── test_contract.py
    ├── test_crypto.py
    ├── test_eip712.py
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
        ├── test_deposit_contract.py
        ├── test_directory.py
        ├── test_client_factory.py
        ├── test_namespace.py
        ├── test_precompiles.py
        ├── test_seismic_counter.py
        ├── test_src20_events.py
        ├── test_src20_token.py
        └── test_transparent_counter.py
```
