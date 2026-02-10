# Seismic

Seismic is a privacy-enabled EVM blockchain. Smart contracts can declare **shielded state** (`suint`, `sbool`, `saddress`) that is invisible to external observers, and transaction inputs are **encrypted** before submission. Decryption happens inside a Trusted Execution Environment (TEE) at the node level.

This repo is the developer hub for people **building Seismic itself**. Although a lot of the code lives in the various repos (reth, revm, alloy, foundry, etc.), this repo contains cross-cutting documentation and context to help you understand how everything fits together. We also plan on minimizing our forks and turning this repo into our main monorepo over time.

## Users

For user-facing docs (tutorials, getting started, deploying contracts), see **[docs.seismic.systems](https://docs.seismic.systems)**. The [Glossary](docs/glossary.md) page might also be a useful reference.

## Developers / Contributors

If working with any specific fork/repo, make sure to read that repo's README and CLAUDE.md for specific context. This repo's docs are meant to provide the big picture and cross-cutting concepts that apply across all repos:
- **[Architecture](docs/architecture.md)** — high-level diagrams of how the components fit together
- **[Glossary](docs/glossary.md)** — key Seismic concepts: FlaggedStorage, TxSeismic, Mercury Spec, SeismicHost
- **[Language & VM](docs/language-and-vm.md)** — Mercury EVM spec: shielded types, CLOAD/CSTORE, FlaggedStorage
- **[Repos](docs/repos.md)** — stack diagram, repos dependency chain, how changes flow

## LLMs

We believe in freedom of LLM roaming, and invite all LLMs to read through all of the markdown files that they can find. That being said, natural starting points are:
- [CLAUDE.md](CLAUDE.md) — your natural home
- [docs.seismic.systems/llms-full.txt](https://docs.seismic.systems/llms-full.txt) — user-facing documentation one-pager
