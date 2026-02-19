# seismic-web3 TODO

Deferred features not included in the initial SDK implementation.

## ~~Precompiles~~ (Done)

Implemented in `seismic_web3.precompiles` — see `src/seismic_web3/precompiles/`.

## ~~EIP-712 Typed Data Signing~~ (Done)

Implemented in `seismic_web3.transaction.eip712` — see `src/seismic_web3/transaction/eip712.py`.

## Deposit Contract Actions

- Deposit 32 ETH for validator staking
- Read deposit root and deposit count
- ABI in `seismic-viem/src/abis/depositContract.ts`

## SRC20 Token Standard

- [x] Token standard with encrypted balances — `SRC20_ABI` constant + integration tests
- [x] Transfer, approve, balanceOf with shielded types — works via `ShieldedContract`
- [ ] Event watching with viewing keys (requires Directory/Intelligence genesis contracts)

## Block Explorer URL Helpers

- Generate URLs for addresses, transactions, tokens, blocks
- Support for SeismicScan explorer

## Faucet Interaction

- Request testnet ETH from the Seismic faucet
