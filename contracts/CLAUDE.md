# Seismic Contracts

On-chain smart contracts for the [Seismic network](https://seismic.systems) — a privacy-preserving blockchain platform. These contracts handle validator deposits, encrypted communication, enclave upgrade governance, session key management, and protocol parameters. They use Seismic-specific shielded types (`suint256`) and cryptographic precompiles (AES-256-GCM, HKDF, RNG) that only exist on the Seismic EVM.

## Build

Foundry project using **`sforge`** (Seismic's fork of Foundry). Standard `forge` is not installed; always use `sforge`.

### macOS (arm64/x86_64)

```bash
# Install sforge via Seismic's toolchain (must already be at ~/.seismic/bin/sforge)
# Dependencies: git (for submodules), jq (optional, for artifact formatting)
brew install jq  # optional

# Initialize submodules (required on fresh clone)
git submodule update --init --recursive

# Build
sforge build
```

### Linux (Ubuntu/Debian)

```bash
sudo apt-get update && sudo apt-get install -y git jq
git submodule update --init --recursive
sforge build
```

### via-ir mode

Some contracts (e.g., `DepositContract.sol`) hit "stack too deep" errors with the default compiler pipeline. To build everything, use `--via-ir --unsafe-via-ir`:

```bash
sforge build --via-ir --unsafe-via-ir
sforge test --via-ir --unsafe-via-ir
```

The `foundry.toml` has `via_ir = false` by default because the via-ir pipeline is experimental in ssolc. Always pass the flags explicitly when building or testing.

### Verify

```bash
sforge build --via-ir --unsafe-via-ir
# Expected: "Compiler run successful with warnings:"
# Warning (3805) about pre-release compiler is expected and safe to ignore.
```

## Test

```bash
# All tests (212 pass across 18 suites)
sforge test -vv

# Single test suite
sforge test -vv --match-contract DepositContractTest

# Single test function
sforge test -vv --match-test test_SuccessfulDeposit

# Verbose trace on failure
sforge test -vvvv --match-contract IntelligenceTest
```

### Test suites

One `*.t.sol` per contract under `test/`, plus regression suites for audit
findings (`DirectoryNonceReuse.t.sol`, `IntelligenceZeroKeyLeak.t.sol`).
Tests that need the genesis predeploys (`Directory`, `Intelligence`) install
them with `deployCodeTo` at their fixed addresses.

## Scripts

```bash
# Sync compiled artifacts to artifacts/ (builds first, copies JSON ABIs)
bash script/sync-artifacts.sh
```

`script/genesis-contracts.txt` lists the contracts included in genesis artifacts.

## Project Layout

```
src/
  directory/             Encrypted key directory (AES-256-GCM via precompiles)
    Directory.sol          Stores per-user encryption keys using suint256 (shielded)
    IDirectory.sol
  intelligence/          Provider encryption management
    Intelligence.sol       Encrypts data to a list of providers via Directory
    IIntelligence.sol
  enclave/               TEE measurement admission
    MeasurementRegistry.sol       Admission status of compiled measurement IDs (genesis predeploy)
    MeasurementAuthorityDev.sol   Dev-only single-owner authority for MeasurementRegistry
    UpgradeOperator.sol           Legacy measurement store (not used by the network)
    MultisigUpgradeOperator.sol   2-of-3 multisig wrapper for UpgradeOperator
  examples/              SeismicCounter, TransparentCounter, WrappedNativeTokenSrc20
  seismic-std-lib/       Seismic standard library (reusable contracts)
    ProtocolParams.sol     Owner-managed key-value parameter store (IDs 0-255)
    DepositContract.sol    Eth2-style validator deposit contract (Merkle tree, SHA-256)
    SRC20.sol              Privacy-preserving ERC20 with shielded balances
    SRC20Token.sol         Concrete SRC20 deployed by SRC20Factory.sol
    SRC20Multicall.sol     Batch signed balance reads
    ShieldedDelegationAccount.sol   EIP-7702 delegation with session keys (P256/WebAuthn/Secp256k1)
    interfaces/            IDirectory, IIntelligence, ISRC20, IShieldedDelegationAccount
    utils/precompiles/CryptoUtils.sol   RNG (0x64), AES encrypt (0x66), AES decrypt (0x67), HKDF (0x68) wrappers
lib/
  forge-std/             Foundry test framework (submodule)
  openzeppelin-contracts/  OpenZeppelin v5.4.0 (submodule)
  solady/                Solady v0.1.26 — P256, WebAuthn, SignatureChecker (submodule)
test/                    Foundry tests (*.t.sol)
script/
  sync-artifacts.sh      Build + copy JSON artifacts for genesis contracts
  genesis-contracts.txt  List of genesis contract names
artifacts/               Pre-built JSON ABI artifacts
```

## Dependencies

Managed as git submodules in `lib/` plus import remappings in `foundry.toml`:

| Dependency             | Remapping          | Version                        |
| ---------------------- | ------------------ | ------------------------------ |
| forge-std              | `forge-std/`       | `8e40513`                      |
| openzeppelin-contracts | `@openzeppelin/`   | v5.4.0                         |
| solady                 | `solady/`          | v0.1.26                        |
| seismic-std-lib        | `seismic-std-lib/` | local (`src/seismic-std-lib/`) |

## Key Architectural Patterns

- **Shielded types**: `suint256` variables use confidential storage (`CSTORE`/`CLOAD` opcodes). Only available on Seismic EVM.
- **Precompiles**: Crypto operations at fixed addresses — RNG (`0x64`), AES encrypt (`0x66`), AES decrypt (`0x67`), HKDF (`0x68`).
- **Genesis addresses**: Several contracts are deployed at fixed genesis addresses (e.g., UpgradeOperator at `0x1000...0001`, Directory at `0x1000...0004`). The Intelligence contract hardcodes these.
- **`via_ir = true`**: All compilation goes through the Yul IR pipeline (set in `foundry.toml`).
- **EIP-7702**: ShieldedDelegationAccount uses custom storage slot layout via assembly to avoid collision with delegated accounts.

## Code Style

- Solidity `^0.8.13` minimum (some files use `^0.8.20`)
- **Always run `sforge fmt` before committing/pushing.** Formatting is enforced.
- 4-space indentation in Solidity
- Test files use `Test.t.sol` naming convention
- Test functions: `test_CamelCase()` for unit tests, `testFuzz_CamelCase()` for fuzz tests, `test_RevertWhen_*()` for failure cases

## Troubleshooting

| Problem                                                                        | Fix                                                                                                                                                                                                                                                                             |
| ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `forge: command not found`                                                     | Use `sforge` (at `~/.seismic/bin/sforge`), not `forge`. This is Seismic's Foundry fork.                                                                                                                                                                                         |
| `Warning (3805): pre-release compiler version`                                 | Expected. Seismic's ssolc compiler is a pre-release fork. Safe to ignore.                                                                                                                                                                                                       |
| Submodules empty (`lib/forge-std/` has no files)                               | Run `git submodule update --init --recursive`.                                                                                                                                                                                                                                  |
| `sforge build` recompiles everything                                           | Normal on first build (45 files). Subsequent builds are incremental.                                                                                                                                                                                                            |
| Artifact sync fails: `not found in out/`                                       | Run `sforge build` before `bash script/sync-artifacts.sh`, or just run the script (it builds first).                                                                                                                                                                            |
