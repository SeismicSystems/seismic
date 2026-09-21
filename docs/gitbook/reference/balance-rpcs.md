---
description: Compatibility balances, native funds, and the balance RPC migration.
---

# Balance RPCs

These semantics apply to seismic-reth with [PR #502](https://github.com/SeismicSystems/seismic-reth/pull/502). They do not imply that older nodes or Sanvil implement the same `eth_getBalance` extension.

## Compatibility balance versus real funds

| Request | Result on updated reth |
| --- | --- |
| `eth_getBalance(address, block)` | Fixed wallet-compatibility placeholder, independent of holdings |
| `eth_getBalance(address, block, true)` | Actual public native balance |
| `eth_getAccountInfo(address, block)` | Actual public native `{ balance, nonce, code }` |

The default placeholder is **not spendable funds, an sUSDC balance, or proof that a transaction is affordable**. Never use it to calculate transfer amounts, trigger refills, or assert balance changes. Internal gas accounting continues to use real funds.

Standard SDK methods—viem's `getBalance()`, Web3.py's `w3.eth.get_balance()`, and Alloy's `provider.get_balance()`—still issue ordinary balance requests and therefore receive the placeholder on updated reth. They do not automatically opt into native mode.

Neither balance RPC exposes sUSDC holdings. Reading those requires an authenticated contract read under the token's access rules, not a public balance request. Native balance can be zero even when a wallet has sUSDC available for gas.

## Native queries

On updated reth, an explicit native request looks like:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "eth_getBalance",
  "params": ["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", "latest", true]
}
```

Alternatively, use `eth_getAccountInfo` with **exactly two arguments** and read its `balance` field. This is the endpoint used by seismic-viem's [`getNativeBalance()`](../clients/typescript/viem/shielded-public-client.md#native-balance), and is also supported by Sanvil. Errors are not converted into a fallback to ordinary `eth_getBalance`.

## Breaking parameter changes

- The old `includeGasToken` / `include_gas_token` named option is removed from both endpoints; sending it produces invalid-params error `-32602`.
- The third positional boolean on `eth_getBalance` now means `native`. Previously, `true` selected the USDC-inclusive effective balance. Do not assume it has the new meaning on older nodes.
- Omitted, `null`, or `false` `native` selects the placeholder. Only `true` selects native funds.
- `eth_getAccountInfo` requires address and block. A third argument is rejected, even if it is `null` or `false`.
- Unknown named fields, invalid types, and extra positional arguments are rejected.

For named parameters, `eth_getBalance` accepts `address`, `blockNumber` (or `block_number`), and `native`. `eth_getAccountInfo` accepts `address` and `block`.

## Block selection

Native queries resolve the requested block state. Default compatibility queries validate address/block syntax but do not resolve state: even a syntactically valid nonexistent block returns the same placeholder. A successful default balance request therefore does not prove that the block exists.

Sanvil may return native funds from ordinary `eth_getBalance` and reject reth's third parameter. Test against the intended backend rather than inferring RPC behavior from the response type.
