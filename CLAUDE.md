# Seismic Monorepo

This is seismic's monorepo, targeting Seismic developers and contributors.

**Workspace context**: if the workspace file isn't already in your context, read [workspace/CLAUDE.workspace.md](workspace/CLAUDE.workspace.md) for key concepts, the repo map, and cross-repo conventions.

## Repo Layout

```
seismic/
├── docs/            # developer docs + gitbook (user-facing) — see docs/README.md for the index
├── contracts/       # Solidity contracts
├── clients/ts/      # TypeScript client (Viem + React)
├── clients/py/      # Python client (Web3.py)
└── workspace/       # cross-repo workspace files: CLAUDE.workspace.md, seismic.code-workspace, cargo-local-patches.toml
```

## Docs ↔ Code Sync

- **Always keep docs and source code in sync.** When changing a function signature, behavior, or API in source code (`clients/`), update the corresponding docs (`docs/gitbook/`). When changing docs, update the corresponding source code. Never change one without the other.

## Merge Conflict Resolution

- **Never blindly `git checkout --ours` or `--theirs` for an entire file.** That discards all non-conflicting changes from the other side. Instead, resolve each conflict hunk individually: pick the correct version for the conflicted lines while preserving all auto-merged content from both branches.

## Code Style

- **No imports inside functions.** All imports must be at the top of the file. Never use deferred/lazy imports inside function or method bodies.

### Python
- **Trailing commas.** When a function call or definition spans multiple lines, use trailing commas so each item ends up on its own line. Single-line calls like `func(x=1, y=2)` or definitions like `def func(x: int, y: str) -> int:` are fine without trailing commas.
- **No `break` statements.** Extract loops that search for a value into their own function and use early `return` instead of `break`. If no match is found, raise or return a default at the end of the function.
- **Lint and format before committing.** Run `uv run ruff check src/ tests/ && uv run ruff format src/ tests/` from `clients/py/` before every commit. The CI runs both `ruff check` and `ruff format --check` with an 88-character line limit.
- **Test before committing.** When adding or modifying code, run `uv run pytest tests/` (unit tests) from `clients/py/` before committing. For integration tests, run `uv run pytest tests/integration/ -v` against a running node.

### TypeScript
- **Lint before committing.** Run `bun run lint:check` from `clients/ts/` and fix all errors before every commit.
- **Test before committing.** When adding or modifying code, run `bun run viem:test` from `clients/ts/` against a running node (sanvil or sreth) before committing.

### Docs
- **No "rule of three" bias.** Include exactly as many bullet points, list items, or examples as the content demands — no more, no fewer. Don't pad lists to three items or trim to three for aesthetic reasons.
- **Placeholder addresses.** For EOA addresses, use the first Anvil address: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`. For generic contract addresses where the reader will substitute their own, use a descriptive string like `"0xYourTokenAddress"`. Only use the deposit contract address `0x00000000219ab540356cBB839Cbe05303d7705Fa` when actually referring to the deposit contract.
- **No made-up function names in examples.** Use real function names from the actual contracts/specs in the codebase (e.g. SRC20 `balanceOf()`, ERC20 `transfer()`). If an example needs a function that doesn't exist in a real spec, show the Solidity interface so users can follow along. Never invent plausible-sounding names like `getMyBalance()` — readers can't tell if it's real or fake.
