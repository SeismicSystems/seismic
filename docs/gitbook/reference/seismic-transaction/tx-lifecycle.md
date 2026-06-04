---
icon: arrows-spin
---

# Tx Lifecycle

How a Seismic transaction is built, signed, submitted, validated, and executed. For the encryption primitives themselves (ECIES on secp256k1, KDF, AEAD, AAD binding), see [Cryptography](cryptography.md).

## Building a Seismic transaction

A Seismic transaction is a legacy Ethereum tx with two additions: encrypted calldata in the `data` field, and a metadata struct called **`SeismicElements`**. The tx type is `74` / `0x4a`.

`SeismicElements` consists of six Seismic-specific fields:

* **`encryption_pubkey`** — the client's encryption pubkey for this tx (see [Cryptography → Client keys](cryptography.md#client-keys) for the available strategies)
* **`encryption_nonce`** — 12-byte AES-GCM nonce
* **`recentBlockHash`** — must be within 100 blocks of the latest block
* **`expires_at_block`** — block number after which the tx is invalid
* **`signed_read`** — boolean (defaults to `true` in our client implementations); if `true`, restricts the tx to signed-read use only — see [Signed reads](#signed-reads-read-variant)
* **`message_version`** — integer controlling the signing format; see [Wire vs. signing format](#wire-vs-signing-format)

Combined with the standard Ethereum fields (`sender`, `chain_id`, `nonce`, `to`, `value`), the full 11-field bundle is **`TxSeismicMetadata`**, which is RLP-encoded and used as AAD when the AEAD seals the calldata (see [Cryptography → Encryption scheme](cryptography.md#encryption-scheme)). Definitions in [`seismic-alloy-consensus`](https://github.com/SeismicSystems/seismic-alloy/tree/seismic/crates/consensus).

The encrypted calldata is placed in the standard `data` / `input` field of the tx envelope.

## Wire vs. signing format

A Seismic transaction has the same **wire format** in every case: an RLP-encoded type-`0x4a` envelope containing the tx fields, the encrypted calldata, and a signature. What can differ is the **signing format** — the bytes the ECDSA signature actually covers. This is controlled by the `message_version` field in `SeismicElements`:

* **`message_version = 0`** — Standard. Signature covers the RLP-encoded preimage of the tx (same shape as other Ethereum typed txs, e.g. EIP-2930, EIP-1559). The validator re-RLP-encodes the preimage and recovers the signer from the signature.
* **`message_version = 2`** — EIP-712 typed-data form. Signature covers an EIP-712 hash of the tx as a structured-data message. **This is what makes Seismic txs signable by MetaMask and other browser-extension wallets**, which don't natively understand tx type `0x4a` but do support `eth_signTypedData`. The validator reconstructs the typed-data hash and recovers the signer accordingly.
* **`message_version = 1`** — Reserved (originally planned for `eth_personalSign`-style signing; not implemented).

The wire-level type byte is always `0x4a`; only how the signer was authenticated differs.

## Submission and validation

Send to `eth_sendRawTransaction` like any Ethereum tx. Tx-pool validation rejects:

* Txs of type `0x4a` with incomplete or malformed `SeismicElements`
* Txs not marked `0x4a` but containing `SeismicElements`
* Txs with `signed_read = true` (those are for `eth_call`, not write txs)
* Calldata that fails to decrypt (see below)

Validated txs land in the pool and are included in blocks as usual.

## Node-side decryption

For each Seismic tx in the pool, the validator:

1. Decodes the tx and recovers the signer (using the `message_version`-appropriate signing format)
2. Assembles the metadata, RLP-encodes it — this is the AAD
3. Derives the AES key from `tx_io_sk` and the client's `encryption_pubkey` via ECDH + HKDF (see [Cryptography](cryptography.md))
4. AEAD-opens the ciphertext with the derived key, the `encryption_nonce`, and the AAD

On decryption failure, the tx is removed from the pool. On success, the plaintext calldata is passed to revm for execution inside the validator.

**`eth_getTransactionByHash` returns the on-chain ciphertext form, not the plaintext.** Clients that want to view their own historical calldata must arrange this client-side — see [Cryptography → Client keys](cryptography.md#client-keys) for the available strategies.

## Read variant

For authenticated reads — `eth_call` requests that prove `msg.sender` identity to a contract — Seismic uses **signed reads**: the same Seismic tx protocol, sent to `eth_call` instead of `eth_sendRawTransaction`. See [Signed Reads](signed-reads.md) for the specifics.

## Other notes

* Seismic txs use the same gas parameters as legacy txs: `gasPrice` and `gasLimit`
* Seismic txs cannot deploy bytecode. They must be sent to a specific address; `CREATE` / `CREATE2` from a Seismic tx is rejected. This simplifies metadata validation, and there's no useful reason to deploy encrypted bytecode
* **Calldata encoding subtlety:** shielded types (`suint`, `sbool`, etc.) are encoded the same way as their public analogues. Solidity has no idea whether a function is called with a Seismic tx or a plain tx — so it's legal (though usually wrong) to alter shielded state via a plain tx, or to alter public state via a Seismic tx. Be deliberate about which kind of tx your contract accepts
