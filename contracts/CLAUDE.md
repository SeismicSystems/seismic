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

### Build with library linking (for genesis artifacts)

```bash
sforge build --libraries lib/AesLib.sol:AesLib:0x1000000000000000000000000000000000000003
```

### Verify

```bash
sforge build
# Expected: "Compiler run successful with warnings:"
# Warning (3805) about pre-release compiler is expected and safe to ignore.
```

## Test

```bash
# All tests (74 pass, 1 known failure — see Troubleshooting)
sforge test -vv

# Skip the known-failing Intelligence test
sforge test -vv --match-contract 'Directory|ProtocolParams|DepositContract|ShieldedDelegation'

# Single test suite
sforge test -vv --match-contract DepositContractTest

# Single test function
sforge test -vv --match-test test_SuccessfulDeposit

# Verbose trace on failure
sforge test -vvvv --match-contract IntelligenceTest
```

### Test suites

| Suite                             | Tests           | Status                                                           |
| --------------------------------- | --------------- | ---------------------------------------------------------------- |
| `DepositContract.t.sol`           | 26 (incl. fuzz) | Pass                                                             |
| `ProtocolParams.t.sol`            | 34 (incl. fuzz) | Pass                                                             |
| `ShieldedDelegationAccount.t.sol` | 12              | Pass                                                             |
| `Directory.t.sol`                 | 2               | Pass                                                             |
| `Intelligence.t.sol`              | 1 (setUp)       | **Fail** — requires Directory at genesis address `0x1000...0004` |

## Scripts

```bash
# Sync compiled artifacts to artifacts/ (builds first, copies JSON ABIs)
bash script/sync-artifacts.sh
```

`script/genesis-contracts.txt` lists the 8 contracts included in genesis artifacts.

## Project Layout

```
src/
  directory/             Encrypted key directory (AES-256-GCM via precompiles)
    Directory.sol          Stores per-user encryption keys using suint256 (shielded)
    IDirectory.sol
  intelligence/          Provider encryption management
    Intelligence.sol       Encrypts data to a list of providers via Directory
    IIntelligence.sol
  enclave/               Enclave upgrade governance
    UpgradeOperator.sol    Manages enclave defining attributes (MRTD, PCR registers)
    MultisigUpgradeOperator.sol   2-of-3 multisig wrapper for UpgradeOperator
  seismic-std-lib/       Seismic standard library (reusable contracts)
    ProtocolParams.sol     Owner-managed key-value parameter store (IDs 0-255)
    DepositContract.sol    Eth2-style validator deposit contract (Merkle tree, SHA-256)
    session-keys/
      ShieldedDelegationAccount.sol   EIP-7702 delegation with session keys (P256/WebAuthn/Secp256k1)
      interfaces/IShieldedDelegationAccount.sol
    utils/
      EIP7702Utils.sol     Signature verification for multiple key types
      MultiSend.sol        Batch call execution (from Safe)
      SRC20.sol            Privacy-preserving ERC20 with shielded balances
      TestToken.sol        Simple test token extending SRC20
      precompiles/CryptoUtils.sol   RNG (0x64), AES encrypt (0x66), AES decrypt (0x67) precompile wrappers
lib/
  AesLib.sol             AES-256-GCM + HKDF library using precompiles (0x66, 0x67, 0x68)
  forge-std/             Foundry test framework (submodule)
  openzeppelin-contracts/  OpenZeppelin v5.4.0 (submodule)
  solady/                Solady v0.1.26 — P256, WebAuthn, SignatureChecker (submodule)
test/                    Foundry tests (*.t.sol)
script/
  sync-artifacts.sh      Build + copy JSON artifacts for genesis contracts
  genesis-contracts.txt  List of genesis contract names
artifacts/               Pre-built JSON ABI artifacts (8 contracts)
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
- `sforge fmt` is available but formatting is not strictly enforced
- 4-space indentation in Solidity
- Test files use `Test.t.sol` naming convention
- Test functions: `test_CamelCase()` for unit tests, `testFuzz_CamelCase()` for fuzz tests, `test_RevertWhen_*()` for failure cases

## Troubleshooting

| Problem                                                                        | Fix                                                                                                                                                                                                                                                                             |
| ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `forge: command not found`                                                     | Use `sforge` (at `~/.seismic/bin/sforge`), not `forge`. This is Seismic's Foundry fork.                                                                                                                                                                                         |
| `Warning (3805): pre-release compiler version`                                 | Expected. Seismic's ssolc compiler is a pre-release fork. Safe to ignore.                                                                                                                                                                                                       |
| `Intelligence.t.sol` setUp fails: `call to non-contract address 0x1000...0004` | Known issue. The Intelligence contract calls Directory at a hardcoded genesis address that doesn't exist in the test EVM. Requires Seismic-specific test environment or mocking. Skip with `--match-contract 'Directory\|ProtocolParams\|DepositContract\|ShieldedDelegation'`. |
| Submodules empty (`lib/forge-std/` has no files)                               | Run `git submodule update --init --recursive`.                                                                                                                                                                                                                                  |
| `sforge build` recompiles everything                                           | Normal on first build (45 files). Subsequent builds are incremental.                                                                                                                                                                                                            |
| Artifact sync fails: `not found in out/`                                       | Run `sforge build` before `bash script/sync-artifacts.sh`, or just run the script (it builds first).                                                                                                                                                                            |
