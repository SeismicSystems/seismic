---
icon: dollar-sign
---

# The Seismic Transaction

<figure><img src="../../.gitbook/assets/seismic-tx-format.png" alt=""><figcaption></figcaption></figure>

Seismic extends Ethereum's transaction model with encrypted calldata and authenticated reads. This section covers:

* [**Cryptography**](cryptography.md) — The encryption scheme (ECIES on secp256k1), key derivation, AEAD construction, and client key-management strategies
* [**Tx Lifecycle**](tx-lifecycle.md) — How a Seismic transaction is built, signed, submitted, validated, and executed; the `SeismicElements` metadata fields; wire vs. signing format; tx-pool validation rules
* [**Signed Reads**](signed-reads.md) — The read variant: Seismic txs sent to `eth_call`, with `msg.sender` recovered from the signature
