# Python SDK Documentation - Work in Progress

**Branch**: `py-docs-richer`
**Goal**: Complete documentation for all 58 public API items in the Seismic Python SDK

## Current Status

### Completed (35/58 items - 60%)

✅ **Phase 1: API Reference** (20 files)
- Types: Bytes32, PrivateKey, CompressedPublicKey, EncryptionNonce
- Transaction Types: Signature, SeismicElements, UnsignedSeismicTx, TxSeismicMetadata, etc.
- EIP-712 Functions: sign_seismic_tx_eip712, domain_separator, struct_hash, etc.

✅ **Phase 2: Client Documentation** (7 files)
- Factory functions: create_wallet_client, create_async_wallet_client, create_public_client, create_async_public_client
- EncryptionState class
- get_encryption helper

✅ **Phase 4: Chains Documentation** (6 files)
- ChainConfig, SEISMIC_TESTNET, SANVIL, make_seismic_testnet, SEISMIC_TX_TYPE

✅ **Phase 5: Precompiles Documentation** (7 files)
- rng, ecdh, aes_gcm_encrypt, aes_gcm_decrypt, hkdf, secp256k1_sign

✅ **Phase 3 (Partial): Contract Namespaces** (6 files - just completed)
- .write, .read, .twrite, .tread, .dwrite namespaces

### In Progress / Remaining (23/58 items - 40%)

🔄 **Phase 3: Contract Classes** (Need 4 files)
- `contract/shielded-contract.md` - ShieldedContract class
- `contract/async-shielded-contract.md` - AsyncShieldedContract class
- `contract/public-contract.md` - PublicContract class
- `contract/async-public-contract.md` - AsyncPublicContract class

🔄 **Phase 3: w3.seismic Namespace Classes** (Need 4 files)
- `namespaces/seismic-namespace.md` - SeismicNamespace (wallet, sync)
- `namespaces/async-seismic-namespace.md` - AsyncSeismicNamespace (wallet, async)
- `namespaces/seismic-public-namespace.md` - SeismicPublicNamespace (public, sync)
- `namespaces/async-seismic-public-namespace.md` - AsyncSeismicPublicNamespace (public, async)

🔄 **Phase 3: Namespace Methods** (Need 7 files)
- `namespaces/methods/get-tee-public-key.md`
- `namespaces/methods/send-shielded-transaction.md`
- `namespaces/methods/debug-send-shielded-transaction.md`
- `namespaces/methods/signed-call.md`
- `namespaces/methods/get-deposit-root.md`
- `namespaces/methods/get-deposit-count.md`
- `namespaces/methods/deposit.md`

🔄 **Phase 6: SRC20 Documentation** (Need 9 files)
- Event watching: watch_src20_events, watch_src20_events_with_key, SRC20EventWatcher, AsyncSRC20EventWatcher
- Directory: register_viewing_key, get_viewing_key, check_has_key
- Types: DecryptedTransferLog, DecryptedApprovalLog

🔄 **Phase 6: ABIs Documentation** (Need 5 files)
- `abis/src20-abi.md`
- `abis/deposit-contract.md` (ABI + ADDRESS)
- `abis/directory.md` (ABI + ADDRESS)
- `abis/compute-deposit-data-root.md`
- `abis/make-withdrawal-credentials.md`

🔄 **Phase 7: Examples** (Need 5 files)
- `examples/basic-wallet-setup.md`
- `examples/shielded-write-complete.md`
- `examples/signed-read-pattern.md`
- `examples/src20-workflow.md`
- `examples/async-patterns.md`

## Key Source Files

- **Main exports**: `clients/py/src/seismic_web3/__init__.py`
- **Namespaces**: `clients/py/src/seismic_web3/module.py`
- **Contracts**: `clients/py/src/seismic_web3/contract/shielded.py`, `public.py`
- **SRC20**: `clients/py/src/seismic_web3/src20/__init__.py`
- **ABIs**: `clients/py/src/seismic_web3/abis/`

## Documentation Structure

```
docs/gitbook/clients/python/
├── README.md (updated with navigation)
├── api-reference/
│   ├── types/ (4 files ✅)
│   ├── transaction-types/ (8 files ✅)
│   └── eip712/ (5 files ✅)
├── client/ (7 files ✅)
├── chains/ (6 files ✅)
├── precompiles/ (7 files ✅)
├── contract/
│   ├── namespaces/ (6 files ✅)
│   └── [4 contract class files needed]
├── namespaces/
│   ├── [4 namespace class files needed]
│   └── methods/ [7 method files needed]
├── src20/ [9 files needed]
├── abis/ [5 files needed]
├── guides/ (existing guides moved ✅)
└── examples/ [5 files needed]
```

## Recent Commits

- `df33691` - docs(client/py): expand Python SDK documentation to 48+ pages
- [Contract namespaces documentation just completed, not yet committed]

## Next Steps

1. Complete contract class documentation (4 files)
2. Complete namespace class and methods documentation (11 files)
3. Complete SRC20 documentation (9 files)
4. Complete ABIs documentation (5 files)
5. Complete examples (5 files)
6. Final cross-reference verification
7. Commit completion

## Template Pattern

Each doc file follows this structure:
- Front matter with `description` and `icon`
- Function signature
- Parameters table
- Returns section
- Both sync and async examples
- Warnings/security considerations
- Related links

## Navigation

All documentation is indexed in:
- `docs/gitbook/SUMMARY.md` - GitBook navigation
- `docs/gitbook/clients/python/README.md` - Main Python SDK landing page
