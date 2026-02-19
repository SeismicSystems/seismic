---
icon: network-wired
---

# net_version

Returns the current network ID as a string. This method works identically to standard Ethereum.

{% hint style="info" %}
`net_version` and `eth_chainId` both identify the network, but `net_version` returns a decimal string while `eth_chainId` returns a hex value. Prefer `eth_chainId` for transaction signing.
{% endhint %}

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

## Try It

<iframe src="https://seismicsystems.github.io/seismic/rpc-terminal/index.html?method=net_version" width="100%" height="500" style="border:none; border-radius:8px;"></iframe>

## Related

- [eth_chainId](eth-chain-id.md) — hex chain ID (preferred for signing)
- [Testnet](../../networks/testnet.md) — testnet details
