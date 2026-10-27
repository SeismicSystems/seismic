# Seismic Client

Seismic's typescript client. This repository contains:

- [seismic-viem](packages/seismic-viem), a set of extensions for [viem](https://viem.sh/)
- [seismic-react](packages/seismic-react), a set of React hooks compatible with [wagmi](https://wagmi.sh/)

## Docs

View the docs [here](https://client.seismic.systems)

## Running tests

### Managed local nodes

From this directory, run `mise run test::integration::sanvil` or
`mise run test::integration::sreth` to start a node, run the suite, and stop it.
The reth task uses the shared [`../test-genesis/reth.json`](../test-genesis/reth.json),
which funds the SDK's Anvil account zero from block 0. Production `--chain dev`
does not fund public Anvil keys. See the [test-genesis notes](../test-genesis/README.md)
for provenance and refresh instructions.

`SRETH_BIN` selects a prebuilt reth binary (CI uses nightly); without it, the task
builds the sibling seismic-reth checkout. Direct `mise run sreth::start-local`
still defaults to production `--chain dev`; to use it for SDK tests, pass the
absolute path to `clients/test-genesis/reth.json` with `--chain`.

Sanvil is unchanged. For an externally managed node, fund the test signer before
running `bun run viem:test`.

### Legacy process helpers: `.env` file

Create a `.env` file in the repo root. These variables are only used for running tests. This file should have:

- `SRETH_ROOT`: the path to your local `seismic-reth` directory

Optionally, you may set these variables to specify where `reth` will store its state:

- `RETH_DATA_DIR`: absolute path for all reth files & subdirectories
- `RETH_STATIC_FILES`: absolute path to store static files

See `.env.example` for more information
