# Seismic Standard Library Contracts

Seismic's standard library contracts — SRC20 token standard, interfaces, and multicall utilities. These are reusable library contracts intended to be imported by other repos building on the Seismic network.

## Contracts

- **SRC20.sol** — Abstract ERC20 with confidential (shielded) balances and transfers, EIP-2612 permit support, encrypted event emission via Directory/Intelligence precompiles, and signed balance reads (`balanceOfSigned`).
- **SRC20Token.sol** / **SRC20Factory.sol** — Concrete owner-minted SRC20 and the factory that deploys it.
- **SRC20Multicall.sol** — Batch reader for SRC20 shielded balances across multiple tokens using `balanceOfSigned`.
- **interfaces/** — `ISRC20`, `IDirectory`, `IIntelligence`, `IShieldedDelegationAccount`.
- **DepositContract.sol** — Eth2-style validator deposit contract (Merkle tree, SHA-256).
- **ProtocolParams.sol** — Owner-managed key-value parameter store (IDs 0-255).
- **ShieldedDelegationAccount.sol** — EIP-7702 delegation with session keys (P256/WebAuthn/Secp256k1); signature verification and the MultiSend batch executor are inlined.
- **utils/precompiles/CryptoUtils.sol** — RNG, AES-GCM and HKDF precompile wrappers.

## Origin

SRC20, ISRC20, and SRC20Multicall are sourced from the [src20 repo](https://github.com/SeismicSystems/src20) (`packages/contracts/src/`).
