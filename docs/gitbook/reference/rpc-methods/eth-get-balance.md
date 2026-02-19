---
icon: wallet
---

# eth_getBalance

Returns the native token balance of an address in wei. This method works identically to standard Ethereum.

{% hint style="info" %}
This returns the **public** native token balance (ETH on Seismic). It does not reveal shielded ERC-20 token balances — those require [signed reads](../seismic-transaction/signed-reads.md).
{% endhint %}

## Parameters

| Index | Type     | Description                                                               |
| ----- | -------- | ------------------------------------------------------------------------- |
| 1     | `string` | Address to check                                                          |
| 2     | `string` | Block number (`"latest"`, `"earliest"`, `"pending"`, or hex block number) |

## Returns

| Field  | Type     | Description                |
| ------ | -------- | -------------------------- |
| result | `string` | Hex-encoded balance in wei |

## Example Request

```bash
curl -X POST https://gcp-0.seismictest.net/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_getBalance",
    "params": ["0x0000000000000000000000000000000000000000", "latest"],
    "id": 1
  }'
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x0"
}
```

## Try It

<iframe src="https://seismicsystems.github.io/seismic/rpc-terminal/index.html?method=eth_getBalance" width="100%" height="500" style="border:none; border-radius:8px;"></iframe>

## Related

- [Testnet](../../networks/testnet.md) — get testnet ETH from faucet
- [Shielded Types](../../seismic-solidity/shielded-types/README.md) — shielded balances in contracts
