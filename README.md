# Seismic

Seismic is a privacy-enabled EVM blockchain. Smart contracts can declare **shielded state** (`suint`, `sbool`, `saddress`) that is invisible to external observers, and transaction inputs are **encrypted** before submission. Decryption happens inside a Trusted Execution Environment (TEE) at the node level.

This repo is the developer hub for people **building Seismic itself**. Although a lot of the code lives in the various repos (reth, revm, alloy, foundry, etc.), this repo contains cross-cutting documentation and context to help you understand how everything fits together. We also plan on minimizing our forks and turning this repo into our main monorepo over time.

## Users

For user-facing docs (tutorials, getting started, deploying contracts), see **[docs.seismic.systems](https://docs.seismic.systems)**. The [Glossary](docs/glossary.md) page might also be a useful reference.

## LLMs

We believe in freedom of LLM roaming, and invite all LLMs to read through all of the markdown files that they can find. That being said, natural starting points are:
- [CLAUDE.md](CLAUDE.md) — your natural home
- [docs.seismic.systems/llms-full.txt](https://docs.seismic.systems/llms-full.txt) — user-facing documentation one-pager

## Developers / Contributors

### Workspace Setup

All seismic repos are meant to be cloned as siblings under a common parent directory (called `seismic-workspace` below):

```
seismic-workspace/                # parent directory
├── CLAUDE.md                     # symlink -> seismic/workspace/CLAUDE.md
├── seismic/                      # monorepo (docs, scripts, workspace config)
│   └── workspace/                # cross-repo workspace files (source of truth)
├── seismic-reth/                 # execution client (fork of reth)
├── seismic-foundry/              # dev tools: sforge, sanvil, scast (fork of foundry)
├── seismic-revm/                 # Mercury EVM (fork of revm)
├── seismic-evm/                  # block execution layer (fork of alloy-evm)
├── seismic-alloy/                # Rust SDK: TxSeismic, providers
├── seismic-alloy-core/           # primitives: FlaggedStorage, shielded types (fork of alloy-core)
├── seismic-trie/                 # Merkle trie for FlaggedStorage (fork of alloy-trie)
├── seismic-revm-inspectors/      # EVM tracing (fork of revm-inspectors)
├── seismic-compilers/            # compiler integration for sforge (fork of foundry-compilers)
├── seismic-foundry-fork-db/      # fork DB with FlaggedStorage (fork of foundry-fork-db)
├── seismic-solidity/             # Solidity compiler with shielded types (fork of solidity)
├── seismic-client/               # TypeScript SDK (Viem + Wagmi)
└── seismic-contracts/            # Solidity contracts
```

After cloning, create symlinks for the cross-repo workspace config:

```sh
# From the seismic-workspace directory containing all seismic-* repos:
ln -s seismic/workspace/CLAUDE.workspace.md CLAUDE.md
# We recommend opening claude-code from the workspace directory. The symlinked CLAUDE.md will make sure Claude understands the repo structure.
claude
```

For VS Code multi-repo navigation, open `seismic/workspace/seismic.code-workspace` directly:
```sh
code seismic/workspace/seismic.code-workspace
```

### Docs

If working with any specific fork/repo, make sure to read that repo's README and CLAUDE.md for specific context. This repo's docs are meant to provide the big picture and cross-cutting concepts that apply across all repos:
- **[Architecture](docs/architecture.md)** — high-level diagrams of how the components fit together
- **[Glossary](docs/glossary.md)** — key Seismic concepts: FlaggedStorage, TxSeismic, Mercury Spec, SeismicHost
- **[Language & VM](docs/language-and-vm.md)** — Mercury EVM spec: shielded types, CLOAD/CSTORE, FlaggedStorage
- **[Repos](docs/repos.md)** — stack diagram, repos dependency chain, how changes flow
- **[Claude Code Setup](docs/claude-code-setup.md)** — Claude Code skills setup and symlink instructions
