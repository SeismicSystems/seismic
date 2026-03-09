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
│   ├── workspace/                # cross-repo workspace files (source of truth)
│   └── contracts/                # Solidity contracts
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
└── seismic-solidity/             # Solidity compiler with shielded types (fork of solidity)
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

### Dev Env

We use mise to manage our dev environment, including dependency version management and running common tasks. It is not mandatory to use mise, but we recommend it for ease of use and consistency. Start by [installing mise](https://mise.jdx.dev/getting-started.html), and then run `mise install`. This will install the latest version of the Seismic Dev Environment, which includes all the tools you need to build and test Seismic (sforge, sanvil, scast, ssolc, etc.). It will also set up your PATH so that you can run these tools from anywhere inside the repo. Make sure to take a look at comments inside the various `mise.toml` files across the repos, and run `mise run` from anywhere to see the tasks available in that subdir.

![](assets/mise-demo.gif)

### Docs

If working with any specific fork/repo, make sure to read that repo's README and CLAUDE.md for specific context. This repo's docs are meant to provide the big picture and cross-cutting concepts that apply across all repos:
- **[Architecture](docs/architecture.md)** — high-level diagrams of how the components fit together
- **[Glossary](docs/glossary.md)** — key Seismic concepts: FlaggedStorage, TxSeismic, Mercury Spec, SeismicHost
- **[Language & VM](docs/language-and-vm.md)** — Mercury EVM spec: shielded types, CLOAD/CSTORE, FlaggedStorage
- **[Repos](docs/repos.md)** — stack diagram, repos dependency chain, how changes flow
- **[Claude Code Setup](docs/claude-code-setup.md)** — Claude Code skills setup and symlink instructions
