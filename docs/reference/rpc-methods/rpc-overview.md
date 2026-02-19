---
icon: share-nodes
---

# rpc-overview

Users have access to all the standard [ETH RPC Methods](https://ethereum-json-rpc.com/?method=) as well as [seismic\_getTeePublicKey](seismic-get-tee-public-key.md).



It is worth noting there are some small but powerful differences in a few RPC methods that make Seismic unique.

### [eth\_call](eth-call.md)

Executes a message call without creating a transaction on the blockchain. On Seismic, `eth_call` has two important differences from standard Ethereum:

1. **`from` is zeroed on unsigned calls** — When you send an unsigned `eth_call`, the `from` field is set to `0x0000...0000`. This prevents contracts from using `msg.sender` to leak information about the caller during read operations.
2. **Signed reads via type `0x4A`** — To make an `eth_call` that preserves your identity (so the contract can return caller-specific data like balances), you must send a [signed read](/broken/pages/Iy1iEnGF6LoJvAy1G6ix) using Seismic transaction type `0x4A`.

### [eth\_getStorageAt](eth-get-storage-at.md)

Returns the value at a given storage position for an address. On Seismic, this method **fails with an error** if the requested storage slot contains shielded (private) data.

This is a deliberate security measure — shielded storage values are encrypted and flagged with `is_private = true` in the state trie. Exposing them via RPC would defeat the purpose of confidential storage.

### [eth\_sendRawTransaction](eth-send-raw-transaction.md)

Submits a signed transaction to the network. On Seismic, this endpoint accepts both standard Ethereum transaction types and the Seismic transaction type `0x4A`.

Seismic transactions (`0x4A`) include encrypted calldata and `SeismicElements` metadata (encryption public key, nonce, recent block hash, expiration block, and message version). The node decrypts the calldata inside the TEE before execution.
