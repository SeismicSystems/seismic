---
icon: paper-plane-top
---

# eth\_sendRawTransaction

Submits a signed transaction to the network. On Seismic, this endpoint is extended in two ways beyond standard Ethereum.

## Seismic transactions (type `0x4A`)

The endpoint accepts Seismic transaction type `0x4A` in addition to all standard Ethereum transaction types. Seismic transactions include encrypted calldata and `SeismicElements` metadata (e.g. encryption public key, block expiry). The transaction stays encrypted in the mempool and in blocks — decryption only happens inside the TEE at execution time. See [Transaction Lifecycle](../seismic-transaction/tx-lifecycle.md) for the full flow.

## EIP-712 typed data

The endpoint also accepts EIP-712 typed data as an alternative to RLP-encoded bytes. Instead of passing a hex-encoded raw transaction, you can pass a signed EIP-712 `TypedData` object directly. The node decodes the typed data, recovers the signer, and submits the transaction to the pool.

## Related

* [The Seismic Transaction](../seismic-transaction/README.md) — transaction type specification
* [Transaction Lifecycle](../seismic-transaction/tx-lifecycle.md) — end-to-end transaction flow
* [Shielded Writes](../../clients/typescript/viem/shielded-writes.md) — sending transactions with seismic-viem
