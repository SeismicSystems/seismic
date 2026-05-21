---
icon: key
---

# Cryptography

This page describes the cryptographic primitives and key material underlying Seismic transactions. It targets readers who want enough detail to audit, port, or build alternative implementations — the encryption scheme, KDF, AEAD, and AAD binding are all named explicitly.

## Network keys

* The network is bootstrapped with a chain-wide **`root_key`** (32 random bytes) which validators keep secret
  * Enclave can boot in either `--genesis-node` or `--peers <ip>` mode. The former generates its own root key and shares it with peers after they pass validation. The latter loops through its peer IPs until it receives the root key from one of them
* All other network-shared keys are derived from `root_key` via HKDF-SHA256, with a purpose label for domain separation. For example, the **`tx_io`** key (the ECIES recipient for transaction encryption) is derived as `HKDF(root_key, salt=None, info="tx_io key", length=32)`
* The key clients care about is **`tx_io`** — the secp256k1 keypair used as the ECIES recipient for transaction calldata. Clients fetch the `tx_io_pk` via the [`seismic_getTeePublicKey`](../rpc-methods/seismic-get-tee-public-key.md) RPC once and cache it for subsequent transactions.

## Client keys

For each transaction the client uses a secp256k1 encryption keypair to encrypt calldata to `tx_io_pk`. The client's encryption public key is sent on-chain as part of `TxSeismicMetadata`. Two strategies are supported, with different convenience/privacy trade-offs:

### Strategy 1 — ephemeral key per tx (default, recommended)

* Generate a fresh secp256k1 keypair for each transaction; use it once; discard `eph_sk` after submission
* Each tx has its own AES key — a different `eph_sk` produces a different ECDH output, a different `shared_secret`, and therefore a different `K`. The AES-GCM `encryptionNonce` can be a **fixed constant** (the reference clients use all-zeros) because `(K, nonce)` pairs are unique by construction, never reused
* **Self-decryption is unavailable once `eph_sk` is discarded** — the client can no longer reproduce `K` for that tx, and the protocol exposes no validator-side recovery mechanism. Clients that want to view their own historical calldata must persist `eph_sk` (or `K`) per tx in wallet-side storage at submission time
* **Privacy benefit:** a discarded `eph_sk` removes the client's *own* ability to recover the plaintext from the on-chain ciphertext. Reduces blast-radius of a wallet compromise (only txs whose ephemerals are still cached are decryptable), and provides strong forward deniability — even the client themselves cannot prove what was in past calldata

### Strategy 2 — long-lived client key

* Use a persistent secp256k1 keypair across many transactions
* `(client_sk, tx_io_pk)` produces the same `shared_secret` and therefore the same `K` for every tx the client sends; the `encryptionNonce` **must** be unique per tx under this `K`. Use an incrementing counter, or a freshly random 12-byte value (birthday-bound — collision-probable after ~2⁴⁸ txs)
* **Self-decryption is offline and free:** the client recomputes `K` once from `(client_sk, tx_io_pk)` and decrypts any of their past transactions directly from on-chain data. Useful for wallets that want to display a user's tx history without round-tripping a validator
* **Trade-off:** loss of `client_sk` exposes every transaction ever encrypted under it. Strategy 1 doesn't have this exposure by construction

### Strategy 3 — deterministic derivation from the signing key

Derive each per-tx encryption key deterministically from a seed bound to the user's wallet. Gives offline self-decryption without managing a separate encryption key — the only thing the user needs to back up is their normal signing seed.

* `eph_sk = HKDF(seed, chain_id, tx_nonce)`; `seed` source depends on the wallet (see below)
* **Self-decryption is offline:** any past `eph_sk` re-derives from the seed plus on-chain `(chain_id, tx_nonce)`. No per-tx storage required
* **Trade-off:** loss of the wallet's signing material exposes every past Seismic tx (no forward deniability)

Two sub-cases depending on what the wallet exposes:

* **(a) Direct** — if the dApp can access `signing_sk` (mnemonic-based JS wallets, server-side signers, programmatic test setups): `seed = signing_sk`
* **(b) Session signature** — if the wallet only exposes signing (MetaMask, Ledger, Trezor, KMS): prompt the user once at session start to `personal_sign` a fixed message (e.g., `"Seismic encryption session for {address}"`); use the resulting signature as `seed`, cached in dApp memory for the session lifetime. The same prompt always yields the same signature (deterministic ECDSA), so refresh-recovery works. The signature must not be exposed — sharing it grants offline decryption of every past tx

## Encryption scheme

Transaction calldata encryption is **ECIES on secp256k1** — a standard KEM-DEM hybrid public-key encryption construction, using the EC primitives Seismic already requires for signing.

End-to-end pipeline (client side; decryption inverts):

```
ECDH(client_sk, network_pk) ──► shared point
                              │
                              ▼  SHA256 of compressed point
                          shared_secret (32 B)
                              │
                              ▼  HKDF-SHA256, info = "aes-gcm key"
                          AES-256 key (32 B)
                              │
                              ▼  AES-GCM, 12 B nonce, AAD = RLP(TxSeismicMetadata)
                          ciphertext
```

**KEM (key encapsulation)**

* Curve: **secp256k1**
* ECDH on secp256k1: shared point = `client_sk × network_pk` (encryption side) / `network_sk × client_pk` (decryption side)
* Shared-secret derivation: `SHA256` of the SEC1 **compressed-point encoding** of the shared ECDH point — i.e., `SHA256(0x02 || x)` if the y-coordinate is even, `SHA256(0x03 || x)` if odd. This is libsecp256k1's `ecdh::SharedSecret::new(pk, sk)` default behaviour and matches the Rust [`ecies`](https://github.com/ecies/rs) crate convention; Python and TypeScript reimplementations construct the parity byte explicitly as `(shared_point.y[-1] & 0x01) | 0x02`

**KDF**

Takes the 32-byte `shared_secret` produced by the KEM step and expands it into the actual symmetric encryption key.

* Algorithm: **HKDF-SHA256**
* `ikm` (input keying material): the `shared_secret` from above
* `salt`: none — the `ikm` is already a SHA-256 output and therefore uniformly random, so Extract has no biased entropy to extract from. [RFC 5869 §3.1](https://datatracker.ietf.org/doc/html/rfc5869#section-3.1) explicitly permits this
* `info`: `"aes-gcm key"` — provides domain separation; lets the same `shared_secret` derive other context-specific keys in the future with a different label
* Output length: 32 bytes — the AES-256 key

**DEM (authenticated encryption)**

* **AES-256-GCM**, with a 12-byte nonce supplied by the client (the `encryptionNonce` field in `TxSeismicMetadata`)
* **AAD = RLP-encoded `TxSeismicMetadata`** — the 11-field metadata struct (`sender`, `chain_id`, tx `nonce`, `to`, `value`, `encryption_pubkey`, `encryption_nonce`, `recentBlockHash`, `expires_at_block`, `signedRead`, `messageVersion`). Binding everything that contextualizes the tx prevents ciphertext malleability across senders, replay across chains or blocks, and substitution attacks

**Decryption (any validator node)**

* `tx_io_sk` (re-derived from `root_key` on demand) + client's `eph_pk` from the on-chain tx → ECDH → same shared secret → same AES key → AEAD-open with the same AAD → plaintext calldata

## Curve choice

* **secp256k1** rather than X25519 (the curve used by HPKE / RFC 9180) to avoid a curve split: the same secp256k1 stack already required for Ethereum signing handles encryption. Clients can encrypt to `tx_io_pk` using libraries they already have for signatures (libsecp256k1, ethers, viem, web3.py, etc.) — no separate X25519 dependency
* The trade-off is that we use a custom ECIES variant instead of a standardized HPKE suite. RFC 9180 defines KEMs for X25519/X448/P-256/P-384/P-521 but not secp256k1, so adopting HPKE would have required switching curves

## Reference implementations

All implementations produce byte-identical ciphertexts for the same inputs.

* **Rust (ECIES primitive):** [`enclave/crates/enclave/src/crypto.rs`](https://github.com/SeismicSystems/enclave/tree/seismic/crates/enclave/src/crypto.rs) in the [`seismic-enclave`](https://github.com/SeismicSystems/enclave) crate. This is the source-of-truth Rust implementation, used by both the client side (via `seismic-alloy`) and the server side (via `seismic-enclave-server`)
* **Rust (TxSeismic wire format + tx construction):** [`seismic-alloy/crates/consensus`](https://github.com/SeismicSystems/seismic-alloy/tree/seismic/crates/consensus) — depends on `seismic-enclave` for the ECIES math
* **Python:** [`clients/py/src/seismic_web3/crypto/ecdh.py`](https://github.com/SeismicSystems/seismic/tree/main/clients/py/src/seismic_web3/crypto/ecdh.py) — independent reimplementation; matches the Rust shared-secret derivation by construction
* **TypeScript:** [`clients/ts/`](https://github.com/SeismicSystems/seismic/tree/main/clients/ts) — independent reimplementation
