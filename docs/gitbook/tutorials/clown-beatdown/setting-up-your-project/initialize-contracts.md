# Initialize contracts

1. **Navigate to the contracts subdirectory:**

```bash
cd packages/contracts
```

2. **Initialize a project with `sforge`:**

```bash
sforge init --no-commit && rm -rf .github
```

This command will:

- Create the contract project structure (e.g., `src/`, `test/`, `foundry.toml`).
- Automatically install the Forge standard library (`forge-std`) as a submodule.
- Remove the `.github` workflow folder (not required)

3. Edit the `.gitignore` file to be the following:

```bash
.env
broadcast/
cache/
```

4. Delete the default contract, test and script files (`Counter.sol` and `Counter.t.sol` and `Counter.s.sol`) and replace them with their `ClownBeatdown` counterparts (`ClownBeatdown.sol`, `ClownBeatdown.t.sol` and `ClownBeatdown.s.sol`):

```bash
# Remove the Counter files
rm -f src/Counter.sol test/Counter.t.sol script/Counter.s.sol

# Create empty ClownBeatdown files in the same locations
touch src/ClownBeatdown.sol test/ClownBeatdown.t.sol script/ClownBeatdown.s.sol
```

These files are empty for now, but we will add to them as we go along.
