# Seismic Standard Library Contracts

Seismic's standard library contracts — SRC20 token standard, interfaces, and multicall utilities. These are reusable library contracts intended to be imported by other repos building on the Seismic network.

## Contracts

- **SRC20.sol** — Abstract ERC20 with confidential (shielded) balances and transfers, EIP-2612 permit support, encrypted event emission via the Directory/Intelligence predeploys, and signed balance reads (`balanceOfSigned`).
- **SRC20Token.sol** / **SRC20Factory.sol** — Concrete SRC20 token and its factory.
- **SRC20Multicall.sol** — Batch reader for SRC20 shielded balances across multiple tokens using `balanceOfSigned`.
- **interfaces/** — `ISRC20`, plus the interfaces of the predeploys dApps call (`IDirectory`, `IIntelligence`, `IShieldedDelegationAccount`). Their implementations live in `../predeploys/`, since dApps call the genesis instances rather than deploy their own.
- **utils/** — Tx-context precompile helpers (`TxUtils`) and cryptographic precompile wrappers (`precompiles/CryptoUtils`).

## Origin

SRC20, ISRC20, and SRC20Multicall are sourced from the [src20 repo](https://github.com/SeismicSystems/src20) (`packages/contracts/src/`).
