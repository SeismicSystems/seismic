---
icon: key
---

# seismic\_getTeePublicKey

Returns the TEE's encryption public key. This is the first step in building a [Seismic transaction](../seismic-transaction.md) — clients use this public key with their own ephemeral key to derive a shared AES encryption key via ECDH.

## Try It

{% embed url="https://codesandbox.io/embed/github/SeismicSystems/seismic/tree/gh-pages?view=preview&hidenavigation=1&initialpath=%2Frpc-terminal%2Findex.html%3Fmethod%3Dseismic_getTeePublicKey%26embed%3Dtrue" %}

## Parameters

None.

## Returns

| Field  | Type     | Description                                            |
| ------ | -------- | ------------------------------------------------------ |
| result | `string` | Hex-encoded compressed secp256k1 public key (33 bytes) |

## Example Request

```bash
curl -X POST https://gcp-0.seismictest.net/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "seismic_getTeePublicKey",
    "params": [],
    "id": 1
  }'
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x02abc123..."
}
```

## Related

* [Tx Lifecycle](/broken/pages/xCs9tpFaGc4bynMxW8zk) — how this key is used in transaction encryption
* [Encryption (seismic-viem)](../../client-libraries/seismic-viem/encryption.md) — client-side key exchange
* [ECDH Precompile](../precompiles.md#ecdh-0x65) — on-chain ECDH
