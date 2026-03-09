# Seismic — LLM Context

Seismic is a privacy-enabled EVM blockchain. This file gives you the context to work effectively across the Seismic codebase, which spans multiple repos.

- For user-facing docs: https://docs.seismic.systems/llms-full.txt

## Key Concepts

See [seismic/docs/glossary.md](seismic/docs/glossary.md) for full definitions. Quick summary:

- **FlaggedStorage** — `(value: U256, is_private: bool)` tuple replacing `U256` for all storage values. Private slots return 0 via RPC; only `CLOAD`/`CSTORE` opcodes can access them.
- **Shielded Types** — `suint`, `sint`, `sbool`, `saddress` compile to `CLOAD`/`CSTORE` instead of `SLOAD`/`SSTORE`.
- **Mercury Spec** — modified EVM: CLOAD/CSTORE opcodes + 6 precompiles (RNG, ECDH, AES-GCM, HKDF, secp256k1 Sign).
- **TxSeismic** — transaction type `74` with encrypted calldata (ECDH + AEAD).
- **TEE Integration** — nodes run in a TEE; calldata decrypted at RPC layer before EVM execution.

## Docs

Detailed docs live in the [seismic](seismic/) monorepo:

- [seismic/docs/architecture.md](seismic/docs/architecture.md) — diagrams: Seismic node, RPC/EVM/storage interactions, tries + SeismicTx
- [seismic/docs/glossary.md](seismic/docs/glossary.md) — key concepts: FlaggedStorage, TxSeismic, Mercury Spec, SeismicHost
- [seismic/docs/language-and-vm.md](seismic/docs/language-and-vm.md) — Mercury EVM spec: shielded types, CLOAD/CSTORE, FlaggedStorage, arrays, casting
- [seismic/docs/repos.md](seismic/docs/repos.md) — all repos, fork management, dependency flow

When working in a specific repo, also check that repo's README and CLAUDE.md, as well as anything under that repo's `docs/seismic` directory.

## Workspace Layout

All repos live as siblings under the parent directory. Open `seismic/workspace/seismic.code-workspace` in VS Code for full multi-repo navigation.

```
seismic/                          # parent directory
├── CLAUDE.md                     # symlink -> seismic/workspace/CLAUDE.md
├── seismic/                      # monorepo (docs, scripts, workspace config)
│   ├── workspace/                # cross-repo workspace files (source of truth)
│   ├── contracts/                # Solidity contracts
│   ├── clients/ts/               # TypeScript client (Viem + React)
│   └── clients/py/               # Python client (Web3.py) 
├── seismic-reth/                 # execution client (fork of reth)
├── seismic-evm/                  # block execution layer (fork of alloy-evm)
├── seismic-revm/                 # Mercury EVM (fork of revm)
├── seismic-revm-inspectors/      # EVM tracing (fork of revm-inspectors)
├── seismic-alloy/                # Rust SDK: TxSeismic, providers
├── seismic-alloy-core/           # primitives: FlaggedStorage, shielded types (fork of alloy-core)
├── seismic-trie/                 # Merkle trie for FlaggedStorage (fork of alloy-trie)
├── seismic-foundry/              # dev tools: sforge, sanvil, scast (fork of foundry)
├── seismic-foundry-fork-db/      # fork DB with FlaggedStorage (fork of foundry-fork-db)
├── seismic-compilers/            # compiler integration for sforge (fork of foundry-compilers)
├── enclave/                      # TEE enclave server and contracts
├── seismic-solidity/             # Solidity compiler with shielded types (fork of solidity)
```

## Working Across Repos

- **Building**: All Rust repos use `cargo build`. seismic-reth and seismic-foundry produce binaries (`seismic-reth`, `sforge`, `sanvil`, `scast`).
- **Testing**: Most repos use `cargo nextest run --workspace` or `cargo test --workspace`. seismic-alloy tests require `sanvil` in PATH.
- **Formatting**: `cargo +nightly fmt --all` across all repos.
- **Linting**: `RUSTFLAGS="-D warnings" cargo clippy --workspace --all-features`.
- **Fork management**: All forks pin upstream commits. Dependency versions are coordinated across repos via `[patch]` sections in `Cargo.toml`.
