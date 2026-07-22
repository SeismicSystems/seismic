# Key Schedule

Every key derivation in the Seismic protocol, in one place. Security reviews
of domain separation start here: within each layer, no two rows may share an
HKDF info label unless a compatibility reason is documented.

Two derived keys are independent unless **both** the input keying material
(IKM) and the label match. Labels therefore only need to be unique within a
layer — cross-layer label reuse is harmless because the IKM classes never
overlap. Within a layer, distinct labels make key independence a structural
property of the protocol rather than an assumption about which keypairs are
(re)used where.

## Layer 1 — Root key

The network root key (32 bytes, generated from the OS CSPRNG on the genesis
node, distributed to joining nodes via the root-key-wrap handshake) is the IKM
for every long-lived node secret.

- **Code**: `enclave/crates/key-custodian/src/custodian.rs` (`KeyPurpose`)
- **KDF**: HKDF-SHA256, salt `"seismic-purpose-derive-salt"`,
  info `"seismic-purpose-{label}" || epoch_be64`

| Purpose | Label | Output | Consumers |
| --- | --- | --- | --- |
| `TxIo` | `tx-io` | secp256k1 secret key (TEE side of transaction ECDH) | block executor, RPC signed reads |
| `RngPrecompile` | `rng-precompile` | schnorrkel keypair (IKM for the RNG precompile) | RNG precompile (0x64) |
| `Snapshot` | `snapshot` | AES-256-GCM key for state snapshots | snapshot encrypt/decrypt |
| `Storage` | `storage` | LUKS volume unlock key (epoch 0 only) | setup-persistent-luks |
| `LuksHeaderMac` | `luks-header-mac` | HMAC key for LUKS2 header verification (epoch 0 only) | setup-persistent-luks |

## Layer 2 — ECDH shared secrets

Every AES key derived from a secp256k1 ECDH shared secret (the SHA-256 of the
compressed shared point). This is the layer where an on-chain oracle exists:
the ECDH precompile evaluates the `EcdhPrecompile` row for **caller-chosen
keys**, so no other row may reuse its label — otherwise contracts could
compute that row's keys.

- **Code**: `enclave/crates/crypto/src/lib.rs` (`AesKeyDomain`),
  mirrored in `seismic-viem` (`crypto/aes.ts`) and `seismic_web3`
  (`crypto/ecdh.py`)
- **KDF**: HKDF-SHA256, no salt (the IKM is already a SHA-256 output;
  permitted by RFC 5869 §3.1), info = label below
- **Cross-language enforcement**: known-answer tests in all three languages
  pin identical key bytes for the same ECDH inputs (`bf0dd6…` for both
  `TxRequest` and `EcdhPrecompile` — same original label — and `974b31…` for
  `TxResponse`; Rust additionally pins `f48820…` for `RootKeyWrap`)

| Domain | Label | Output | Consumers |
| --- | --- | --- | --- |
| `TxRequest` | `aes-gcm key` (original label) | AES-256-GCM request traffic key | clients encrypt calldata / signed-read requests; TEE decrypts |
| `TxResponse` | `seismic/response/aes-256-gcm/v1` | AES-256-GCM response traffic key | TEE encrypts signed-read results; clients decrypt |
| `EcdhPrecompile` | `aes-gcm key` | AES-256 key returned on-chain — **consensus-frozen** | ECDH precompile (0x65) |
| `RootKeyWrap` | `seismic/root-key-wrap/aes-256-gcm/v1` | AES-256-GCM handshake key for root-key bootstrap | key-custodian wrap/unwrap |

`TxRequest` and `TxResponse` are distinct because a signed read carries one
public nonce that is used in both directions: independent keys keep every
AES-GCM `(key, nonce)` pair unique. Nonces must still be unique per
transaction when an ECDH keypair is reused across transactions.

`TxRequest` keeps `aes-gcm key`, the original label every ECDH-derived key used
before they were split by domain. The request path is decrypted in the block
executor, so changing its label is a consensus break; the nonce-reuse fix only
needs the response key to differ, so only `TxResponse` took a fresh label.
`EcdhPrecompile` stays on the same original label (consensus-frozen).
`RootKeyWrap` can rotate its label with a coordinated custodian release — the
handshake uses fresh ephemeral keypairs, one message per key, and persists
nothing.

## Layer 3 — On-chain precompile KDFs (consensus-frozen)

Self-contained derivations inside seismic-revm whose inputs come from
contracts. No off-chain implementation exists or needs to match them, so
their labels live with the precompile code, not in a shared library.

| Precompile | KDF | Notes |
| --- | --- | --- |
| HKDF (0x68) | HKDF-SHA256, info `"seismic_hkdf_105"`, IKM = caller input | `crates/seismic/src/precompiles/hkdf_derive_sym_key.rs` |
| RNG (0x64) | HKDF-SHA256, salt `"seismic rng context"`, IKM = RNG keypair secret, info = per-call domain data (block hash, tx hash, gas) ‖ `"pers"` ‖ personalization | `crates/seismic/src/precompiles/rng/domain_sep_rng.rs`; fresh derivation per call, no state |

## Changing an existing label

A label change re-derives the key, so what a label pins down is whatever its
key protects. Labels fall into four classes, from most to least frozen:

1. **Contract-data-pinned** — `EcdhPrecompile`, HKDF precompile. Outputs are
   observable by contracts and may be persisted in contract state (encrypted
   blobs, derived identifiers). A hardfork can change *future* outputs at a
   consensus-agreed block, but can never make data derived under the old
   label reachable again. Treat these as frozen forever.
2. **Consensus-replay-pinned** — `TxIo` and `RngPrecompile` (root-key layer),
   and the `TxRequest`/`TxResponse` ECDH labels. Historical blocks embed
   ciphertexts and RNG outputs produced under the old derivation; a node
   syncing from genesis re-executes them. Changing these on a live chain
   requires either a chain reset (new genesis) or a fork gate that selects
   the derivation by block timestamp or message version, so old blocks
   replay with the old derivation.
3. **Persisted-secret-pinned** — `Storage` and `LuksHeaderMac` (existing LUKS
   volumes stop unlocking; nodes need re-provisioning) and `Snapshot`
   (existing snapshots become undecryptable until regenerated).
4. **Unpinned** — `RootKeyWrap`. One ephemeral handshake message, nothing
   persisted; rotatable with a coordinated custodian release (all peers must
   run the same release for a joining node to bootstrap).

## Adding a new key

1. Pick the layer by IKM: root key → `KeyPurpose`; ECDH secret →
   `AesKeyDomain`; anything else → document it here as a new layer.
2. Add an enum variant with a fresh, versioned, self-describing label
   (`seismic/<context>/<algorithm>/v1`). Never reuse an existing label.
3. Mirror the variant in the TS/Python registries only if clients
   participate in the protocol, and extend the cross-language known-answer
   tests.
4. Add the row to this file.
