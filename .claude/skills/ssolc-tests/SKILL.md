# Running the Test Suite Locally

Instructions for building and testing seismic-solidity.

## Building the Code

From the repository root:

```bash
mkdir -p build && cd build
cmake ..
make -j$(nproc)
```

This produces the compiler binary at `<repo-root>/build/solc/solc` (note: the binary is named `solc`, not `ssolc`).

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

Semantic tests use the seismic-revm fork (not evmone). They must be run from `~/code/seismic-workspace/seismic-revm`. See the below sections for the specific commands

**Important:** Replace `<repo-root>` below with the absolute path to the seismic-solidity repo you are working in (e.g., for git worktrees, use the worktree path).

Note: the --unsafe-via-ir command will allow us to bypass a restriction in Seismic Solidity that prevents compiling --via-ir or --experimental-via-ir. It does not run all the tests --via-ir necessarily. You can read a more detailed writeup of this argument by calling --help on the `semantics` revme subcommand

### Without Optimizer, without --via-ir

```bash
cd ~/code/seismic-workspace/seismic-revm && cargo run -p revme -- semantics \
  --keep-going --unsafe-via-ir \
  -s "<solidity-repo-root>/build/solc/solc" \
  -t "<solidity-repo-root>/test/libsolidity/semanticTests"
```

### With Optimizer, without --via-ir

```bash
cd ~/code/seismic-workspace/seismic-revm && cargo run -p revme -- semantics \
  --keep-going --unsafe-via-ir \
  --optimize --optimizer-runs 200 \
  -s "<repo-root>/build/solc/solc" \
  -t "<repo-root>/test/libsolidity/semanticTests"
```

### Without Optimizer, with --via-ir

```bash
cd ~/code/seismic-workspace/seismic-revm && cargo run -p revme -- semantics \
  --keep-going --unsafe-via-ir --via-ir \
  -s "<solidity-repo-root>/build/solc/solc" \
  -t "<solidity-repo-root>/test/libsolidity/semanticTests"
```

### With Optimizer, with --via-ir

```bash
cd ~/code/seismic-workspace/seismic-revm && cargo run -p revme -- semantics \
  --keep-going --unsafe-via-ir --via-ir \
  --optimize --optimizer-runs 200 \
  -s "<repo-root>/build/solc/solc" \
  -t "<repo-root>/test/libsolidity/semanticTests"
```


## Running isoltest

`isoltest` is the interactive tool for managing syntax/analysis test expectations. Build it with `make -j$(nproc) isoltest` from the build directory.

**Always** pass `--no-semantic-tests` — semantic tests are run via the seismic-revm `revme` binary, not isoltest.

```bash
# Run specific test(s)
<repo-root>/build/test/tools/isoltest --no-semantic-tests -t "syntaxTests/types/shielded_*"

# Run all syntax tests
<repo-root>/build/test/tools/isoltest --no-semantic-tests -t "syntaxTests/*"
```

### Auto-updating expectations (`--accept-updates`)

You may use `--accept-updates` to batch-fix test expectations, but **you must warn the user loudly before doing so**. This flag silently rewrites every failing test's expected output to match the obtained output — if any of those new expectations are wrong, they will hide real bugs. **Never run `--accept-updates` without explicit user approval.** When proposing it, tell the user:

> **WARNING:** `--accept-updates` will automatically overwrite all failing test expectations. You MUST carefully review every change it produces (via `git diff`) before committing. Blindly accepting updates can mask regressions.

```bash
<repo-root>/build/test/tools/isoltest --no-semantic-tests --accept-updates -t "syntaxTests/types/shielded_*"
```

## Notes

- The compiler binary is `solc` (not `ssolc`) inside `build/solc/`
- `--skip-via-ir` is required until the via-IR pipeline is re-enabled. You can find this on the `seismic-revm` branch called `ci-via-ir-support`
- Some tests may only fail with the optimizer enabled
- Some tests may only fail with the optimizer disabled
- Always test both configurations when debugging issues
