---
description: Query SRC20 tokens deployed through the factory via a REST API
icon: plug
---

# REST API

The `src20-factory` repo includes a Rust API server (Axum) that reads factory and token state over HTTP. It is useful for indexing deployed tokens or fetching token metadata from non-TypeScript environments.

## Running the server

```bash
cd packages/api
cargo run
```

The server starts on port **3001** and connects to Seismic testnet at `https://gcp-2.seismictest.net/rpc`.

---

## GET /api/tokens

Returns all tokens ever deployed through the factory, with their metadata.

### Request

```
GET http://localhost:3001/api/tokens
```

### Response

```json
{
  "count": 2,
  "tokens": [
    {
      "address": "0xabc123...",
      "name": "My Private Token",
      "symbol": "MPT",
      "decimals": 18,
      "owner": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
      "total_supply": "1000000000000000000000000"
    },
    {
      "address": "0xdef456...",
      "name": "Another Token",
      "symbol": "AT",
      "decimals": 6,
      "owner": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
      "total_supply": "500000000000"
    }
  ]
}
```

{% hint style="info" %}
Addresses are returned as lowercase hex, not EIP-55 checksummed.
{% endhint %}

Note that `total_supply` is always in base units (not scaled by decimals).

---

## GET /api/token/{address}

Returns metadata for a single token by address.

### Request

```
GET http://localhost:3001/api/token/0xabc...
```

### Response

```json
{
  "name": "My Private Token",
  "symbol": "MPT",
  "decimals": 18,
  "owner": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
  "total_supply": "1000000000000000000000000"
}
```

### Error response

If the address is invalid or the RPC call fails, the server returns a JSON error:

```json
{
  "error": "Invalid address: ..."
}
```

---

## What the API can and cannot read

The API uses a standard public provider — it does not hold a private key. This means it can read public state like `name`, `symbol`, `decimals`, `owner`, and `totalSupply`, but it cannot read shielded state like individual balances or allowances, which require a signed read from the balance holder. See [Signed Reads](../../reference/seismic-transaction/signed-reads.md) for how those work.
