# Seismic Standard Library Contracts

Seismic's standard library contracts — SRC20 token standard, interfaces, and multicall utilities. These are reusable library contracts intended to be imported by other repos building on the Seismic network.

## Contracts

- **SRC20.sol** — Abstract ERC20 with confidential (shielded) balances and transfers, EIP-2612 permit support, encrypted event emission via Directory/Intelligence precompiles, and signed balance reads (`balanceOfSigned`).
- **SRC20Multicall.sol** — Batch reader for SRC20 shielded balances across multiple tokens using `balanceOfSigned`.
- **interfaces/ISRC20.sol** — Minimal interface for the SRC20 `balanceOfSigned` function.
- **DepositContract.sol** — Eth2-style validator deposit contract (Merkle tree, SHA-256).
- **ProtocolParams.sol** — Owner-managed key-value parameter store (IDs 0-255).
- **session-keys/** — EIP-7702 delegation with session keys (P256/WebAuthn/Secp256k1).
- **utils/** — EIP7702 signature verification, MultiSend batch calls, and cryptographic precompile wrappers.

## Origin

SRC20, ISRC20, and SRC20Multicall are sourced from the [src20 repo](https://github.com/SeismicSystems/src20) (`packages/contracts/src/`).
