---
icon: hard-drive
---

# eth\_getStorageAt

Returns the value at a given storage position for an address. On Seismic, this method **returns zero** for shielded (private) storage slots — as if the slot were uninitialized.

This is a deliberate security measure — shielded storage values are flagged with `is_private = true` in the state trie. Rather than exposing the encrypted value or returning an error, the node returns the same response as an empty slot.

## Parameters

| Index | Type     | Description                                                               |
| ----- | -------- | ------------------------------------------------------------------------- |
| 1     | `string` | Address of the contract                                                   |
| 2     | `string` | Storage slot position (hex)                                               |
| 3     | `string` | Block number (`"latest"`, `"earliest"`, `"pending"`, or hex block number) |

## Returns

| Field  | Type     | Description                                               |
| ------ | -------- | --------------------------------------------------------- |
| result | hex string (32 bytes) | Storage value (only for public slots) |

## Example Request

```bash
curl -X POST https://gcp-1.seismictest.net/rpc \
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
  "result": "0x0000000000000000000000000000000000000000000000000000000000000000"
}
```

{% hint style="warning" %}
If you need to read shielded data, use a [signed read](../seismic-transaction/signed-reads.md) via `eth_call` with a signed Seismic transaction. The contract itself must expose a getter function for the shielded value.
{% endhint %}

## `eth_getFlaggedStorageAt` (sanvil only)

In [seismic-foundry](https://github.com/SeismicSystems/seismic-foundry)'s local development node (`sanvil`), an additional method `eth_getFlaggedStorageAt` is available. It takes the same parameters as `eth_getStorageAt` but returns a `FlaggedStorage` object containing both the actual value and whether the slot is private:

```json
{
  "value": "0x0000000000000000000000000000000000000000000000000000000000000042",
  "is_private": true
}
```

This is useful for debugging and testing — you can inspect shielded slot values and confirm their privacy flag. **This method does not exist in seismic-reth** — production nodes must never expose private storage values over RPC.

## Related

* [Storage](../../seismic-solidity/storage.md) — how shielded storage works in Seismic Solidity
* [Opcodes](../opcodes.md) — `CLOAD`/`CSTORE` for shielded storage access
* [Differences from Ethereum](../../overview/differences-from-ethereum.md#rpc-compatibility) — RPC compatibility overview
