# seismic-web3 TODO

Deferred features not included in the initial SDK implementation.

## ~~Precompiles~~ (Done)

Implemented in `seismic_web3.precompiles` — see `src/seismic_web3/precompiles/`.

## EIP-712 Typed Data Signing

- `messageVersion=2` for JSON-RPC wallet accounts (MetaMask, WalletConnect)
- Domain: `{name: "Seismic Transaction", version: "2"}`
- Reference: `seismic-viem/src/signSeismicTypedData.ts`

## Debug Write (dwrite)

- Returns both plaintext and encrypted transaction + hash
- Useful for debugging encrypted calldata

## Deposit Contract Actions

- Deposit 32 ETH for validator staking
- Read deposit root and deposit count
- ABI in `seismic-viem/src/abis/depositContract.ts`

## SRC20 Token Standard

- Token standard with encrypted balances
- Event watching with viewing keys
- Transfer, approve, balanceOf with shielded types

## Block Explorer URL Helpers

- Generate URLs for addresses, transactions, tokens, blocks
- Support for SeismicScan explorer

## Faucet Interaction

- Request testnet ETH from the Seismic faucet
