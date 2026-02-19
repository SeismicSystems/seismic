---
icon: paper-plane-top
---

# eth\_sendRawTransaction

Submits a signed transaction to the network. On Seismic, this endpoint accepts both standard Ethereum transaction types and the Seismic transaction type `0x4A`.

Seismic transactions (`0x4A`) include encrypted calldata and `SeismicElements` metadata (encryption public key, nonce, recent block hash, expiration block, and message version). The node decrypts the calldata inside the TEE before execution.

## Try It

{% embed url="https://codesandbox.io/embed/github/SeismicSystems/seismic/tree/gh-pages?view=preview&hidenavigation=1&initialpath=%2Frpc-terminal%2Findex.html%3Fmethod%3Deth_sendRawTransaction%26embed%3Dtrue" %}

## Parameters

| Index | Type     | Description                         |
| ----- | -------- | ----------------------------------- |
| 1     | `string` | Hex-encoded signed transaction data |

## Returns

| Field  | Type     | Description                              |
| ------ | -------- | ---------------------------------------- |
| result | `string` | Transaction hash (32 bytes, hex-encoded) |

## Example Request

```bash
curl -X POST https://gcp-0.seismictest.net/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_sendRawTransaction",
    "params": ["0x02f87..."],
    "id": 1
  }'
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0xabc123def456..."
}
```

{% hint style="info" %}
You typically don't call this method directly. Client libraries like [seismic-viem](../../client-libraries/seismic-viem/) handle transaction construction, encryption, and submission automatically.
{% endhint %}

## Seismic Transaction Validation

The node rejects transactions in these cases:

* Type `0x4A` with incomplete `SeismicElements`
* Non-`0x4A` type that contains `SeismicElements`
* Failed calldata decryption
* `recentBlockHash` older than 100 blocks
* Past `expiresAtBlock`

## Related

* [The Seismic Transaction](../seismic-transaction.md) — transaction type specification
* [Tx Lifecycle](/broken/pages/xCs9tpFaGc4bynMxW8zk) — end-to-end transaction flow
* [Shielded Writes](../../client-libraries/seismic-viem/shielded-writes.md) — sending transactions with seismic-viem
