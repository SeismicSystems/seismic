---
icon: hard-drive
---

# eth_getStorageAt

Returns the value at a given storage position for an address. On Seismic, this method **fails with an error** if the requested storage slot contains shielded (private) data.

This is a deliberate security measure — shielded storage values are encrypted and flagged with `is_private = true` in the state trie. Exposing them via RPC would defeat the purpose of confidential storage.

## Parameters

| Index | Type     | Description                                                               |
| ----- | -------- | ------------------------------------------------------------------------- |
| 1     | `string` | Address of the contract                                                   |
| 2     | `string` | Storage slot position (hex)                                               |
| 3     | `string` | Block number (`"latest"`, `"earliest"`, `"pending"`, or hex block number) |

## Returns

| Field  | Type     | Description                                               |
| ------ | -------- | --------------------------------------------------------- |
| result | `string` | Hex-encoded 32-byte storage value (only for public slots) |

## Example Request

```bash
curl -X POST https://gcp-0.seismictest.net/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_getStorageAt",
    "params": [
      "0x1234567890abcdef1234567890abcdef12345678",
      "0x0",
      "latest"
    ],
    "id": 1
  }'
```

## Example Response (Public Slot)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": "0x0000000000000000000000000000000000000000000000000000000000000001"
}
```

## Example Response (Shielded Slot)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32000,
    "message": "storage slot is private"
  }
}
```

{% hint style="warning" %}
If you need to read shielded data, use a [signed read](../seismic-transaction/signed-reads.md) via `eth_call` with a Seismic transaction. The contract itself must expose a getter function for the shielded value.
{% endhint %}

## Try It

{% embed url="https://codesandbox.io/embed/github/SeismicSystems/seismic/tree/gh-pages?view=preview&hidenavigation=1&initialpath=%2Frpc-terminal%2Findex.html%3Fmethod%3Deth_getStorageAt" %}

## Related

- [Storage](../../seismic-solidity/storage.md) — how shielded storage works in Seismic Solidity
- [Opcodes](../opcodes.md) — `CLOAD`/`CSTORE` for shielded storage access
- [Differences from Ethereum](../../overview/differences-from-ethereum.md#rpc-compatibility) — RPC compatibility overview
