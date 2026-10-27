# SDK test genesis

`reth.json` is the shared execution-layer genesis for the Python and TypeScript
SDK integration tests. It is **not** a network founding template.

It is copied from [seismic-reth's dev genesis at
`c4c048c153ce9043debed81a2efa6e6196eca19d`](https://github.com/SeismicSystems/seismic-reth/blob/c4c048c153ce9043debed81a2efa6e6196eca19d/crates/seismic/chainspec/res/genesis/dev.json),
with exactly one additional allocation: Anvil account zero,
`0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`, receives 1,000,000 native tokens
(`0xD3C21BCECCEDA1000000` wei). All production allocations, predeploy code,
storage, and chain settings are preserved.

Production `--chain dev` deliberately does not fund public Anvil keys. `--dev`
enables auto-mining; it does not restore those balances. Tests use
`--dev --chain <this file>` so their public signing key is funded from block 0,
including historical native-balance assertions. The resulting genesis hash is
intentionally different from the production dev hash; no Summit process or
consensus-genesis hash pin is used by these SDK tests.

## Consumers

- Python's `tests/integration/conftest.py` resolves this file relative to itself,
  independently of the working directory.
- The root `pitchfork.toml` passes this file to `sreth::start-local` for the
  TypeScript `test::integration::sreth` task. Direct `sreth::start-local` keeps
  its production `--chain dev` default.
- Sanvil and tests against externally managed nodes are unchanged. An external
  node must fund the test signer explicitly.

## Refreshing

When adopting a new reth genesis/predeploy revision, copy its `dev.json` here and
add **only** the account-zero allocation above. Do not restore the other old dev
prefunds or change the production genesis to accommodate tests. Update the pinned
source link and the canonical production digest in
`clients/py/tests/test_reth_genesis.py` together, then run both SDK integration
suites against the intended reth binary.

The regression test removes the one SDK allocation and checks the digest of the
entire remaining genesis, so unrelated changes to contracts, storage, or network
settings cannot silently enter the test fixture. Its digest is SHA-256 of the
source parsed and serialized with Python's
`json.dumps(genesis, sort_keys=True, separators=(",", ":"))`.
