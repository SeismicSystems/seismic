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
* The **`signed_read`** field should be set to `true` (default in our clients). The tx-pool rejects `signed_read = true` at write submission, so an intercepted signed read can't be replayed as a write tx
* The validator decrypts the calldata, runs the call inside the EVM, and returns the result **encrypted to the client's `encryption_pubkey`** — an on-path interceptor can't read the response either
* `signed_read = false` is also valid for `eth_call` — the same tx body can be passed to either endpoint. This is intentional so that `eth_estimateGas` works on **write** txs: gas estimation simulates the tx via the same `eth_call` path internally, so if `eth_call` rejected `signed_read = false`, clients couldn't estimate gas on a normal write tx without building a separate tx variant. Safety still holds: decrypting the response requires the client's `eph_sk`, and once the account's tx nonce increments, a captured `eth_call` payload can no longer execute as a write
