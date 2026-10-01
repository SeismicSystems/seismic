# Root-key commitment (decision record)

> **Decision record — September 2026.** A point-in-time capture of the design
> pass that settled how a verifier knows a `root_key` is the network's. It is
> not updated as the design moves; the current-state docs it links are the
> authority on what runs today. It supersedes parts of
> [the roots-of-trust record](2026-08-roots-of-trust.md), listed in
> [what this supersedes](#what-this-supersedes).

**Outcome: `root_key` is minted before the manifest, and the manifest pins
`tx_io_pk@0`, so `network_id` commits to the key.** Every founding box's
custodian mints a candidate at identity-free boot, harvest quotes each
candidate's `tx_io_pk@0`, and assemble pins one. A custodian keeps its
candidate only if it matches the pin, and a joiner's custodian refuses to
install a fetched key that does not match. That settles epoch 0 for both
parties that ask: a node joining, and a client encrypting. Key epochs above 0
are not settled here; the direction is a record signed by the validator set,
with the epoch-0 pin as its base case. The mechanisms are in
[network-manifest.md](../network-manifest.md),
[architecture.md](../architecture.md), and
[trust-model.md](../trust-model.md). As of this record they are decided, not
built.

- [Who asks](#who-asks)
- [The rule](#the-rule)
- [The candidates](#the-candidates)
  - [The addendum's second-mint forgery](#the-addendums-second-mint-forgery)
  - [The other rejected anchors](#the-other-rejected-anchors)
  - [The validator-signed record, for later epochs](#the-validator-signed-record-for-later-epochs)
  - [Not pinned: a root-derived signing key](#not-pinned-a-root-derived-signing-key)
- [Mint-first](#mint-first)
- [Left open](#left-open)
- [Who plays which role](#who-plays-which-role)
- [What this supersedes](#what-this-supersedes)
- [References](#references)

## Who asks

One question, "is this the network's key?", has two askers:

| | Joiner | Client |
| --- | --- | --- |
| Receives | `root_key`, wrapped, from a responder | `tx_io_pk`, from whichever RPC it uses |
| Checks before this decision | nothing: the joiner admits any genuine Azure TDX guest ([`DangerouslyAdmitAnyAzureGuest`](https://github.com/SeismicSystems/enclave/blob/seismic/bin/attestation-service/src/join.rs)) | nothing: the SDKs take `seismic_getTeePublicKey` on first use ([`fillers.rs`](https://github.com/SeismicSystems/seismic-alloy/blob/seismic/crates/network/src/fillers.rs)) |
| Attacker | the joiner's own host, with one genuine TDX VM on any image | the RPC operator, a DNS or TLS hijacker, or a node that was itself fooled; no TEE needed |
| A wrong key costs | the node's first-boot disk: its summit keys leave the TEE for good, and it then serves the wrong `tx_io_pk` to its clients | that client's calldata and signed-read responses |
| Later epochs, from an epoch-0 pin | covered: every epoch re-derives from `root_key` | not covered: one pin per epoch |

The two are linked. A fooled joiner on an accepted image produces genuine
tx-io evidence, a real quote over `tx_io_binding(real network_id, wrong
tx_io_pk, 0)`, so any client check that rests on node evidence is sound only
once joiners are checked.

## The rule

`network_id` is a hash, so it can vouch only for what exists when it is
computed. It is trusted because it is published and cross-checked everywhere,
in the committed network directory, SDK chain definitions and explorers, and
there is no holder to compromise and nothing to rotate. The price is that it
cannot sign anything later.

**A verifier trusts only what `network_id` commits to, or what an authority it
commits to signs.** Facts fixed at founding are committed directly: the reth
genesis, the summit genesis and its validator set, the bootstrap policy. Facts
that change later are reached through a committed authority: the live
admission policy is registry storage on the chain `eth.genesis_hash` names,
moved only by the authority the manifest pins. Direction matters. A thing
`network_id` commits to is unique; a thing that merely names `network_id` is
not.

## The candidates

Each candidate answer to "is this the network's key?", judged for both askers
and for key epochs above 0:

| # | Anchor | Joiner, epoch 0 | Client, epoch 0 | Epochs > 0 | Verdict |
| --- | --- | --- | --- | --- | --- |
| 1 | nothing (before this decision) | ✗ | ✗ | ✗ | the hole |
| 2 | the holder's quote, checked against an allowlist | ✗ a manifest allowlist goes stale at the first image rotation | ✗ vTPM and DCAP verification in every SDK; the live registry is readable only through the untrusted RPC; unsound until joiners are checked | ✗ same | rejected |
| 3 | the addendum: a pin created after `network_id` | ✗ forgeable by a second mint under the real manifest | ✗ same, plus quote verification in the SDKs | ✗ its quote is appraised against a bootstrap policy that is stale by then | rejected |
| 4 | mint-first: the pin inside the manifest | ✓ the custodian compares | ✓ a hash comparison | the base case only | adopted for epoch 0 |
| 5 | a CCF-style network identity key | needs its own pin (3 or 4) | ✓ | ✗ every custodian holds it, so an exit does not revoke it | rejected |
| 6 | a record signed by the validator set | needs 4 for the base case | a light client | ✓ the only anchor that evolves with the network | the direction for epochs > 0, open |
| 7 | K-of-N agreement across responders | ✗ the host controls every peer the joiner sees | ✗ one party can run many RPCs | – | rejected |
| 8 | a pin signed by Seismic, out of band | – | ✓ | ✓ | rejected: operator-vouched |

### The addendum's second-mint forgery

The August design pinned `tx_io_pk@0` in `network-attestation.json`, an
addendum produced after the genesis node's first boot and checked against the
manifest: a quote over `tx_io_binding(network_id, tx_io_pk, 0)` from an image
in the bootstrap policy. The addendum names `network_id`; `network_id` does not
name it. Any host can therefore:

1. Boot a fresh box on an image in the bootstrap policy.
2. POST the real manifest with the genesis flag set. Nothing on a node stops a
   second mint; one minter per network was tooling discipline.
3. Read that box's tx-io evidence: a genuine quote from an accepted image over
   `tx_io_binding(real network_id, tx_io_pk(R′), 0)`. It passes every check the
   addendum specified.
4. Hand it to its own joiner, or to clients, with the R′ box as the responder.

Binding the genesis box's pinned summit key into the addendum quote, and
marking that box in the manifest, would patch it. That requires knowing the
minting box at harvest, which is half of mint-first. Rotating the addendum
under the same `network_id` after a recovery, which the August record adopted,
cannot be told apart from this forgery, and is withdrawn.

### The other rejected anchors

**The allowlist (2).** A joiner that appraises its responder against a
manifest-pinned measurement list rejects honest responders on every image
admitted after founding, and keeps accepting a founding image after it is
deprecated, since deprecations live on chain. For a client, the allowlist
means vTPM and DCAP verification inside every SDK, and the live list is
readable only through the RPC the client is trying not to trust. And a fooled
joiner on an accepted image attests the wrong key, so the client check is
sound only after the joiner check exists.

**The identity key (5).** A dedicated network keypair would let clients check
a signature rather than evidence. Its public half needs the same pin as
`tx_io_pk@0`, so it adds a key without removing the anchor question. Every
custodian holds the private half, so an operator that exits keeps a TEE able
to sign for the network, and nothing revokes it. Validators are the better
signer for anything that changes after founding.

**K-of-N agreement (7).** Requiring several responders to deliver the same key
needs no anchor, but the joiner's host chooses every peer the joiner reaches.
Agreement across RPCs raises the bar for a client, but one party can run many
RPCs.

**A Seismic-signed pin (8).** It covers every epoch, and it is the trust in an
operator that the TEE exists to remove. Combined with mint-first, what Seismic
would sign for epoch 0 reduces to `network_id`, which it publishes anyway.

### The validator-signed record, for later epochs

A record of a new key at a bumped epoch, in a block finalized by at least
two-thirds of that epoch's validators, is the only anchor that moves with the
network. The founding validator set is committed by `network_id` through the
summit genesis, and each later set is reached from it through finalized
validator-set changes, so the record is signed by an authority `network_id`
commits to. Execution can enforce that a tx-io bump is well formed. Summit's
finalized header carries `payload_hash`, `execution_request_hash` and the
validator-set changes, and
[`verify_checkpoint_chain_with_weak_subjectivity`](https://github.com/SeismicSystems/summit/blob/main/types/src/checkpoint.rs)
walks the chain from a trusted checkpoint.

Two constraints shape it. The record cannot be contract storage proven by an
`eth_getProof` Merkle proof, because seismic-reth disables that method: proofs
of public slots help brute-force private slots in the same contract
([`node.rs`](https://github.com/SeismicSystems/seismic-reth/blob/seismic/crates/seismic/node/src/node.rs)).
And a verifier needs a light client, which is not built. The base case needs
mint-first's pin, the way summit's genesis validator set is the base case for
`added_validators` and `removed_validators`.

### Not pinned: a root-derived signing key

A signing key derived from `root_key`, pinned next to `tx_io_pk@0`, would let
SDKs check that a tx-io bump came from the network before a light client
exists. It proves authenticity, not currency: an old signed bump stays valid.
It can only be pinned at founding, so a network founded without it cannot adopt
it later. It is left out, so that the epoch-0 decision does not depend on the
rotation decision.

## Mint-first

- **Every box mints a candidate.** The custodian mints a candidate `root_key`
  at identity-free boot, before the config POST. The custodian has no network
  listener, so the key holder's harvest endpoint relays the candidate's
  `tx_io_pk@0`, and the harvest quote covers it along with the summit pubkeys.
- **Assemble pins one.** Deploy DCAP-verifies the harvest as before and pins
  one box's `tx_io_pk@0`, so `network_id` commits to it.
- **The pin decides who keeps a key.** At configure, a custodian keeps its
  candidate only if it matches the pin. Otherwise it discards the candidate and
  fetches `root_key` from a peer. The genesis flag goes away.
- **The custodian checks what it installs.** After unwrapping a fetched key,
  the custodian re-derives `tx_io_pk@0`, compares it with the pin read from the
  manifest bytes tdx-init wrote to tmpfs, and refuses a mismatch. A compromised
  attestation service cannot install a key of its own choosing.
- **Clients check a hash.** A client pins `network_id`, hashes the manifest it
  is given, and compares `tx_io_pk@0`. No quote verification ships in the
  SDKs.

**Where the pin lives: the manifest, with summit to follow.** Both places
are committed by `network_id`. A manifest field is what the custodian already
reads, and what a client that only hashes the manifest can check. The summit
genesis, next to the validator set, is where later key records would start,
but putting the key there needs summit's SSZ digest in the guest and in every
SDK, and a summit change. So the pin is a manifest `root_key` section,
`{ root_version: 0, epoch: 0, tx_io_pk }`, the shape of a key record, and the
summit genesis gets the same record later, held in agreement with the
manifest the way the bootstrap policy is held to the registry's genesis
storage. The manifest schema changes in place under `manifest_version = 1`,
since no permanent network pins a v1 manifest yet. Assemble pins the first
harvested box by name.

**Why `tx_io_pk@0`, not a hash of `root_key`.** Both bind `root_key` equally,
but only `tx_io_pk` is usable by a client, which encrypts to it. And epoch 0
is then the base case of the record later epochs would use.

**What it buys beyond the check:**

- No genesis flag, so no stale flag and no second minter under the real
  `network_id`: a second candidate cannot match a pin `network_id` commits to.
  A rebooted genesis node rejoins as a joiner. The custodian that honors the
  founding policy becomes the one whose candidate matched.
- The check lives in the custodian and reads its pin from tmpfs, so the
  joining side of the handshake no longer trusts the network-facing process to
  appraise the responder.
- Once the check exists, the responder's quote carries no weight for the
  joiner. No identity key is needed to drop it.
- The founding archive's DCAP collateral keeps the harvest quote over
  `tx_io_pk@0` re-verifiable, so the addendum's validity-at-creation machinery
  is not needed for epoch 0.

**What it costs.** The burned-key rule becomes critical. Before mint-first, a
first-POST attacker gets one box's summit keys. Under mint-first the genesis
box already holds the future `root_key`, so an attacker who POSTs first, with a
manifest that pins that box's candidate and admits the attacker's own image,
can extract it. The rule still catches it, since the real configure is then
refused, but ignoring that failure now costs the network's privacy rather than
one validator. Deploy must refuse to continue when the genesis box's configure
fails. The custodian also has to start before the config POST, which reorders
the boot chain.

## Left open

- **Epochs above 0**, including fresh-entropy rotation and recovery. The
  direction is the validator-signed record above. Until an anchor exists, no
  nonzero epoch ships.
- **Founding integrity.** The check proves the key matches the manifest the
  node was given, not that the node was given the real manifest. After genesis
  that is enough, since a wrong `network_id` is visible downstream. It is not
  enough for founding validator keys, which predate the manifest: a malicious
  founding orchestrator can configure a box with a fake manifest and get its
  pinned keys onto a disk it can read. Binding a deploy key into the harvest
  quote, so tdx-init accepts only a POST signed by it, is the candidate.
- **Runtime-register hardening is orthogonal.** Extending a runtime
  measurement register with the manifest hash was closed as a documented
  option. It would change how `network_id` is bound into a quote, not what
  `network_id` commits to.

## Who plays which role

Three systems with a network-wide secret inside TEEs, set against Seismic.
The comparison is by role, not by mechanism: which artifact names the network,
who holds the secret, and what each asker checks. Each cell is sourced in
[references](#references).

| | CCF | Secret Network | Oasis Sapphire | Seismic |
| --- | --- | --- | --- | --- |
| Inside the TEE | the whole node | contract execution; Tendermint runs on the host | the ParaTime runtime and the key-manager runtime; CometBFT runs on the host | the whole guest, consensus included |
| Names the network | `service_cert.pem`, the service identity certificate the first node creates | the chain's `genesis.json` | the consensus layer's genesis | `network_id`, a hash of the manifest |
| Network-wide secret | the ledger secrets and the service private key, shared by every trusted node | the consensus seed, sealed to MRSIGNER on each node | master secrets, held by the key-manager committee | `root_key`, in RAM, in every node's custodian |
| What vouches for it at the start | nobody signs the certificate: operators distribute it out of band, and the members' `transition_service_to_open` vote opens the service and records it on the ledger | the seed-exchange and I/O public keys in `genesis.json` | a key-manager policy, valid only when signed by a threshold of keys compiled into the key-manager enclave | `network_id` commits to `tx_io_pk@0` |
| A joining node gets the secret by | a join request whose quote binds its node key; the node it joins checks the quote and answers with the secrets. Its anchor is `service_certificate_file`, from operator config | a registration transaction carrying its attestation, verified in every node's enclave as the transaction executes; the seed comes back encrypted by ECDH against the seed-exchange key in genesis | replication from other key-manager enclaves, under the policy registered on the consensus layer | an attested peer handshake in which the responder reads the on-chain registry; its custodian checks the key against the pin |
| A client gets the encryption key from | none exists: clients use TLS to nodes whose certificates the service key endorses | `consensus_io_exchange_pubkey`, derived from the seed and published in genesis; secret.js fetches it from an endpoint and checks nothing | a per-epoch ephemeral key signed by the key manager's runtime signing key (RSK), which the consensus layer publishes; the reference clients do not check the signature | `tx_io_pk@0`, hash-checked against `network_id` |
| A key change is authorized by | members accepting a recovery, after which the service certificate is new and must be redistributed | the 2023 seed rotation was served by one Secret Labs seed server; nodes keep the genesis seed and the current one | key-manager nodes agreeing on the next generation's checksum, which is chained on the consensus layer | open; the direction is the validator set |

What the table shows:

- **CCF's joiner anchor is operator config**, and CCF can afford that because
  a CCF joiner brings nothing with it. A Seismic joiner brings TEE-born
  validator keys and then serves clients, so its anchor has to be the hash
  everyone pins.
- **For key changes, Seismic's validators play the role of CCF's members**, and
  `network_id` plays the role of the startup configuration. CCF's identity is a
  key and can vouch after the fact; `network_id` is a hash and cannot, which is
  why the epoch-0 key goes inside it.
- **Secret and Sapphire are the closer match for clients**, since both have
  clients encrypt to a network key derived from the secret. In both, the
  reference clients trust the endpoint they fetch it from, which is the hole
  the hash check closes for Seismic at epoch 0.
- **Sapphire's RSK is the root-derived signing key this record leaves out**:
  derived from the master secret, with its public half on the consensus layer.
  Seismic's validator-signed record would be read the same way, by a light
  client over public headers.
- **Secret's registration is checked on chain because its chain is public.**
  A new Secret node syncs the chain and registers through it. A Seismic node
  cannot sync its way in, since its disk and its history both need `root_key`.
- **Secret's seed rotation was operator-vouched**, the shape of candidate 8.

## What this supersedes

In [the roots-of-trust record](https://github.com/SeismicSystems/seismic/blob/a5ccda2/docs/tee/decisions/2026-08-roots-of-trust.md),
cited by line at that commit:

- **L12, L33:** clients are missing as a second party asking for the key, and
  "two anchors" is one root, `network_id`, reached by two paths.
- **L27:** `network_id` in `report_data` stops replay across networks, not an
  impostor running the same manifest.
- **L55, L68, L70:** the commitment proves the key is canonical only if the pin
  is unique. The L68 caveat points the wrong way: the gap is that `network_id`
  does not commit to `tx_io_pk`, not that `tx_io_pk` does not commit to
  `network_id`.
- **L63:** runtime-register hardening is orthogonal, and was closed as a
  documented option ([left open](#left-open)).
- **L71:** once the commitment check exists, the responder's quote carries no
  weight; no identity key is needed to drop it.
- **L75:** "minted at genesis, public half pinned in the manifest" is an
  impossible ordering, unless the key is minted before the manifest, as
  `root_key` now is.
- **L113:** the light-client objections apply only to nodes. LUKS encrypts a
  node's data at rest; headers and finality certificates are public, so a
  client can run a light client with no `root_key`.
- **L126:** rotating the addendum pin under the same `network_id` cannot be
  told apart from a forgery. Withdrawn.
- **L130:** the custodian now checks one thing itself, the installed key
  against the pin, so it no longer relies on the attestation service for the
  joiner's appraisal.

## References

CCF:

- [Cryptography](https://ccf.dev/main/architecture/cryptography.html): the
  service and node identity keys, the secrets every trusted node shares, and
  the join quote over the node's public key.
- [Starting a network](https://ccf.dev/main/operations/start_network.html):
  `service_certificate_file` as the first node's output and the joiner's only
  trust anchor.
- [Opening the network](https://ccf.dev/main/governance/open_network.html):
  `transition_service_to_open`.
- [Recovery](https://ccf.dev/main/operations/recovery.html) and
  [accepting a recovery](https://ccf.dev/main/governance/accept_recovery.html):
  a new service identity after recovery.

Secret Network:

- [the trusted core](https://docs.scrt.network/secret-network-documentation/introduction/secret-network-techstack/privacy-technology/intel-sgx/trusted-core):
  what runs in the enclave.
- [the bootstrap process](https://docs.scrt.network/secret-network-documentation/introduction/secret-network-techstack/privacy-technology/encryption-key-management/bootstrap-process)
  and [full-node bootstrap](https://docs.scrt.network/secret-network-documentation/introduction/secret-network-techstack/privacy-technology/encryption-key-management/full-node-boostrap):
  the consensus seed, its sealing, and registration.
- [transaction encryption](https://docs.scrt.network/secret-network-documentation/introduction/secret-network-techstack/privacy-technology/encryption-key-management/transaction-encryption)
  and [secret.js `encryption.ts`](https://github.com/scrtlabs/secret.js/blob/master/src/encryption.ts):
  `consensus_io_exchange_pubkey` and how the client fetches it.
- [consensus seed rotation](https://docs.scrt.network/secret-network-documentation/introduction/secret-network-techstack/privacy-technology/encryption-key-management/consensus-seed-rotation):
  the 2023 rotation and its seed server.

Oasis Sapphire:

- [the key manager](https://docs.oasis.io/core/consensus/services/keymanager/):
  the committee, replication, and the signed policy.
- [ADR 0022](https://docs.oasis.io/adrs/0022-keymanager-master-secrets/) and
  [oasis-core `secrets/api.go`](https://github.com/oasisprotocol/oasis-core/blob/master/go/keymanager/secrets/api.go):
  master-secret generations, the checksum chain, and the RSK in the published
  status.
- [ADR 0021](https://docs.oasis.io/adrs/0021-keymanager-ephemeral-secrets/)
  and [the JS client's `calldatapublickey.ts`](https://github.com/oasisprotocol/sapphire-paratime/blob/main/clients/js/src/calldatapublickey.ts):
  the per-epoch calldata key, its RSK signature, and a client that does not
  check it.

Seismic code cited above:

- [SeismicSystems/enclave `join.rs`](https://github.com/SeismicSystems/enclave/blob/seismic/bin/attestation-service/src/join.rs):
  the joiner's accept-any responder policy.
- [SeismicSystems/seismic-alloy `fillers.rs`](https://github.com/SeismicSystems/seismic-alloy/blob/seismic/crates/network/src/fillers.rs):
  the client's unauthenticated `seismic_getTeePublicKey` fetch.
- [SeismicSystems/seismic-reth `node.rs`](https://github.com/SeismicSystems/seismic-reth/blob/seismic/crates/seismic/node/src/node.rs):
  `eth_getProof` disabled.
- [SeismicSystems/summit `checkpoint.rs`](https://github.com/SeismicSystems/summit/blob/main/types/src/checkpoint.rs):
  the checkpoint chain walk.
