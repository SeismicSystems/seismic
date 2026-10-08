# Seismic viem actions

This is a package to extend viem clients for use with the Seismic blockchain

## Docs

The docs are hosted [here](https://docs.seismic.systems/clients/typescript/viem)

## Transaction responses

The exported Seismic chains and chain factories format existing transaction lookups and full-block transactions. Seismic responses have `type: 'seismic'`, the original `typeHex` (`'0x4a'` or Reth's `'0x4A'`), and a typed `gasPayment`; ordinary Ethereum responses retain viem formatting. This works with ordinary viem public clients and Seismic public/wallet clients using those chains.

```typescript
const tx = await client.getTransaction({ hash: txHash })
if (tx.type === 'seismic') {
  console.log(tx.gasPayment) // GasPayment: the signed preference, not Auto's selected asset
  console.log(tx.encryptionNonce, tx.expiresAtBlock) // bigint
}
```

`SeismicTransaction` describes the Seismic branch; `SeismicTransactionResponse` includes ordinary Ethereum transactions. Response quantities are formatted like viem quantities and are distinct from signing inputs. See the [gas-payment guide](https://docs.seismic.systems/clients/typescript/viem/gas-payment) for the response shape and examples.

From `clients/ts`, `bun run viem:consumer:test` builds the package and compiles lookup examples against public declarations without source aliases.

## Contributor docs

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the package layout, the transaction flow diagram, and the `actions/` ↔ viem-decorator relationship.
