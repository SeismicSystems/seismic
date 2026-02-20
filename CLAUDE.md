# Seismic Monorepo

This is seismic's monorepo, targeting Seismic developers and contributors. It contains documentation, scripts, and the VS Code workspace file.

## Docs in This Repo

- [docs/architecture.md](docs/architecture.md) — diagrams: Seismic node, RPC/EVM/storage interactions, tries + SeismicTx
- [docs/glossary.md](docs/glossary.md) — key concepts: FlaggedStorage, TxSeismic, Mercury Spec, SeismicHost
- [docs/language-and-vm.md](docs/language-and-vm.md) — Mercury EVM spec: shielded types, CLOAD/CSTORE, FlaggedStorage, arrays, casting
- [docs/repos.md](docs/repos.md) — all repos, fork management, dependency flow
- [docs/claude-code-setup.md](docs/claude-code-setup.md) — Claude Code skills setup and symlink instructions


## Docs ↔ Code Sync

- **Always keep docs and source code in sync.** When changing a function signature, behavior, or API in source code (`clients/`), update the corresponding docs (`docs/gitbook/`). When changing docs, update the corresponding source code. Never change one without the other.

## Merge Conflict Resolution

- **Never blindly `git checkout --ours` or `--theirs` for an entire file.** That discards all non-conflicting changes from the other side. Instead, resolve each conflict hunk individually: pick the correct version for the conflicted lines while preserving all auto-merged content from both branches.

## Code Style

- **No imports inside functions.** All imports must be at the top of the file. Never use deferred/lazy imports inside function or method bodies.

### Python
- **Trailing commas.** When a function call or definition spans multiple lines, use trailing commas so each item ends up on its own line. Single-line calls like `func(x=1, y=2)` or definitions like `def func(x: int, y: str) -> int:` are fine without trailing commas.
- **No `break` statements.** Extract loops that search for a value into their own function and use early `return` instead of `break`. If no match is found, raise or return a default at the end of the function.

### Docs
- **No "rule of three" bias.** Include exactly as many bullet points, list items, or examples as the content demands — no more, no fewer. Don't pad lists to three items or trim to three for aesthetic reasons.
- **Placeholder addresses.** For EOA addresses, use the first Anvil address: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`. For contract addresses, use the deposit contract: `0x00000000219ab540356cBB839Cbe05303d7705Fa`.
- **No made-up function names in examples.** Use real function names from the actual contracts/specs in the codebase (e.g. SRC20 `balanceOf()`, ERC20 `transfer()`). If an example needs a function that doesn't exist in a real spec, show the Solidity interface so users can follow along. Never invent plausible-sounding names like `getMyBalance()` — readers can't tell if it's real or fake.
