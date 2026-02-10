# Seismic — LLM Context

Seismic is a privacy-enabled EVM blockchain. This file gives you the context to work effectively across the Seismic codebase, which spans multiple repos.

- This repo is seismic's monorepo, targeting Seismic developers and contributors.
- For user-facing docs: https://docs.seismic.systems/llms-full.txt

## Key Concepts

See [docs/glossary.md](docs/glossary.md) for full definitions. Quick summary:

- **FlaggedStorage** — `(value: U256, is_private: bool)` tuple replacing `U256` for all storage values. Private slots return 0 via RPC; only `CLOAD`/`CSTORE` opcodes can access them.
- **Shielded Types** — `suint`, `sint`, `sbool`, `saddress` compile to `CLOAD`/`CSTORE` instead of `SLOAD`/`SSTORE`.
- **Mercury Spec** — modified EVM: CLOAD/CSTORE opcodes + 6 precompiles (RNG, ECDH, AES-GCM, HKDF, secp256k1 Sign).
- **TxSeismic** — transaction type `74` with encrypted calldata (ECDH + AEAD).
- **TEE Integration** — nodes run in a TEE; calldata decrypted at RPC layer before EVM execution.

## Docs in This Repo

- [docs/architecture.md](docs/architecture.md) — diagrams: Seismic node, RPC/EVM/storage interactions, tries + SeismicTx
- [docs/glossary.md](docs/glossary.md) — key concepts: FlaggedStorage, TxSeismic, Mercury Spec, SeismicHost
- [docs/language-and-vm.md](docs/language-and-vm.md) — Mercury EVM spec: shielded types, CLOAD/CSTORE, FlaggedStorage, arrays, casting
- [docs/repos.md](docs/repos.md) — all repos, fork management, dependency flow

When working in a specific repo, also check that repo's README and CLAUDE.md, as well as anything under that repo's `docs/seismic` directory.

## Working Across Repos

- **Building**: All Rust repos use `cargo build`. seismic-reth and seismic-foundry produce binaries (`seismic-reth`, `sforge`, `sanvil`, `scast`).
- **Testing**: Most repos use `cargo nextest run --workspace` or `cargo test --workspace`. seismic-alloy tests require `sanvil` in PATH.
- **Formatting**: `cargo +nightly fmt --all` across all repos.
- **Linting**: `RUSTFLAGS="-D warnings" cargo clippy --workspace --all-features`.
- **Fork management**: All forks pin upstream commits. Dependency versions are coordinated across repos via `[patch]` sections in `Cargo.toml`.
