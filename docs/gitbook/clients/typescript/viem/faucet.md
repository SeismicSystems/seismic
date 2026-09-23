---
description: Claim funds from a compatible legacy faucet without reading recipient balances.
---

# Faucet Helper

`checkFaucet()` attempts a claim on every invocation. It does not inspect native or sUSDC balances. The faucet server decides eligibility and enforces cooldowns; rejection is surfaced as an error rather than a successful no-op.

## Supported API

This helper supports a **legacy faucet API**:

- `POST {faucetUrl}/api/claim` with JSON body `{ "address": "..." }`.
- A successful response containing `{ "msg": "Txhash: 0x..." }`.

It is **not compatible with the current public faucet's authenticated `/api/claim/new` API**, which returns `{ claimed, tier }` rather than a transaction hash. Use that faucet's web interface or a separate integration for that API. Removing the balance precheck does not change the helper's HTTP protocol.

```typescript
import { checkFaucet } from "seismic-viem";

// publicClient is an existing viem or shielded public client.
// faucetUrl must point to a server implementing the legacy API above.
const result = await checkFaucet({
  address: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
  publicClient,
  faucetUrl,
});
console.log(result.hash);
```

On success, the helper waits for the transaction receipt and returns `{ sent: true, hash, txUrl? }`. HTTP errors, rejected claims, malformed transaction hashes, and receipt-wait errors propagate to the caller. Invoke it on an explicit funding action rather than on every render or reconnect.

## Migration

The `minBalanceWei` and `minBalanceEther` parameters and the exported `parseMinBalance()` function have been removed. The result no longer includes a `{ sent: false }` variant.

Default `eth_getBalance` now returns a compatibility placeholder on updated reth; it cannot decide whether an account needs funds. A native balance check would also be wrong for an sUSDC faucet, since a wallet can pay gas with sUSDC while holding no native funds. See [Balance RPCs](../../../reference/balance-rpcs.md).

Faucet-contract inventory monitoring is separate from recipient claims. This helper does not read inventory, perform sUSDC signed reads, or replenish the faucet.
