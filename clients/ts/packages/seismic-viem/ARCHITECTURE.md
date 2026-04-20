# seismic-viem Architecture

How the package is organized and how a transaction flows from a caller down to the RPC wire.

For build / test commands and the broader TS monorepo index, see [`clients/ts/CLAUDE.md`](../../CLAUDE.md).

## Three layers

`src/` is organized into three layers:

- **Core** — the decorator surface composed by `createShielded{Public,Wallet}Client`. Lives under `actions/` plus a handful of root files (`client.ts`, `chain.ts`, `explorer.ts`).
- **Transaction path** — Seismic tx typing, serialization, RPC formatting, and the send / signedCall execution paths. Lives under `tx/`.
- **Extensions** — opt-in feature modules (faucet, deposit contract, SRC20). Lives under `extensions/`. Currently wired onto the default clients for convenience, but the split is semantic: these could become separate packages without touching core.

Everything else is supporting code: `crypto/` (off-chain primitives), `precompiles/` (on-chain precompile wrappers), `abis/`, `error/`, `viem-internal/`.

## Directory layout

```
src/
  chain.ts           Chain definitions (sanvil, seismicTestnet, localSeismicDevnet)
  client.ts          createShieldedPublicClient, createShieldedWalletClient
  explorer.ts        Block explorer URL builders (wired into the public decorator)
  actions/           Core viem decorators: public, wallet, encryption
  tx/                Seismic tx path
    seismicTx.ts       TxSeismic typing + serialization
    seismicRpc.ts      Chain formatters + RPC schemas
    sendTransparent.ts, sendShielded.ts, signedCall.ts   Execution paths
    metadata.ts        TxSeismicMetadata + buildTxSeismicMetadata
    signSeismicTypedData.ts   EIP-712 signing path (wallet accounts)
    types.ts           Shared request/response types for the tx path
  contract/          Smart-routing contract layer
    contract.ts        getShieldedContract wrapper
    read.ts, write.ts  smart / transparent / signed / shielded dispatchers
    calldata.ts, abi.ts   Plaintext calldata build + shielded ABI remap
  extensions/        Opt-in feature modules
    faucet.ts          Testnet faucet HTTP helper
    depositContract.ts Beacon deposit contract decorator
    src20/             Shielded ERC20 actions + event watchers
  crypto/            Off-chain primitives: AES-GCM, ECDH, HKDF, AEAD, nonce, secp
  precompiles/       On-chain precompile wrappers (rng, ecdh, hkdf, aes, secp256k1)
  error/             Seismic-specific error types
  viem-internal/     Vendored viem private types/helpers
  abis/              depositContract, directory, src20 ABIs
```

## Actions vs decorators

Viem-speak recap:

- An **Action** is a function `(client, params) => result` — e.g. `readContract`, `sendTransaction`.
- A **Decorator** is a function `(client) => ({ method1, method2, ... })` that bundles many actions so callers can invoke `client.method(...)` after `.extend(decorator)`.

Our `actions/*.ts` files each export a decorator, matching viem's `publicActions` / `walletActions` naming convention:

| file                    | decorator               | viem analogue        |
| ----------------------- | ----------------------- | -------------------- |
| `actions/public.ts`     | `shieldedPublicActions` | `publicActions`      |
| `actions/wallet.ts`     | `shieldedWalletActions` | `walletActions`      |
| `actions/encryption.ts` | `encryptionActions`     | _(no viem analogue)_ |

The decorators are composed together in `client.ts` to build `ShieldedPublicClient` and `ShieldedWalletClient`.

## Transaction flow

Three axes vary independently:

1. **Entry point** — how the caller reaches the tx path (wallet action, contract wrapper, or low-level helper).
2. **Routing** — smart (auto-detect by ABI), forced shielded, or forced transparent.
3. **Signing** — raw-sign a serialized Seismic tx (local key) vs EIP-712 typed data (external wallet). Decided by account type.

```
                   ShieldedWalletClient          (client.ts)
                   ┌─────────────────┐
                   │ .extend() chain │
                   │  actions/       │  ← public, wallet, encryption decorators
                   │  extensions/    │  ← depositContract, src20
                   └────────┬────────┘
                            │
      ┌─────────────────────┼───────────────────────┐
      │                     │                       │
      ▼                     ▼                       ▼
 client.*             getShieldedContract()    signedCall()
 wallet actions       (contract/contract.ts)   (tx/signedCall.ts)
 ──────────────       ──────────────────────   ──────────────
 sendTransaction      .read  / .write     ← smart (ABI-routed)
 writeContract        .sread / .swrite    ← force shielded
 sendShieldedTx       .tread / .twrite    ← force transparent
 signedCall           .dwrite             ← shielded + inspect
                            │
                    (smart routing lives in contract/read.ts & contract/write.ts;
                     uses hasShieldedParams() on the ABI)
                            │
          ┌─────────────────┼─────────────────┐
          ▼                 ▼                 ▼
      transparent        shielded         signed read
      send               send             (eth_call)
      tx/sendTransparent tx/sendShielded  tx/signedCall
          │                 │                 │
          │                 ├── buildTxSeismicMetadata (tx/metadata.ts)
          │                 │       · resolve nonce, chainId, encryptionNonce
          │                 │       · recentBlockHash + expiresAtBlock (default 100-block window)
          │                 │       · pick messageVersion
          │                 │
          │                 ├── encrypt calldata (actions/encryption.ts → crypto/aes.ts)
          │                 │       AES-GCM with metadata bound as AAD
          │                 │
          │                 └── sign ──┐
          │                            │
          │                            ▼
          │               ┌─────────────────────────────────────┐
          │               │  messageVersion = 0 (raw)           │
          │               │    requires account.type === 'local'│
          │               │    serializeSeismicTransaction      │
          │               │    + account.signTransaction        │
          │               │  → bytes for eth_sendRawTransaction │
          │               ├─────────────────────────────────────┤
          │               │  messageVersion = 2 (EIP-712)       │
          │               │    account.type ∈ {local, json-rpc} │
          │               │    signSeismicTxTypedData           │
          │               │    (tx/signSeismicTypedData.ts)     │
          │               │  → {typedData, signature} JSON      │
          │               │    forwarded through viem as the    │
          │               │    "serialized transaction"         │
          │               └─────────────────────────────────────┘
          │                            │
          ▼                            ▼
   eth_sendRawTransaction     eth_sendRawTransaction           eth_call
   (plain viem bytes)         (Seismic raw bytes or            (the only
                               {typedData, signature})          read-side
                                                                signing path)
```

### Picking a caller path

- `client.sendTransaction(...)` / `client.writeContract(...)` → viem's default → **transparent** path. Shielded features are ignored.
- `client.sendShieldedTransaction(...)` → force shielded.
- `getShieldedContract(...).read` / `.write` → **smart**: transparent for non-shielded ABIs, signed-read / shielded-write when any param is `suint` / `sint` / `sbool` / `saddress`.
- `.sread` / `.swrite` / `.tread` / `.twrite` → force a specific path regardless of ABI.
- `.dwrite` → shielded write, then re-reads and decrypts for inspection (debugging tool).

### Signing-path selector (inside a shielded path)

```
account.type === 'local'   && typedDataTx=false → messageVersion=0, raw sign
account.type === 'local'   && typedDataTx=true  → messageVersion=2, typed-data
account.type === 'json-rpc' (wallet ext, Ledger) → messageVersion=2, typed-data (only option)
```

`typedDataTx` defaults to `true` for both `local` and `json-rpc` accounts inside `buildTxSeismicMetadata`, so raw-sign only happens if a caller explicitly opts in.

### `SeismicSecurityParams` — overrides applied at the metadata step

- `encryptionNonce` — force a specific nonce (deterministic tests).
- `recentBlockHash` + `expiresAtBlock` — pin the replay window explicitly.
- `blocksWindow` — widen or shrink the default window (default: 100 blocks).
