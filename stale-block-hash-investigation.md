# Stale Block Hash Investigation

## Problem

Users are seeing stale `recentBlockHash` values in Seismic transactions:
- One user saw a block hash that was **17 hours old**
- After hard-refreshing the page, the block hash was still **12 minutes old**
- Direct curl to the RPC node (`gcp-2.seismictest.net/rpc`) returns a **fresh block** (0s old), so the node itself is not behind

## Repos Involved

| Repo | Path | Role |
|------|------|------|
| seismic (monorepo) | `/home/c/code/seismic-workspace/seismic` | Workspace root, contains clients/ts |
| seismic-viem | `/home/c/code/seismic-workspace/seismic/clients/ts/packages/seismic-viem` | TypeScript SDK — builds and sends shielded transactions |
| seismic-react | `/home/c/code/seismic-workspace/seismic/clients/ts/packages/seismic-react` | React hooks wrapping seismic-viem |
| march-madness web | `/home/c/code/seismic-workspace/apps/march-madness/packages/web` | The dapp where the bug was observed |
| march-madness client | `/home/c/code/seismic-workspace/apps/march-madness/packages/client` | Contract client library used by the dapp |
| seismic-reth | `/home/c/code/seismic-workspace/seismic-reth` | The Seismic execution client (Rust/reth fork) — validates `recentBlockHash` on-chain |

## Architecture / Call Chain

When a user submits a bracket in the march-madness dapp:

```
[march-madness web]  useContract.ts:329
  mmUser.submitBracket(bracketHex)

[march-madness client]  client.ts:193-202
  this.shieldedContract.write.submitBracket([bracket], { value: entryFee })

[seismic-viem]  contract/contract.ts:297-321  (Proxy)
  shieldedWriteContract(walletClient, { abi, address, functionName, args, ...options })

[seismic-viem]  contract/write.ts:151-158
  sendShieldedTransaction(client, request)

[seismic-viem]  sendTransaction.ts:203-211
  buildTxSeismicMetadata(client, { account, nonce, to, value, blocksWindow: 100n, signedRead: false })

[seismic-viem]  metadata.ts:104-128  ← THIS IS WHERE THE BLOCK HASH IS FETCHED
  resolveBlockParams():
    const latestBlock = await client.getBlock({ blockTag: 'latest' })
    return { recentBlockHash: latestBlock.hash, expiresAtBlock: latestBlock.number + blocksWindow }
```

For signed reads (e.g. loading a bracket before deadline), a similar chain goes through:
```
[seismic-viem]  signedCall.ts:298-304
  buildTxSeismicMetadata(client, { account, to, blocksWindow, signedRead: true })
```

## What We've Ruled Out

### 1. seismic-viem has NO block caching
- `buildTxSeismicMetadata` calls `client.getBlock({ blockTag: 'latest' })` fresh every time
- No memoization, no module-level state, no Map/WeakMap caches for block data
- No closure-based caching in the client delegation chain
- The only module-level cache (`schedulerCache` in `viem-internal/call.ts:320`) is for multicall request batching, not block data

### 2. viem's `getBlock` has NO caching
- `getBlock({ blockTag: 'latest' })` makes a direct `eth_getBlockByNumber("latest", false)` RPC call
- No deduplication for blockTag requests (only for numeric blockNumber)
- viem's `cacheTime` (default 4s) only affects `getBlockNumber`, not `getBlock`
- HTTP POST requests are not cached by browsers or CDNs

### 3. nginx is NOT caching
The nginx config at `gcp-2.seismictest.net` has:
- No `proxy_cache_path` or `proxy_cache` directives
- `/rpc` location is a straight `proxy_pass http://localhost:8545`
- `proxy_cache_bypass $http_upgrade` is a no-op without a cache zone

### 4. The reth node is NOT behind
```bash
curl -s https://gcp-2.seismictest.net/rpc -X POST \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest",false],"id":1}'
```
Returns a block with age: 0s.

### 5. seismic-react hooks don't cache block data
- `useShieldedWriteContract` calls `shieldedWriteContract` at execution time (on user click), not during render
- `useSignedReadContract` similarly calls at execution time
- `ShieldedWalletProvider` stores `publicClient` and `walletClient` in React state but these are viem client instances, not cached data

## What We Haven't Ruled Out

### The dapp's caching layers
The march-madness web app uses:

1. **React Query / TanStack Query** (`/home/c/code/seismic-workspace/apps/march-madness/packages/web/src/lib/config.ts:7`):
   ```typescript
   export const queryClient = new QueryClient();  // default config — gcTime: 5min
   ```
   However, the shielded writes do NOT go through React Query — they call `mmUser.submitBracket()` directly.

2. **wagmi + Privy** (`config.ts:28-36`):
   ```typescript
   export const config = createConfig({
     chains: APP_CHAINS,
     transports: {
       [sanvil.id]: http(),
       [seismicTestnet.id]: http(import.meta.env.VITE_RPC_URL),
     },
     ssr: false,
   });
   ```
   This creates the wagmi transport. The seismic-viem wallet client uses `custom(transport)` from the wagmi connector client (see `shieldedWallet.tsx:192`).

3. **The wagmi connector transport** — this is the key unknown. In `ShieldedWalletProvider` (`shieldedWallet.tsx:186-194`):
   ```typescript
   createShieldedWalletClient({
     account,
     chain,
     transport: custom(transport),  // ← transport comes from wagmi's useConnectorClient
     publicClient,
   })
   ```
   The `transport` from `useConnectorClient` wraps the wallet's (Privy's) provider. The `publicClient` uses `http()` transport. When the wallet client calls `client.getBlock()`, it delegates to the `publicClient` (via `publicActions(pubClient)` extension at `client.ts:244`). So `getBlock` goes through the `http()` transport of the public client, NOT through the wagmi connector transport.

   But the `publicClient` is created with `http()` (no URL) which uses the chain's default RPC URL. For `seismicTestnetGcp2`, that's `https://gcp-2.seismictest.net/rpc`. For the dapp, if `VITE_RPC_URL` is set, the wagmi transport uses that URL, but the **publicClient** might use a different URL (the chain default).

## Key Files

### seismic-viem (where block hash is fetched)
- **`metadata.ts`** (`/home/c/code/seismic-workspace/seismic/clients/ts/packages/seismic-viem/src/metadata.ts`) — `buildTxSeismicMetadata()` and `resolveBlockParams()` at lines 104-128
- **`sendTransaction.ts`** (`/home/c/code/seismic-workspace/seismic/clients/ts/packages/seismic-viem/src/sendTransaction.ts`) — `sendShieldedTransaction()` calls `buildTxSeismicMetadata` at line 203
- **`signedCall.ts`** (`/home/c/code/seismic-workspace/seismic/clients/ts/packages/seismic-viem/src/signedCall.ts`) — `signedCall()` calls `buildTxSeismicMetadata` at line 298
- **`contract/write.ts`** (`/home/c/code/seismic-workspace/seismic/clients/ts/packages/seismic-viem/src/contract/write.ts`) — `shieldedWriteContract()` at line 151
- **`contract/contract.ts`** (`/home/c/code/seismic-workspace/seismic/clients/ts/packages/seismic-viem/src/contract/contract.ts`) — `getShieldedContract()` Proxy-based write/read dispatch
- **`client.ts`** (`/home/c/code/seismic-workspace/seismic/clients/ts/packages/seismic-viem/src/client.ts`) — client creation, `publicActions(pubClient)` delegation at line 244

### seismic-react (React context)
- **`shieldedWallet.tsx`** (`/home/c/code/seismic-workspace/seismic/clients/ts/packages/seismic-react/src/context/shieldedWallet.tsx`) — `ShieldedWalletProvider`, creates publicClient and walletClient

### march-madness (the dapp)
- **`config.ts`** (`/home/c/code/seismic-workspace/apps/march-madness/packages/web/src/lib/config.ts`) — wagmi config, QueryClient, transport setup
- **`useContract.ts`** (`/home/c/code/seismic-workspace/apps/march-madness/packages/web/src/hooks/useContract.ts`) — `submitBracket()` at line 329, `loadMyBracket()` at line 277
- **`client.ts`** (`/home/c/code/seismic-workspace/apps/march-madness/packages/client/src/client.ts`) — `MarchMadnessUserClient.submitBracket()` at line 193

### Chain definitions
- **`chain.ts`** (`/home/c/code/seismic-workspace/seismic/clients/ts/packages/seismic-viem/src/chain.ts`) — `seismicTestnetGcp2` at line 394, default RPC: `https://gcp-2.seismictest.net/rpc`

## Proposed Fix

Add a timestamp guard in `resolveBlockParams()` at `metadata.ts:115`:

```typescript
const latestBlock = await client.getBlock({ blockTag: 'latest' })
const nowSec = BigInt(Math.floor(Date.now() / 1000))
const ageSec = nowSec - latestBlock.timestamp
if (ageSec > 120n) {
  throw new Error(
    `RPC node appears stale: latest block #${latestBlock.number} is ${ageSec}s old`
  )
}
```

This makes seismic-viem defensive against any upstream caching — whether it's the dapp layer, a future middleware, or a stale node. The dapp doesn't have to think about it.

## Open Questions

1. Is the `publicClient`'s `http()` transport (with chain-default URL) pointing to a different RPC endpoint than the dapp's `VITE_RPC_URL`? If `VITE_RPC_URL` points to a healthy node but the chain-default URL points to a stale one, that would explain the discrepancy.
2. Could Privy's embedded wallet provider be intercepting or caching RPC calls?
3. Is there a browser extension (e.g. a wallet) injecting a stale provider?
4. Should we also add a retry-via-raw-RPC fallback before throwing, to distinguish "app-layer cache" from "node is behind"?
