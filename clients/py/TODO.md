# seismic-web3 TODO

Deferred features not included in the initial SDK implementation.

## Block Explorer URL Helpers

- Generate URLs for addresses, transactions, tokens, blocks
- Support for SeismicScan explorer

## Faucet Interaction

- Request testnet ETH from the Seismic faucet

## Stricter ABIs Type

- Add a stricter type for ABIs instead of `list[dict[str, Any]]`

## ETH/Wei/Gwei Conversion Helpers

- Check if web3.py already exposes convenient methods for converting between ETH, wei, and gwei (e.g. `w3.to_wei`, `w3.from_wei`) and document or re-export them
