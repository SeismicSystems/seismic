# TEE roots of trust (decision record)

> **Decision record — August 2026.** A point-in-time capture of the
> roots-of-trust design pass. It describes the candidates as they stood when the
> decision was made and is not updated as the design moves; the current-state
> docs it links are the authority on what runs today.

Decision record for the roots-of-trust design pass: the candidate trust anchors that were cataloged, and the reasoning that picked the shipped combination. **Outcome: the network manifest as the joiner's bootstrap root, the** `tx_io_pk@0` **commitment as proof of key provenance, and genesis-pinned on-chain policy for responder admission.** That combination is the adopted design, documented publicly — the mechanisms in [network-manifest.md](../network-manifest.md) and [chain-backed-admission.md](../chain-backed-admission.md), the assumptions and accepted risks in [trust-model.md](../trust-model.md). This record keeps the candidates that lost and why, for when an open decision (disaster recovery, service identity, two-phase admission) is revisited.

# The problem

TEE attestation is not itself the whole root of trust for a Seismic network. A quote proves hardware-backed execution of some measured software and a transcript binding. Seismic still needs a root-key-free way to answer: "is this the canonical network I intend to join/use?"

The Red Hat framing (confidential-computing trust as a pipeline, **REMITS**): Root of trust → Endorsement → Measurement → Identity → Trust → Secrets. The attestation backend covers steps 1–4. Concretely, the root-key bootstrap handshake is one full REMITS pass:

| REMITS step | Seismic mechanism |
| -- | -- |
| Root of trust | Intel TDX hardware; Microsoft's Azure vTPM AK root CAs |
| Endorsement | AK cert-chain + DCAP collateral verification (attested-tls backend) |
| Measurement | measured boot → quote-authenticated vTPM PCR bank ([azure measurements](<https://github.com/SeismicSystems/seismic-images/blob/seismic/docs/azure-measurements.md>)) |
| Identity | verified PCRs → `(pcr4, pcr9, pcr11)` → admission ID ([`seismic-measurement-admission`](https://github.com/SeismicSystems/enclave/tree/seismic/crates/measurement-admission)) |
| Trust | the admission predicate: [`MeasurementRegistry.isAccepted(id)`](../chain-backed-admission.md) on local reth (responder side) |
| Secrets | `root_key` AEAD-wrapped to the requester's attested ephemeral key |

REMITS's Identity/Trust separation is the same seam [`seismic-attestation`](https://github.com/SeismicSystems/enclave/tree/seismic/crates/attestation)'s verify-with-admission API draws: cryptographic verification yields typed measurements (Identity), the admission predicate is the separable policy appraisal (Trust).

The difficult part is step 5, actual network trust. A valid quote with accepted measurements does not prove: canonical-network membership, possession of the correct `root_key`, economic admission, a current chain view, or "not a fresh copy of the right image configured for the wrong fork". Anyone can run the correct image with the same `chain_id` or even the same manifest hash. Hence binding `network_id = H(canonical NetworkManifest)` ([the manifest](../network-manifest.md)) into `report_data` — which prevents cross-network replay but still needs a public, root-key-free source of truth for the manifest itself, policy, key provenance, and admission.

## Why chain state cannot be the joiner's bootstrap source

In networks where consensus state is public and independently syncable, a new node syncs the chain, reads enclave policy, verifies peers against it, then requests the secret. Seismic's execution state is coupled to `root_key`: reth's DB is on the [root-key-derived LUKS volume](../architecture.md#persistent-the-luks-volume), and historical TxSeismic replay requires `root_key`-derived keys ([why a node cannot sync its way in](../architecture.md#client-traffic-encrypting-to-tx_io_pk)) — so a contract is not a sufficient bootstrap source for a node that cannot first obtain `root_key`: reading the policy would require the very secret the policy gates.

That circularity binds only one side. Any node in a position to release `root_key` already holds it and has a readable local chain — including the genesis node at t=0 — so a genesis-pinned contract IS a sufficient admission-policy source from block 0: the responder's anchor is live chain state, with no cycle anywhere. Only the joiner needs a root-key-free anchor, and its question — "is this the canonical network?" — never needed chain state: the manifest answers it. The price is two anchors instead of one, which the [trust model](../trust-model.md#the-trust-anchor-per-action) states as a structural asymmetry: the responder gets the live anchor, the joiner the frozen one.

# The five contests

The pass cataloged nine candidate roots of trust. They were never nine rivals for one job: they competed in five separate contests — the trust questions from the trust model's [per-action anchor table](../trust-model.md#the-trust-anchor-per-action) whose anchor was actually in dispute. Candidates from different contests compose rather than exclude each other, which is why the shipped design combines three.

| Candidate root | Lives in | Contest | Outcome |
| --- | --- | --- | --- |
| network manifest | a document, hash-pinned out of band | [the joiner's anchor](#the-joiners-anchor) | ✅ adopted |
| `tx_io_pk@0` commitment | key material, derived from `root_key` | [the joiner's anchor](#the-joiners-anchor) | ✅ adopted |
| network identity key | key material, a dedicated keypair | [the joiner's anchor](#the-joiners-anchor) | ❌ deferred |
| genesis-pinned on-chain policy | chain state, execution layer | [the responder's policy source](#the-responders-admission-policy-source) | ✅ adopted |
| signed epoch manifests | a document, committee-signed | [the responder's policy source](#the-responders-admission-policy-source) | ❌ rejected as end-state |
| governance contract mirrored into Summit | chain state, execution layer | [registry mutation authority](#registry-mutation-authority) | ❌ open decision |
| Summit public control plane | chain state, consensus layer | [freshness evidence](#freshness-evidence-beyond-the-host) | ❌ open decision |
| external key-manager / KBS | an external party: a service | [disaster recovery](#disaster-recovery) | ❌ rejected near-term |
| threshold recovery shares | an external party: a member quorum | [disaster recovery](#disaster-recovery) | ❌ deferred |

Two patterns are worth naming. The "lives in" families tell the outcome in one line: one document root and one key root shipped — both for the joiner, who cannot read the chain; chain state was adopted exactly where a reader exists (the responder) and parked where one does not; the external-party family was deferred wholesale to disaster recovery. And every ❌ is parked against a named open decision in the trust model — the contests, not the candidates, are what the open-decision list tracks.

# The joiner's anchor

The joiner's question: is this the canonical network, and is the delivered key its key? A joiner holds nothing yet, so the anchor must be root-key-free ([why chain state cannot be it](#why-chain-state-cannot-be-the-joiners-bootstrap-source)). The adopted answer is two complementary halves — the manifest names the network, the commitment proves the delivered secret is that network's — carried by the mutually attested root-key handshake, whose transcript binds nonce, ephemeral pubkeys, and `network_id` on both sides ([the handshake](../architecture.md#the-root-key-handshake)).

## Network manifest as bootstrap root ✅ adopted

Deploy artifacts (`network-manifest.json`, `measurement-policy.json`) written by [tdx-init](../architecture.md#the-processes) to `/run/seismic/conf/`; peers verify against the local artifacts pre-chain; once live, on-chain policy is the live source.

* Pros: the simplest root-key-free anchor for the joiner; matches the startup-config-as-trust-anchor of Microsoft's [Confidential Consortium Framework (CCF)](https://microsoft.github.io/CCF/); easy with existing deployment artifacts.
* Cons: relies on deploy/operator distribution hygiene; does not by itself prove a responder holds the canonical `root_key`; stale manifests need care.
* Hardening (still open): tdx-init extends a runtime measurement register (RTMR3 / designated vTPM PCR) with `H(manifest)` before fan-out — CCF's `host_data` analogue — making the POSTed config hardware-attested.
* Adopted for: joiner-side network authentication, deploy verification, genesis policy pinning. Not responder admission policy — that reads the genesis-pinned contract from block 0.

## Public `root_key` commitment ✅ adopted

No service identity. The requester decrypts the delivered `root_key` and checks it against a public commitment. `tx_io_pk@0` **is already the commitment**: a binding, deterministic public function of `root_key`, already published for TxSeismic clients — pin it at epoch 0 (in the attested addendum) and the joiner re-derives and compares. No separate commitment construction needed. As of this record the addendum and the joiner's check are specified in [network-manifest.md](../network-manifest.md#the-attested-addendum), not yet built. Caveat: `tx_io_pk` commits to `root_key` but not to `network_id`, so `network_id` stays bound in the transcript separately (it is).

* Pros: no additional shared identity key; directly proves the received secret is canonical; safe for a high-entropy `root_key`.
* Cons: the requester still needs a public root for the commitment (the addendum); malicious responders can cause failed decrypt/check attempts; fresh responder attestation stays in the hot path unless combined with a service identity.

## Network identity key ❌ deferred

A CCF-like network identity keypair minted at genesis, public half pinned in the manifest, private half held by the custodian; responders authenticate by signing transcripts.

* Pros: compact membership proof; avoids fresh responder attestation per interaction; a stable cryptographic network name independent of per-node TLS certificates.
* Cons: a second network-wide secret with large impersonation blast radius; needs generation/distribution/custody/recovery machinery; must live in the custodian.
* **Do not reuse** `tx_io` **for this.** Three compounding reasons: (1) key reuse across purposes — one secp256k1 key doing ECDH-decrypt and signing breaks the [per-purpose HKDF domain separation](../architecture.md#per-purpose-keys) the custodian enforces; (2) `tx_io = f(root_key)` deterministically, so a `tx_io_sk` signature proves possession of `root_key` and nothing else — for a joiner, who receives `root_key` and runs the commitment check directly, that adds nothing — while a *dedicated* identity key's value is being independent of `root_key` (own rotation schedule, survives recovery, can endorse node certs); (3) it is the most-exposed network key — under pass-plaintext custody every node's reth holds `tx_io_sk` for its process lifetime, so making it the impersonation anchor inverts the "narrowly held" property the identity key exists for.
* If ever adopted: lives in the custodian as a new method, not a redesign.

# The responder's admission-policy source

The responder's question: may this requester join the trust domain? The responder holds `root_key` and a readable chain by definition, so — unlike the joiner — a live on-chain source is available to it from the network's first moment.

Still open in this contest's scope: whether read-only confidential full nodes should receive `root_key` at all, and what pre-root identity staking should register — parked with the trust model's post-genesis validator-key-binding [open decision](../trust-model.md#open-decisions).

## Genesis-pinned on-chain policy ✅ adopted

Initial accepted measurements compiled into the registry's genesis storage, so `eth.genesis_hash` commits to the initial policy; responders check `isAccepted()` on local reth from block 0; no local-artifact policy path on the responder; serving gated on reth being up and fresh ([chain-backed admission](../chain-backed-admission.md)).

## Signed operator/governance epoch manifests ❌ rejected as end-state

M-of-N committee publishes signed policy manifests per epoch. Simple verifier, usable interim, good emergency fallback — but centralized, and stale-manifest/revocation handling is hard. The variant shaped like [The Update Framework (TUF)](https://theupdateframework.io/) — signed complete policy documents accepted at any revision ≥ bootstrap — was also rejected on the joiner side: rollback/freeze protection needs freshness evidence, which is chain state again (see [the manifest doc's rationale](../network-manifest.md#design-rationale), "No joiner-side policy refresh").

# Registry mutation authority

Governance's question: who may change the accepted measurement set? Open — the manifest pins an authority contract, and a dev authority fills the role today; the mainnet authority is the trust model's registry-mutation [open decision](../trust-model.md#open-decisions).

## EVM governance contract mirrored into Summit ❌ not pursued near-term

Governance UX on the execution layer (typed policy contract), mirrored into Summit via the execution-request path. Initial values must still appear in genesis — the contract cannot solve first-bootstrap.

# Freshness evidence beyond the host

The cross-cutting question: how does a guest know its chain view is current, when its disk, clock, and network all arrive through the host? This is the one contest that is not an action of its own — it is a property two other anchors demand: the responder's policy read, and the exit from the trust model's [rollback family](../trust-model.md#the-rollback-family).

## Summit public control plane / light-client root ❌ not pursued near-term

Move CCF-style public control-plane state (policy hashes, commitments, admission identities) into Summit, verified via checkpoints/light-client proofs.

* Pros: closest analogue to Secret/Oasis public consensus roots; supports permissionless economic admission cleanly.
* Cons: requires Summit to boot/sync/serve proofs before `root_key`; needs finality/freshness semantics; summit's keys live behind root-key LUKS today, so the [storage cycle](../architecture.md#boot-power-on-to-serving) must be broken or narrowed.
* Still relevant to: the rollback family's exit (verifying summit finality signatures against the manifest-pinned validator set), and post-open economic eligibility.

# Disaster recovery

The question: how does the network outlive losing every TEE at once? The current answer is that it does not — `root_key` is RAM-only, and at least one node must stay live. Both candidates were parked with the trust model's pre-mainnet disaster-recovery [open decision](../trust-model.md#open-decisions), because each trades the RAM-only property for a new trusted party.

## External TEE key-manager / KBS committee ❌ rejected near-term

A separate service/committee that owns or reconstructs `root_key` and releases it after attestation and policy checks (Confidential Containers KBS shape). Removes the "one TEE must remain live" assumption and isolates custody, but is a substantial new TCB with its own governance, attestation, and recovery, and a centralization point unless thresholded.

## Threshold recovery shares ❌ deferred

CCF-style: split a wrapping key for `root_key` into shares encrypted to recovery members. Avoids permanent bricking; changes the trust model (threshold members can recover the network secret); needs share refresh and recovery transcripts. CCF also ships an experimental *sealing-based* recovery variant (hardware-sealed secrets + automatic recovery decision protocol) that avoids the member-threshold change — evaluate it before inventing a Seismic-specific scheme. Note CCF deliberately rotates the service identity on every recovery and forces cert redistribution, making recovery client-visible; Seismic's analogue is rotating the addendum's `tx_io_pk` pin under the same `network_id` — adopted into the addendum's recovery semantics.

# Relationship to the custodian split

None of the candidates invalidate the [custodian split](../architecture.md#key-custody-one-process-holds-root_key): the untrusted-evidence parser lives on the responder side of any peer bootstrap in every model, and the custodian receives a pre-verified authorization, never raw evidence — so which root of trust produced it is swappable. Two clarifications: (1) admission and secret-release are the same exchange for a first join — verifying this joiner's quote against policy IS the release decision; a responder signature (the identity key) or the delivered-key commitment check only removes the joiner-side parse of the responder's evidence ("mutual → one-directional") — the responder still parses the joiner's quote. (2) Only two-phase admission separates them: verify once → durable credential → cheap checks on later fetches. Attractive because `root_key` is RAM-only and re-fetched every reboot; trades measurement freshness for revocation/expiry machinery. Deferred.

# References

* [https://www.redhat.com/en/blog/confidential-computing-root-trust-actual-trust](<https://www.redhat.com/en/blog/confidential-computing-root-trust-actual-trust>)
* [trust-model.md](../trust-model.md) — the published durable content
* [attestation crate README](https://github.com/SeismicSystems/enclave/blob/seismic/crates/attestation/README.md) — the attestation layer's standing constraints
