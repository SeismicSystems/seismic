# The TEE Trust Model <!-- omit in toc -->

**Status**: current as of 2026-09. Describes the shipped design plus the
pieces that are specified but not built, each marked. The open decisions at
the end are the live list; an entry leaves it by becoming a mechanism in a
sibling doc.

What the TEE design trusts, what an attestation quote does and does not
prove, and which risks are accepted. The mechanisms themselves live in the
sibling docs — [the node](architecture.md), [the
manifest](network-manifest.md), [founding](network-founding.md), and
[admission](chain-backed-admission.md) — and nothing here is normative. This
doc states the assumptions behind those mechanisms in one place, so the
accepted risk can be reviewed as one list rather than reassembled from a
caveat per mechanism.

- [Summary](#summary)
- [Assumptions](#assumptions)
  - [The host platform, and what it is trusted for](#the-host-platform-and-what-it-is-trusted-for)
  - [What a valid quote proves — and what it does not](#what-a-valid-quote-proves--and-what-it-does-not)
- [Identities, and how they evolve](#identities-and-how-they-evolve)
  - [Client trust models](#client-trust-models)
- [The trust anchor, per action](#the-trust-anchor-per-action)
- [Residuals](#residuals)
  - [Accepted risks](#accepted-risks)
  - [The rollback family](#the-rollback-family)
- [Open decisions](#open-decisions)
- [Design rationale](#design-rationale)

## Summary

- **A quote proves execution, never membership.** A genuine TDX guest running
  reviewed code is not yet a member of *this* network holding *its* secret.
  The gap is closed by named mechanisms — `network_id` in every transcript,
  the registry, the key commitment — never by the quote alone.
- **The service identity evolves through consensus.** `network_id` fixes the
  lineage; the validator set, the accepted images and the service key are chain
  state, so what a verifier should accept depends on the head it has verified
  ([identities](#identities-and-how-they-evolve)).
- **Each action has one anchor, matched to the actor's position.** The
  responder reads live chain state because it can; the joiner holds a frozen
  manifest because it must. [The table](#the-trust-anchor-per-action) names
  all nine actions and the five parties that take them.
- **The accepted risks are host influence.** A host owns its guest's disk,
  its POSTed config, its network, its clock, and its VM lifecycle, so the
  residuals cluster exactly where a locally checkable witness is the only
  witness available.
- **One trust model across platforms.** Azure TDX confidential VMs today,
  GCP planned. The model is written to the weakest platform the network
  admits, so no property rests on platform-specific durable state — not a
  vTPM counter, not a platform seal ([the host
  platform](#the-host-platform-and-what-it-is-trusted-for)).
- **Rollback is one family, not many bugs.** Any state the guest keeps
  locally can be rewound, and no local witness detects it. The family — and
  its one real exit, freshness evidence a host cannot mint — is
  [collected below](#the-rollback-family).
- **The open decisions have deadlines.** Disaster recovery and the registry's
  mainnet mutation authority must close before mainnet; validator key
  custody must close before staking opens to outside operators; root-key
  rotation must close before any purpose key
  ships a nonzero epoch.

## Assumptions

What the design takes as given, and from whom: the platform whose roots
endorse the hardware, and the statement a quote built on them carries about
the guest.

### The host platform, and what it is trusted for

Seismic runs on Azure TDX confidential VMs today, and GCP confidential VMs
are a planned second platform. One trust model covers both, and it is written
to the weakest platform the network admits: a property that holds on one
vendor's hardware and not another's is not a property this design states.

Three places the platform legitimately shows through:

- **Endorsement roots are per-platform.** A quote chain terminates in roots
  the vendors operate — Intel's DCAP collateral for the TDX quote, and on
  Azure the vTPM AK certificate chain rooted in Microsoft's CAs. Trusting
  them to endorse genuine hardware, and only that, is the base assumption of
  any TEE deployment.
- **Measurement shape, and so identity, is per-platform.** An Azure guest's
  identity is its quote-authenticated vTPM PCR bank; a bare TDX guest's is
  its MRTD/RTMR set. Each platform earns its own admission schema over a
  disjoint ID space, and a verified guest whose attestation type has no
  schema is denied — so a policy naming an Azure image says nothing about a
  GCP one, by construction ([admission](chain-backed-admission.md)).
- **Platform-specific hardening, if ever adopted, is named here.** This
  section is the register for it: the platform, the property it buys, and
  which deployments it applies to. Today it holds the endorsement roots and
  nothing more.

**No security property rests on platform-specific durable state.** A
monotonic counter in vTPM NV storage, or a platform seal, would each be a
tempting local anti-rollback anchor. Both rest on state the host stores and
restores, under clone and rollback semantics the platform defines rather than
the guest — Azure's vTPM state is host-persisted VM state, and every further
platform arrives with its own answer. So the design assumes neither, on any
host: rollback resistance is sought in evidence from beyond the host ([the
rollback family](#the-rollback-family)), never in a counter whose behavior
the vendor decides. An operator whose platform does offer a hardware-anchored
counter may harden that node with it; what the *network* states stays what
holds everywhere, because every admission decision is made about peers spread
across all of it.

### What a valid quote proves — and what it does not

A valid quote with accepted measurements proves three things. The requester
is a genuine TDX guest: the quote chain and platform collateral verify. It is
running reviewed software: the measurements match the accepted set. And it
minted this quote for this exchange: `report_data` carries the transcript
binding. That is a strong statement about *execution*. It is not a statement
about *membership*, and by itself it proves none of the following:

```text
this node is part of the canonical network, not a clone running the same image
this node holds the network's root_key
this node has been economically admitted as a validator
this node's view of chain state is current
```

Anyone can run the correct image and configure it with the same chain id, or
even a copy of the same manifest. The quote truthfully says "this measured
code participated in this transcript"; it cannot say "this is a member of the
network you intend, holding that network's secret." Each of those gaps is
closed by a mechanism the quote plugs into, never by the quote alone:

| The quote cannot prove | What closes the gap |
| --- | --- |
| canonical network, not a clone | every transcript binds `network_id`, recomputed by the verifier from its own manifest ([bindings](network-manifest.md#consumers-of-network_id)) |
| holds the canonical `root_key` | the `tx_io_pk@0` pin that `network_id` commits to ([the root-key pin](network-manifest.md#the-root-key-pin)) — decided, not built |
| economically admitted | the summit genesis at founding, the deposit path afterwards ([founding](network-founding.md)) |
| view of chain state is current | [the freshness gate](chain-backed-admission.md#the-readiness-and-freshness-gate) around the responder's policy read |

## Identities, and how they evolve

A measurement is a machine identity: it names the code a box booted. In a
stateless TEE service it is also the service's identity, because the verifier
chooses the image, or learns the current one from a channel it already trusts.
Here the chain decides which images to accept: the accepted set is registry
state, changed by authority transactions that validators finalize, and the
validator set is chain state too. The measurement a verifier should accept is a
function of time, and the verifier learns it only by following consensus.

So the network has one identity and a succession of values. `network_id` names
the lineage and never changes: it commits to the founding artifacts. The head —
the finalized header at height h, named by its digest — is the network's value
at h, and every head descends from the founding through finalized blocks.
Holding `network_id` tells a verifier which lineage it wants; knowing where that
lineage stands now takes a finalized head.

| Identity | Names | Fixed or evolves | Checked by |
| --- | --- | --- | --- |
| `network_id` | the network's lineage | fixed | SHA-256 of the manifest |
| head | the network's value at h | every block | the finality certificate, against V at h |
| measurement → admission ID | the code a box runs | per image build | a quote, against P at h |
| PCK / vTPM AK | the platform that signed a quote | per machine | Intel and Azure collateral |
| Ed25519 + BLS pubkeys | a validator | per validator | V at h |
| `tx_io_pk@(root_version, epoch)` | where clients encrypt | per rotation or epoch bump | the pin, then later key records |

The control plane at h is three values of the head: the validator set V_h, the
accepted measurement set P_h, and the service key K_h. A box is a member at h
while its measurement is in P_h and it holds the `root_key` K_h commits to; a
validator's keys are also in V_h.

![Identities over time: network_id commits to each lane's initial value; the
validator set, the accepted images and the service key each change by finalized
blocks; machines are members while their image is accepted; and each client
model enters the timeline at a different point](diagrams/identities-over-time.svg)

**Each lane starts in the genesis of the state machine that evolves it.** V_0 is
the summit genesis validator set, moved on by summit's finalized epoch
transitions. P_0 is the registry's storage in the reth genesis, moved on by
authority transactions; the manifest also pins a copy, the bootstrap policy,
only because a joiner cannot read encrypted reth state before it holds
`root_key`. K_0 is `tx_io_pk@0`, pinned in the manifest's `root_key` section
([the root-key pin](network-manifest.md#the-root-key-pin)). By this rule its
home is the genesis of whatever evolves the key series, which is open
([SEI-645](https://linear.app/seismic-systems/issue/SEI-645)); the proposal is
a leaf of summit's Merkleized `ConsensusState`, seeded from the summit genesis
([SEI-656](https://linear.app/seismic-systems/issue/SEI-656)). Every summit
header carries that state's root as `parent_beacon_block_root`, and summit
serves SSZ branches against it, so a record there is provable from one
finalized header and one branch. The manifest's pin then stays as a frozen
copy, like the bootstrap policy: it is what the custodian reads, and what a
client that only hash-checks the manifest needs.

**No quote binds chain state today.** The bindings are:

| Quote | `report_data` binds |
| --- | --- |
| harvest | nonce, summit pubkeys |
| root-key request | `network_id`, nonce, requester ephemeral key |
| root-key response | `network_id`, nonce, responder ephemeral key, wrapped key |
| deploy verification | `network_id`, nonce |
| tx-io evidence | `network_id`, `tx_io_pk`, epoch |

A quote therefore cannot say which head its signer had seen. A host that
eclipses an honest node can hold it on an old head, and the node will attest
what it knows. Proposed
([SEI-653](https://linear.app/seismic-systems/issue/SEI-653)): tx-io evidence
over `(network_id, head digest, head height, head timestamp, root_version,
epoch, tx_io_pk)`. A host cannot mint a fresh finalized header without two
thirds of the validators, so a client that checks the bound timestamp against
its own clock learns the signer's view is recent: the TEE vouches for the
finality check, the validators for the time. It does not tell the client which
images to accept. A deprecated image is the one that cannot be trusted to
report its own standing, so P_h still has to reach the client from outside the
TEE.

### Client trust models

A client chooses where it takes trust from a side channel, and verifies the rest
itself:

| Model | From a side channel | The client verifies | Trusts | Freshness from |
| --- | --- | --- | --- | --- |
| Genesis + light client | `network_id` | the manifest hash, the summit genesis, every epoch's finality certificate since genesis, the key records | two thirds of each epoch's validators | its own clock, against finalized header timestamps |
| Checkpoint + light client | `network_id` and a recent finalized header | the certificates since the checkpoint, the key records | the validators, and the checkpoint's source | its own clock |
| Attested head (proposed) | `network_id` and the accepted measurement set | one quote binding a finalized head and `tx_io_pk`, and the head's timestamp | the platform and the image, the measurement set's source, the validators for the time | the head the quote binds |
| Pinned key | `tx_io_pk@(root_version, epoch)` | nothing | the side channel | the side channel |
| Today | nothing | nothing | the RPC it asks, on first use | none |

At epoch 0 the first model is the pin check alone: hash the manifest and compare
`tx_io_pk@0`. Mixes exist: a client can follow the registry with a light client
to learn P_h, then verify one attested head.

## The trust anchor, per action

Every trust-sensitive action in the network's life answers one question
first, and each answers it against a different anchor, because each is taken
from a different position. Only five parties take the nine actions — the
deployer, the validators, the clients, governance, and the security council
that disaster recovery will one day need — and four of the nine are stations
in a single validator's lifecycle:

| Party | Action | Must answer | Anchor | Status |
| --- | --- | --- | --- | --- |
| Genesis deployer | assemble the founding artifacts | which founding artifacts are canonical, before any chain exists | its own verification at assemble: recomputed genesis hashes, DCAP-verified harvest quotes, registry storage recompiled from the policy document — all committed into `network_id` | shipped |
| Validator | release `root_key` — the responder | may this requester join the trust domain | the requester's verified quote, then `MeasurementRegistry.isAccepted` at fresh finalized state of the manifest-pinned chain | shipped |
|  | fetch `root_key` at every boot — the joiner | is this the network's key | the POSTed manifest: its custodian re-derives `tx_io_pk@0` from the delivered key and compares it with the pin `network_id` commits to; once that check exists the responder's quote carries no weight | the `network_id` binding in both halves of the handshake is shipped; the pin and the check are decided, not built — today the joiner appraises nothing |
|  | stake for a seat | does a validator seat imply TEE custody of its keys | at founding, the harvest quote binds both pubkeys to the measured guest; post-genesis, the deposit path registers keys with no hardware binding | open |
|  | receive a snapshot at a resync | is this state the canonical network's | `K_snap` is derivable only from `root_key`, so a snapshot that decrypts came from inside the trust domain | designed; the purpose is ungranted and no process serves it |
| Client | submit a TxSeismic | is this `tx_io_pk` this network's recipient key | epoch 0: the pin — hash the manifest against a pinned `network_id` and compare `tx_io_pk@0`, with no quote verification; later epochs: a record signed by the validator set, checked by a light client | epoch 0 decided, not built; later epochs open — today the SDKs trust whichever RPC they ask |
| Governance | change the accepted measurement set | is the change authorized | the manifest-pinned authority contract | a dev authority today; the mainnet authority is open |
|  | rotate `root_key` to fresh entropy | is the rotation authorized, and does the successor chain to the key it replaces | undecided — the candidates are the manifest-pinned authority contract and a consensus event, and a post-recovery rotation is the security council's, authorized by the recovery ceremony itself; a published chain of wraps links each version to its predecessor, which is continuity, not authenticity: two holders can each wrap a different successor, and both chains verify. Authenticity needs an anchor `network_id` commits to; the direction is a record signed by the validator set, with the epoch-0 pin as its base case | open, and prerequisite to any nonzero purpose-key epoch |
| Security Council | recover the network after a full-fleet loss | how does the network outlive losing every TEE at once | nothing — at least one node must stay live | open, pre-mainnet |

The validator's four actions repeat and interleave — `root_key` is RAM-only,
so a validator is the joiner again at every reboot. Party and action also
come apart at the edges: a deposit-path validator today takes its seat with
no joiner-style hardware check at all (the open row above), and whether a
read-only full node may join the trust domain without ever staking is part
of the same [open decision](#open-decisions).

The asymmetry between the responder and the joiner is structural, not an
implementation gap. A responder by definition holds `root_key` and a readable
chain — the genesis node included, from block 0 — so a genesis-pinned
contract is a sufficient live policy source from the network's first moment.
A joiner holds nothing yet: reading Seismic state at all is what `root_key`
buys. So the design gives the responder the live anchor and the joiner the
frozen one, and the joiner's protection is shaped accordingly — it holds no
secrets yet, so a dishonest responder can at worst deliver a wrong key, and
the check against the pin catches exactly that.

## Residuals

What the design accepts rather than closes. The risks are stated one by one,
then the rollback family collects the instances of a single mechanism: local
state a host can serve back to its guest as the guest's own past.

### Accepted risks

Each stated plainly, with the reason it is accepted.

**RAM-only `root_key`.** The network secret is never written to disk, so a
network that loses every node at once loses it permanently — there is no
on-disk and no on-chain copy. Accepted because every durable copy changes who
can become the network: a platform seal unlocks for whoever holds the
platform, and threshold shares make the share-holders a recovery quorum with
the power to reconstitute the secret. The operational rule is that at least
one node stays live through maintenance, outages, and fleet-wide changes
([why no on-disk backup](architecture.md#design-rationale)). Whether this
holds for mainnet is [the disaster-recovery decision](#open-decisions).

**Eclipse plus clock control.** The responder's freshness check measures a
finalized block's timestamp against the guest's wall clock, because that is
the one locally checkable witness of currency. A host that both eclipses the
guest and controls its clock can therefore have an honest enclave compute a
fresh-looking verdict on a stale allowlist. This is accepted host influence
under the TEE threat model; what would close it is
[the rollback family's exit](#the-rollback-family).

**A single responder grants membership.** A joiner accepts `root_key` from
whichever one responder answers yes; no corroboration across independent
responders is required. So the bar to defeat admission is compromising or
eclipsing one node that already holds `root_key`, not the
two-thirds-of-validators bar consensus sets — and unlike a block, a granted
`root_key` never reorgs away. Whether the joiner should require independent
corroboration is open design work.

### The rollback family

A host owns its guest's disk, its POSTed configuration, its network, its
clock, and its VM lifecycle, snapshots included. So any state the guest keeps
locally can be served back to it as its own past, and no local witness
detects the rewind. The instances:

- **LUKS is tamper-evident, not rollback-protected.** The header MAC and
  dm-integrity refuse an *edited* volume, but a snapshot of the whole volume
  is internally consistent, MAC and all, and restores cleanly. Everything
  under `/persistent` — reth's datadir, summit's database, certbot state —
  can be rewound together.
- **A chain view can be held at block 0.** That lands the responder's
  admission gate on the founding policy, where no timestamp check bites and
  "still at genesis" is indistinguishable from "chain withheld" from inside
  the guest. So not every responder honors the founding policy: only the
  custodian that minted `root_key` does, and only until the chain is seen past
  block 0, which rules out rewinding any joined node and rewinding the genesis
  node after block 1. What remains is the genesis node's own host keeping it
  at block 0 from birth: it never retires the founding policy. The genesis check bounds
  that to the founding accepted set — a reviewed list, never an image of the
  attacker's choosing — but a founding image deprecated for a vulnerability
  is exactly what it would revive. It is visible, since that validator never
  takes part in consensus, and a genesis-timestamp deadline would close it
  absent clock control; that belongs with the freshness-evidence decision.
- **TPM sealing was rejected partly on rollback grounds.** Sealed durability
  for founding keys would rest on vTPM clone and rollback semantics the
  platform defines ([the host
  platform](#the-host-platform-and-what-it-is-trusted-for)) — and a cloned consensus key is
  accidental equivocation ([key custody](network-founding.md#key-custody-ram-only-no-tpm-sealing)).

What no local witness can supply is freshness evidence the host cannot mint.
Every input a guest can check by itself — its disk, its clock, its chain view
— arrives through the host, so the ceiling of local defense is
tamper-evidence and bounded windows, and this family is where the design
accepts that ceiling. The exit is evidence from beyond the host: verifying
summit's finality signatures against the manifest-pinned validator set, so
"this block is final" becomes a claim only two-thirds of the validators can
fabricate, rather than whatever the local reth tags as finalized. That is
open design work.

## Open decisions

- **Disaster recovery** — must close before mainnet. The current default is
  permanent-brick risk on a full-fleet outage. The candidate shapes — a
  recovery-share quorum, hardware-sealed recovery, an external key custodian
  — each trade the RAM-only property for a new trusted party, which is why
  the decision is a trust-model change and not an implementation task.
  Whatever wins, a recovered network has a new `root_key`, so recovery is a
  key change at a bumped epoch and needs the anchor root-key rotation needs:
  client-visible, never a silent fork.
- **Validator key custody** — must close before staking opens to outside
  operators. It has two halves. The first is the post-genesis binding of
  validator keys to a TEE. Founding validators have the binding:
  [the harvest quote](network-founding.md#the-key-holder) proves both pubkeys
  were generated inside a measured guest. A deposit-path validator today
  registers keys with no hardware binding, so nothing stops its consensus
  keys from living, or signing, outside a TEE. The candidate fix is binding
  the node's consensus pubkeys into the admission transcript and recording
  the verified pair at admission — nearly free, since a quote is already
  verified at root-key release. This is the shape Microsoft's
  [Confidential Consortium Framework (CCF)](https://microsoft.github.io/CCF/)
  uses: a joining node's quote binds `report_data = SHA256(node pubkey)`,
  verified at admission and recorded in the ledger. The same decision covers
  the reverse case — whether a read-only full node may receive `root_key`
  without ever staking, and what pre-root identity staking should register.
  The second half is one live copy per key, and it applies to founding
  validators too. TEE custody does not keep a key to one signer: the LUKS key
  is network-shared ([the keys](architecture.md#the-keys)), so any admitted
  guest can open any node's volume. A host that copies a validator's disk to
  a second VM on an accepted image, and boots both, has two summits signing
  with one validator's keys; each runs reviewed code, and together they can
  vote twice at the same height. Nothing ties a volume to the VM that
  formatted it, and summit logs a detected equivocation but does not slash
  for it. A check at admission meets the
  storage cycle again: on a reboot the keys are on the volume `root_key`
  unlocks, so they cannot be presented before the fetch.
- **Registry mutation authority** — must close before mainnet. The manifest
  pins which contract may change the accepted measurement set, and today
  that role is filled by a dev authority. Who holds it on mainnet — a
  multisig, a governance contract, a council — decides who can admit code
  into the trust domain.
- **Root-key rotation** — must close before any purpose key ships a nonzero
  epoch. Nothing introduces fresh entropy after genesis, and the holder set
  only grows: a retired operator's TEE keeps `root_key` in RAM indefinitely.
  The candidate design is a chained rekey in
  [CCF](https://microsoft.github.io/CCF/)'s shape — mint a fresh `root_key`,
  publish the old key wrapped under a key derived from the new one, so
  current holders unwrap the chain for historical decryption while
  everything new derives from fresh entropy. The decision is the trigger set
  (suspected compromise; possibly validator exit) and the authorizing party
  per trigger: a live-network rotation fits the authority's reaction-time
  lanes — the registry-mutation question again, who may change a
  network-defining commitment, at which latency — while a post-recovery
  rotation belongs to the security council, authorized by the recovery
  ceremony itself. Whatever wins, the new key needs an anchor `network_id`
  commits to, since a pin that merely names `network_id` can be forged by a
  second mint. The direction is a record signed by the validator set, with
  the epoch-0 pin as its base case; a tx-io epoch bump needs a new pin for
  clients but none for joiners, who re-derive every epoch from `root_key`.

## Design rationale

Alternatives weighed and set aside, with the reasons that decided them. Each
names the section whose rule it settles. The full options pass — every
candidate anchor, the contests they competed in, and the candidates weighed
for the open decisions — is captured in
[the roots-of-trust decision record](decisions/2026-08-roots-of-trust.md), and
the key commitment's in
[the root-key commitment record](decisions/2026-09-root-key-commitment.md).

**The pin inside the manifest rather than in an addendum** ([the trust anchor,
per action](#the-trust-anchor-per-action)). A verifier trusts only what
`network_id` commits to, or what an authority it commits to signs, because a
thing that merely names `network_id` is not unique. The addendum, a
`tx_io_pk@0` pin attested after the genesis node's first boot, only named it:
a host that boots a second box on an accepted image and POSTs it the real
manifest gets a second genuine attestation of a different key. So `root_key`
is minted before the manifest, on every founding box, and assemble pins one
candidate. Clients gain a check with no quote verification, and the joiner's
check moves into the custodian, reading the pin from tmpfs.

**A key commitment rather than a network identity key** ([the trust anchor,
per action](#the-trust-anchor-per-action)). The joiner's appraisal of the
responder is a commitment check: re-derive `tx_io_pk@0` from the
delivered `root_key` and compare against the manifest's pin. The alternative
is the shape of [CCF](https://microsoft.github.io/CCF/),
whose clients authenticate the service by its identity key — here, a
dedicated network identity keypair, private half in the custodian, signing
handshake transcripts so joiners and clients verify a signature instead of
evidence. Set aside because its public half needs the same pin, and it is a
second network-wide impersonation-grade secret held by every custodian, so an
operator that exits keeps the power to sign; it brings its own generation,
custody, rotation, and recovery story, while the commitment already exists:
`tx_io_pk` is a binding,
deterministic function of `root_key`, published for TxSeismic clients anyway.
Reusing `tx_io` *as* the signing identity would be worse than either option:
one secp256k1 key doing both ECDH decryption and signing breaks the
per-purpose domain separation the key schedule enforces everywhere, and
`tx_io_sk` is the most-exposed network key — every node's reth holds it for
its process lifetime. If handshake ergonomics ever justify the identity key,
it arrives as a new custodian method, not a redesign.

**One list rather than a caveat per mechanism** (the whole doc). A residual
stated only where it bites is easy to accept twice and review never. The
sibling docs keep one sentence at the point of use and link here; this doc
holds the statement, the family it belongs to, and the reason it is accepted.
