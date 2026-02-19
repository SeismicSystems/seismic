---
icon: cube
---

# eth_blockNumber

Returns the number of the most recent block. This method works identically to standard Ethereum.

Useful for checking node sync status and for providing `recentBlockHash` when constructing Seismic transactions (the hash must be within 100 blocks of the latest).

## Parameters

None.

## Returns

| Field  | Type     | Description              |
| ------ | -------- | ------------------------ |
| result | `string` | Hex-encoded block number |

## Example Request

```bash
curl -X POST https://gcp-0.seismictest.net/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_blockNumber",
    "params": [],
    "id": 1
  }'
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x4b7"
}
```

{% hint style="info" %}
Seismic can produce multiple blocks per second. Block timestamps are stored in milliseconds internally but formatted in seconds via RPC for Ethereum compatibility.
{% endhint %}

## Try It

{% embed url="https://codesandbox.io/embed/github/SeismicSystems/seismic/tree/gh-pages?view=preview&hidenavigation=1&initialpath=%2Frpc-terminal%2Findex.html%3Fmethod%3Deth_blockNumber" %}

## Related

- [Differences from Ethereum](../../overview/differences-from-ethereum.md#block-times) — block time differences
- [Testnet](../../networks/testnet.md) — testnet information
- [Devnet](../../networks/devnet.md) — devnet information
