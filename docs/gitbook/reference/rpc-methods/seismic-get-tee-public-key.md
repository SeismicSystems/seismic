---
icon: key
---

# seismic\_getTeePublicKey

Returns the TEE's encryption public key. This is the first step in building a [Seismic transaction](../seismic-transaction/README.md) — clients use this public key with their own ephemeral key to derive a shared AES encryption key via ECDH.

## Parameters

None.

## Returns

| Field  | Type     | Description                                            |
| ------ | -------- | ------------------------------------------------------ |
| result | hex string (33 bytes) | Compressed secp256k1 public key |

## Example Request

```bash
curl -X POST https://testnet-1.seismictest.net/rpc \
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
  "result": "028e76821eb4d77fd30223ca971c49738eb5b5b71eabe93f96b348fdce788ae5a0"
}
```

## Related

* [The Seismic Transaction](../seismic-transaction/README.md) — how this key is used in transaction encryption
* [Encryption (seismic-viem)](../../clients/typescript/viem/encryption.md) — client-side key exchange
* [ECDH Precompile](../precompiles/ecdh.md) — on-chain ECDH

## Try It

{% embed url="https://codesandbox.io/embed/github/SeismicSystems/seismic/tree/gh-pages?view=preview&hidenavigation=1&initialpath=%2Frpc-terminal%2Findex.html%3Fmethod%3Dseismic_getTeePublicKey%26embed%3Dtrue" %}
