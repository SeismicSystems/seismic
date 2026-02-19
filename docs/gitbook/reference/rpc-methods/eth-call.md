---
icon: phone-arrow-right
---

# eth_call

Executes a message call without creating a transaction on the blockchain. On Seismic, `eth_call` has two important differences from standard Ethereum:

1. **`from` is zeroed on unsigned calls** — When you send an unsigned `eth_call`, the `from` field is set to `0x0000...0000`. This prevents contracts from using `msg.sender` to leak information about the caller during read operations.

2. **Signed reads via type `0x4A`** — To make an `eth_call` that preserves your identity (so the contract can return caller-specific data like balances), you must send a [signed read](../seismic-transaction/signed-reads.md) using Seismic transaction type `0x4A`.

## Parameters

| Index | Type     | Description                                                               |
| ----- | -------- | ------------------------------------------------------------------------- |
| 1     | `object` | Transaction call object                                                   |
| 2     | `string` | Block number (`"latest"`, `"earliest"`, `"pending"`, or hex block number) |

### Transaction call object

| Field      | Type     | Required | Description                               |
| ---------- | -------- | -------- | ----------------------------------------- |
| `from`     | `string` | No       | Sender address (zeroed on unsigned calls) |
| `to`       | `string` | Yes      | Contract address                          |
| `gas`      | `string` | No       | Gas limit (hex)                           |
| `gasPrice` | `string` | No       | Gas price (hex)                           |
| `value`    | `string` | No       | Value in wei (hex)                        |
| `data`     | `string` | No       | Encoded function call data                |

## Returns

| Field  | Type     | Description                           |
| ------ | -------- | ------------------------------------- |
| result | `string` | Hex-encoded return data from the call |

## Example Request

```bash
curl -X POST https://gcp-0.seismictest.net/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_call",
    "params": [
      {
        "to": "0x1234567890abcdef1234567890abcdef12345678",
        "data": "0x70a08231000000000000000000000000000000000000000000000000000000000000dead"
      },
      "latest"
    ],
    "id": 1
  }'
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x0000000000000000000000000000000000000000000000000000000000000000"
}
```

{% hint style="warning" %}
If you need caller-specific data (e.g., a user's shielded balance), you must use a [signed read](../seismic-transaction/signed-reads.md). A plain `eth_call` will have `from` set to the zero address.
{% endhint %}

## Try It

<iframe src="https://seismicsystems.github.io/seismic/rpc-terminal/index.html?method=eth_call" width="100%" height="500" style="border:none; border-radius:8px;"></iframe>

## Related

- [Signed Reads](../seismic-transaction/signed-reads.md) — how to make authenticated read calls
- [Shielded Public Client](../../client-libraries/seismic-viem/shielded-public-client.md) — viem client that handles this automatically
- [Differences from Ethereum](../../overview/differences-from-ethereum.md) — overview of all behavioral differences
