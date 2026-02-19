---
icon: network-wired
---

# net_version

Returns the current network ID as a string. This method works identically to standard Ethereum.

{% hint style="info" %}
`net_version` and `eth_chainId` both identify the network, but `net_version` returns a decimal string while `eth_chainId` returns a hex value. Prefer `eth_chainId` for transaction signing. The chain ID is `0x1404` (5124) on testnet.
{% endhint %}

## Try It

{% embed url="https://codesandbox.io/embed/github/SeismicSystems/seismic/tree/gh-pages?view=preview&hidenavigation=1&initialpath=%2Frpc-terminal%2Findex.html%3Fmethod%3Dnet_version%26embed%3Dtrue" %}

## Parameters

None.

## Returns

| Field  | Type     | Description                    |
| ------ | -------- | ------------------------------ |
| result | `string` | Network ID as a decimal string |

## Example Request

```bash
curl -X POST https://gcp-0.seismictest.net/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "net_version",
    "params": [],
    "id": 1
  }'
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "5124"
}
```

## Related

- [Testnet](../../networks/testnet.md) — testnet details
