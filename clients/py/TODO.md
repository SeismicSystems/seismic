# seismic-web3 TODO

Deferred features not included in the initial SDK implementation.

## EIP-712 Typed Data Signing

- `messageVersion=2` for JSON-RPC wallet accounts (MetaMask, WalletConnect)
- Domain: `{name: "Seismic Transaction", version: "2"}`
- Reference: `seismic-viem/src/signSeismicTypedData.ts`

## Block Explorer URL Helpers

- Generate URLs for addresses, transactions, tokens, blocks
- Support for SeismicScan explorer

## Faucet Interaction

- Request testnet ETH from the Seismic faucet
