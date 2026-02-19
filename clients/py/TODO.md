# seismic-web3 TODO

Deferred features not included in the initial SDK implementation.

## Precompiles

Mercury EVM precompile wrappers (all on-chain, require a running node):

- **RNG** (`0x64`) — on-chain random number generation
- **ECDH** (`0x65`) — on-chain elliptic-curve Diffie-Hellman
- **AES-GCM Encrypt** (`0x66`) — on-chain AES-256-GCM encryption
- **AES-GCM Decrypt** (`0x67`) — on-chain AES-256-GCM decryption
- **HKDF** (`0x68`) — on-chain HKDF-SHA256 key derivation
- **secp256k1 Sign** (`0x69`) — on-chain ECDSA signing

All precompile addresses, gas costs, and parameter encoding are documented in
`seismic-client/packages/seismic-viem/src/precompiles/`.

## EIP-712 Typed Data Signing

- `messageVersion=2` for JSON-RPC wallet accounts (MetaMask, WalletConnect)
- Domain: `{name: "Seismic Transaction", version: "2"}`
- Reference: `seismic-viem/src/signSeismicTypedData.ts`

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
