# Seismic Client

Bun monorepo containing TypeScript client libraries for the [Seismic blockchain](https://seismic.systems) — a privacy-focused EVM chain that encrypts transaction calldata using AEAD (AES-GCM). The two published npm packages are **seismic-viem** (viem extensions) and **seismic-react** (wagmi-compatible React hooks).

## Build

### Mise

If you use mise, all you need is `mise install` which will install all the dependencies. cd'ing into this directory will automatically build the packages as well. You can also run `mise run build` to explicitly trigger a build.

### No Mise

Requires **Bun >=1.2.5**.

```bash
# If bun is not installed already:
curl -fsSL https://bun.sh/install | bash

bun install         # install dependencies
bun run all:build   # builds seismic-viem, seismic-react, seismic-viem-tests
```

### Verify

```bash
ls packages/seismic-viem/dist/_cjs/index.js packages/seismic-react/dist/_esm/index.js
# Both files should exist
```

### Individual package builds

```bash
bun run viem:build       # packages/seismic-viem  (CJS + ESM + types)
bun run react:build      # packages/seismic-react (CJS + ESM + types)
bun run tests:build      # packages/seismic-viem-tests (ESM + types)
```

## Checks

```bash
bun run check            # typecheck + lint (runs everything below)
bun run typecheck        # tsc --noEmit on seismic-viem then seismic-react
bun run lint:check       # eslint + prettier (no auto-fix)
bun run lint             # eslint + prettier (with auto-fix)
```

Note: `react:typecheck` rebuilds seismic-viem first (it depends on the built types).

## Tests

### Integration tests (require a running node)

Tests live in `tests/seismic-viem/` and connect to an already-running node. The test runner auto-detects the chain from `eth_chainId`.

**Quick start** (mise starts the node for you):

```bash
mise run test::anvil   # starts sanvil, runs tests, stops sanvil
mise run test::reth    # builds & starts reth, runs tests, stops reth
```

**Manual** (start a node yourself, then run tests):

```bash
# Terminal 1: start a node
sanvil                    # or: mise run anvil::start (from seismic/ root)

# Terminal 2: run tests
bun run viem:test         # connects to http://127.0.0.1:8545 by default
```

**Environment variables**:
- `RPC_URL` — HTTP RPC URL (default: `http://127.0.0.1:8545`)
- `WS_URL` — WebSocket URL (default: derived from `RPC_URL` by replacing `http` with `ws`)

**Anvil tests** require `sanvil` on PATH (install via `mise` or `sfoundryup`).

**Reth tests** require `SRETH_ROOT` pointing to a [seismic-reth](https://github.com/SeismicSystems/seismic-reth) checkout (or defaulting to `../../seismic-reth`).

> **Note**: The `seismic-viem-tests` package still exports process management functions (`setupNode`, `buildNode`, etc.) for backward compatibility with seismic-reth and seismic-foundry. These are deprecated and will be removed in a future version.

### Docs

```bash
bun run docs:dev         # local dev server
bun run docs:build       # production build (VoCs)
bun run docs:preview     # preview production build
```

## Project Layout

```
packages/
  seismic-viem/          Core viem extensions (npm: seismic-viem@1.1.1)
    src/
      chain.ts           Chain definitions (sanvil, seismicTestnet, localSeismicDevnet)
      client.ts          createShieldedPublicClient, createShieldedWalletClient
      sendTransaction.ts Seismic transaction sending
      contract/          getShieldedContract, shieldedWriteContract, signedReadContract
      crypto/            AES-GCM, ECDH, HKDF, nonce generation, AEAD
      precompiles/       On-chain precompile wrappers (rng, ecdh, hkdf, aes, secp256k1)
      actions/           Deposit contract, SRC20 token support
      abis/              SRC20, deposit contract, directory ABIs
  seismic-react/         React hooks (npm: seismic-react@1.1.1)
    src/
      context/           ShieldedWalletProvider
      hooks/             useShieldedWriteContract, useSignedReadContract, useShieldedContract
      rainbowkit/        RainbowKit integration
  seismic-viem-tests/    Shared test utilities (npm: seismic-viem-tests@0.1.4)
    src/
      process/           Node process management (anvil, reth spawn/kill)
      tests/             Reusable test functions (contract, precompiles, encoding, etc.)
  seismic-bot/           Slack bot for faucet management (internal)
  seismic-spammer/       Transaction load testing tool (internal)
tests/
  seismic-viem/          Integration test runner (bun test)
docs/                    Documentation site (VoCs + Tailwind)
```

## Code Style

**Prettier** (`.prettierrc.json`): 2-space indent, single quotes, no semicolons, 80-char width, trailing commas (ES5). Import sorting via `@trivago/prettier-plugin-sort-imports` with groups: types → external → relative (separated by blank lines).

**ESLint** (`eslint.config.cjs`): No relative import paths (`no-relative-import-paths` plugin), no unused imports. TypeScript-aware via `@typescript-eslint` with project service.

**TypeScript** (`tsconfig.base.json`, `tsconfig.json`): Shared base config with strict mode, NodeNext module resolution, ES2021 target. Each package extends the base and defines its own path aliases.

**TypeScript path aliases** — each package uses aliases instead of relative imports:

- `@sviem/*` → `seismic-viem/src/*`
- `@sreact/*` → `seismic-react/src/*`
- `@sviem-tests/*` → `seismic-viem-tests/src/*`

These are resolved at build time by `tsc-alias`.

## CI

GitHub Actions (`.github/workflows/ci.yml`):

- **lint**: ESLint + Prettier on ubuntu-24.04 (Bun 1.2.5)
- **typecheck**: tsc on ubuntu-24.04
- **test-anvil**: Self-hosted runner, runs anvil tests (sanvil from PATH)
- **test-devnet**: Self-hosted runner (after test-anvil), builds seismic-reth from `SRETH_ROOT`, runs reth tests

## Key Concepts

- **Seismic TX type** (`0x4a` / 74): Custom transaction type with encryption fields (pubkey, nonce, messageVersion, recentBlockHash, expiresAtBlock, signedRead)
- **Shielded clients**: Wrappers around viem clients that handle ECDH key exchange with the node and AES-GCM encryption of calldata
- **SRC20**: Seismic's shielded ERC20 variant with encrypted transfer logs
- **Precompiles**: On-chain crypto primitives at addresses `0x64`–`0x69` (RNG, ECDH, HKDF, AES-GCM encrypt/decrypt, secp256k1)
- **Chain ID**: 5124 (testnet), 31337 (local anvil)

## Troubleshooting

| Problem                                                     | Fix                                                                                                                                                                                                     |
| ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `SRETH_ROOT env variable must be set to build reth`         | Set `SRETH_ROOT` to your seismic-reth repo path (with Rust/Cargo installed)                                                                                                                             |
| `react:typecheck` fails with missing types                  | Run `bun run viem:build` first — react typecheck depends on built viem types                                                                                                                            |
| `Browserslist: browsers data is X months old` on docs build | Harmless warning. Fix with `npx update-browserslist-db@latest` if desired                                                                                                                               |
| `hideExternalIcon` React prop warning during docs build     | Harmless VoCs warning — safe to ignore                                                                                                                                                                  |
