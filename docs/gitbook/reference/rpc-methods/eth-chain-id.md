---
icon: link
---

# eth_chainId

Returns the chain ID of the connected network. This method works identically to standard Ethereum.

| Network | Chain ID (Hex) | Chain ID (Decimal) |
| ------- | -------------- | ------------------ |
| Testnet | `0x1404`       | 5124               |

## Parameters

None.

## Returns

| Field  | Type     | Description          |
| ------ | -------- | -------------------- |
| result | `string` | Hex-encoded chain ID |

## Example Request

```bash
curl -X POST https://gcp-0.seismictest.net/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_chainId",
    "params": [],
    "id": 1
  }'
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x1404"
}
```

{% hint style="info" %}
The chain ID is important for transaction signing — using the wrong chain ID will cause transactions to be rejected. Client libraries like [seismic-viem](../../client-libraries/seismic-viem/chains.md) configure this automatically.
{% endhint %}

## Try It

{% embed url="https://seismicsystems.github.io/seismic/rpc-terminal/index.html?method=eth_chainId" %}

## Related

- [Chains](../../client-libraries/seismic-viem/chains.md) — chain configuration in seismic-viem
- [Testnet](../../networks/testnet.md) — testnet details
- [Devnet](../../networks/devnet.md) — devnet details
