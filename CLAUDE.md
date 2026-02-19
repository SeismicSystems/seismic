# Seismic Monorepo

This is seismic's monorepo, targeting Seismic developers and contributors. It contains documentation, scripts, and the VS Code workspace file.

## Code Style

- **No imports inside functions.** All imports must be at the top of the file. Never use deferred/lazy imports inside function or method bodies.
- **Trailing commas.** When a function call or definition spans multiple lines, use trailing commas so each item ends up on its own line. Single-line calls like `func(x=1, y=2)` or definitions like `def func(x: int, y: str) -> int:` are fine without trailing commas.

## Docs in This Repo

- [docs/architecture.md](docs/architecture.md) — diagrams: Seismic node, RPC/EVM/storage interactions, tries + SeismicTx
- [docs/glossary.md](docs/glossary.md) — key concepts: FlaggedStorage, TxSeismic, Mercury Spec, SeismicHost
- [docs/language-and-vm.md](docs/language-and-vm.md) — Mercury EVM spec: shielded types, CLOAD/CSTORE, FlaggedStorage, arrays, casting
- [docs/repos.md](docs/repos.md) — all repos, fork management, dependency flow
- [docs/claude-code-setup.md](docs/claude-code-setup.md) — Claude Code skills setup and symlink instructions
