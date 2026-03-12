---
icon: phone-arrow-right
---

# eth\_call

Executes a message call without creating a transaction on the blockchain. On Seismic, `eth_call` behaves differently depending on whether the call is unsigned or signed.

## Unsigned calls

When you send a standard unsigned `eth_call`, the `from` field is **zeroed out** (`0x00...0`) regardless of what you pass. This is what makes `msg.sender`-based access control possible on Seismic — without it, anyone could set an arbitrary `from` address and bypass access checks.

## Signed calls (signed reads)

To make an `eth_call` that preserves your identity — so the contract can return caller-specific data like shielded balances — you send a raw signed Seismic transaction (type `0x4A`) or an EIP-712 typed data request as the call payload. This is the same format you'd send to [`eth_sendRawTransaction`](eth-send-raw-transaction.md).

Set the `signed_read` field to `true` in the transaction's `SeismicElements` to prevent the read from being replayed as an actual transaction. See [Transaction Lifecycle](../seismic-transaction/tx-lifecycle.md) for the full flow.

## Related

* [Signed Reads](../seismic-transaction/signed-reads.md) — detailed signed read specification
* [Shielded Wallet Client](../../clients/typescript/viem/shielded-wallet-client.md) — viem client that handles this automatically
