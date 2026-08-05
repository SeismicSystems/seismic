# Seismic — LLM Context

Seismic is a privacy-enabled EVM blockchain. This file gives you the context to work effectively across the Seismic codebase, which spans multiple repos.

- For user-facing docs: https://docs.seismic.systems/llms-full.txt

## Key Concepts

See `seismic/docs/glossary.md` for full definitions. Quick summary:

- **FlaggedStorage** — `(value: U256, is_private: bool)` tuple replacing `U256` for all storage values. Private slots return 0 via RPC; only `CLOAD`/`CSTORE` opcodes can access them.
- **Shielded Types** — `suint`, `sint`, `sbool`, `saddress` compile to `CLOAD`/`CSTORE` instead of `SLOAD`/`SSTORE`.
- **Mercury Spec** — modified EVM: CLOAD/CSTORE opcodes + 6 precompiles (RNG, ECDH, AES-GCM, HKDF, secp256k1 Sign).
- **TxSeismic** — transaction type `74` with encrypted calldata (ECDH + AEAD).
- **TEE Integration** — nodes run in a TEE. Encrypted calldata is decrypted in-enclave at execution time: in the **block executor** for state-changing txs (so txs stay encrypted in the mempool and over gossip), and at the **RPC layer** for reads (`eth_call`/signed reads).

## Docs

Detailed docs live in `seismic/docs/` — read its README for the index (architecture, key schedule, Mercury EVM spec, network founding, diagram conventions, repo/fork management).

Diagrams: Mermaid first; hand-authored structured SVG only when spatial layout carries meaning. Read `seismic/docs/tee/diagrams/README.md` before creating or editing any diagram.

When working in a specific repo, also check that repo's README and CLAUDE.md/AGENTS.md, as well as anything under that repo's `docs/seismic` directory.

## Workspace Layout

All repos live as siblings under the parent directory. Open `seismic/workspace/seismic.code-workspace` in VS Code for full multi-repo navigation.

```
seismic/                          # parent directory
├── CLAUDE.md                     # symlink -> seismic/workspace/CLAUDE.workspace.md
├── seismic/                      # monorepo: docs, Solidity contracts, TS/Python clients, workspace config
├── seismic-reth/                 # execution client (fork of reth)
├── summit/                       # consensus client
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
├── seismic-images/               # TDX confidential VM images (fork of flashbots-images)
├── deploy/                       # network deployment tooling (Pulumi + tee CLIs)
├── seismic-solidity/             # Solidity compiler with shielded types (fork of solidity)
```

## Working Across Repos

- **Building**: All Rust repos use `cargo build`. seismic-reth and seismic-foundry produce binaries (`seismic-reth`, `sforge`, `sanvil`, `scast`).
- **Testing**: Most repos use `cargo nextest run --workspace` or `cargo test --workspace`. seismic-alloy tests require `sanvil` in PATH.
- **Formatting**: `cargo +nightly fmt --all` across all repos.
- **Linting**: `RUSTFLAGS="-D warnings" cargo clippy --workspace --all-features`.
- **Fork management**: All forks pin upstream commits. Dependency versions are coordinated across repos via `[patch]` sections in `Cargo.toml`.
- **Cross-repo verification**: `mise run cargo::local-patches::on` (from `seismic/`) makes all sibling repos resolve Seismic dependencies from local checkouts instead of pinned commits; `::off` reverts, `::status` shows current state. Never hand-edit pins.
- **Cross-repo file references** (code comments, commits, PR text): use the full GitHub URL so readers can click through — default branch normally, a pinned commit SHA for historical references. Same-repo references stay relative paths.
