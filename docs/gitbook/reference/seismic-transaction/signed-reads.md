---
icon: file-signature
---

# Signed Reads

A signed read is a Seismic transaction (type `0x4a`) sent to `eth_call` instead of `eth_sendRawTransaction`. It lets contracts authenticate the reader's `msg.sender` for read-only queries — e.g., "only the owner can view their balance."

## Why signed reads exist

In the EVM, anyone can set the `from` field of an `eth_call` to spoof any address. Seismic closes this in two parts:

1. Vanilla `eth_call` has its `from` field **zeroed out** by the node — `msg.sender == 0` inside contract code
2. A signed read is a Seismic tx where the validator recovers the signer from the signature and uses *that* address as `msg.sender`

## How signed reads differ from write txs

A signed read is built exactly like a write tx (same `SeismicElements`, same encryption flow — see [Tx Lifecycle](tx-lifecycle.md) and [Cryptography](cryptography.md)). The differences are:

* **Sent to `eth_call`**, not `eth_sendRawTransaction`. Either a raw tx or an EIP-712 envelope (`message_version = 2`)
* The **`signed_read`** field **must** be `true` — the node rejects a signed read submitted with `signed_read = false`. The tx-pool conversely rejects `signed_read = true` at write submission, so a signed read can't be replayed as a write, and a write-intent payload can't be used as a signed read
* The validator decrypts the calldata, runs the call inside the EVM, and returns the result **encrypted to the client's `encryption_pubkey`** — an on-path interceptor can't read the response either
* **Estimating a write:** since a signed read must set `signed_read = true`, clients estimate gas for a *write* by signing a **separate provisional** signed-read tx (same sender / `to` / `value` / calldata, `signed_read = true`) and sending that to `eth_estimateGas`; the real write is signed separately. The provisional is non-broadcastable — the pool rejects signed reads as writes — so a captured estimate payload can't be replayed as a write
