---
name: ssolc-tests
description: Build and test seismic-solidity locally — cmake build, soltest.sh Boost unit tests, revme semantic tests via seismic-revm, and isoltest for syntax test expectations. Use when running, filtering, or debugging solidity/ssolc tests, semantic tests, or updating test expectations.
---

# Running the Test Suite Locally

Instructions for building and testing seismic-solidity.

## Building the Code

From the repository root:

```bash
mkdir -p build && cd build
cmake ..
make -j$(nproc)
```

This produces the compiler binary at `<solidity-repo-root>/build/solc/solc` (note: the binary is named `solc`, not `ssolc`).

## Running soltest.sh

The `soltest.sh` script runs the Boost C++ unit tests (excluding semantic tests):

```bash
<solidity-repo-root>/scripts/soltest.sh
```

You can filter tests with `-t`:

```bash
<solidity-repo-root>/scripts/soltest.sh -t YulOptimiser
```

## Running Semantic Tests

Semantic tests use the seismic-revm fork (not evmone). They must be run from `<workspace-root>/seismic-revm`, where `<workspace-root>` is the parent directory containing all seismic repos as siblings (the same directory that holds `seismic/`, `seismic-revm/`, `seismic-solidity/`, etc.).

**Important:** Replace `<solidity-repo-root>` below with the absolute path to the seismic-solidity repo you are working in (e.g., for git worktrees, use the worktree path). Replace `<workspace-root>` with the absolute path to your seismic workspace directory.

Note: `--unsafe-via-ir` bypasses a restriction in Seismic Solidity that prevents compiling `--via-ir` or `--experimental-via-ir`. It does not run all the tests via-ir necessarily. See `--help` on the `semantics` revme subcommand for details.

Base command:

```bash
cd <workspace-root>/seismic-revm && cargo run -p revme -- semantics \
  --keep-going --unsafe-via-ir \
  -s "<solidity-repo-root>/build/solc/solc" \
  -t "<solidity-repo-root>/test/libsolidity/semanticTests"
```

Run all four configurations by adding flags to the base command:

| Configuration | Extra flags |
|---|---|
| No optimizer, no via-ir | (none) |
| Optimizer, no via-ir | `--optimize --optimizer-runs 200` |
| No optimizer, via-ir | `--via-ir` |
| Optimizer, via-ir | `--via-ir --optimize --optimizer-runs 200` |

## Running isoltest

`isoltest` is the interactive tool for managing syntax/analysis test expectations. Build it with `make -j$(nproc) isoltest` from the build directory.

**Always** pass `--no-semantic-tests` — semantic tests are run via the seismic-revm `revme` binary, not isoltest.

```bash
# Run specific test(s)
<solidity-repo-root>/build/test/tools/isoltest --no-semantic-tests -t "syntaxTests/types/shielded_*"

# Run all syntax tests
<solidity-repo-root>/build/test/tools/isoltest --no-semantic-tests -t "syntaxTests/*"
```

### Auto-updating expectations (`--accept-updates`)

You may use `--accept-updates` to batch-fix test expectations, but **you must warn the user loudly before doing so**. This flag silently rewrites every failing test's expected output to match the obtained output — if any of those new expectations are wrong, they will hide real bugs. **Never run `--accept-updates` without explicit user approval.** When proposing it, tell the user:

> **WARNING:** `--accept-updates` will automatically overwrite all failing test expectations. You MUST carefully review every change it produces (via `git diff`) before committing. Blindly accepting updates can mask regressions.

```bash
<solidity-repo-root>/build/test/tools/isoltest --no-semantic-tests --accept-updates -t "syntaxTests/types/shielded_*"
```

## Notes

- The compiler binary is `solc` (not `ssolc`) inside `build/solc/`
- Semantic test runs require `--unsafe-via-ir` (see above); via-IR pipeline re-enablement work lives on the seismic-revm branch `ci-via-ir-support`
- Some tests may only fail with the optimizer enabled, others only with it disabled — always test both configurations when debugging issues
