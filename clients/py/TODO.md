# seismic-web3 TODO

Deferred features not included in the initial SDK implementation.

## EIP-712 Typed Data Signing

- `messageVersion=2` for JSON-RPC wallet accounts (MetaMask, WalletConnect)
- Domain: `{name: "Seismic Transaction", version: "2"}`
- Reference: `seismic-viem/src/signSeismicTypedData.ts`

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
