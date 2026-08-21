# The TEE Trust Model <!-- omit in toc -->

**Status**: current as of 2026-08. Describes the shipped design plus the
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
  mainnet mutation authority must close before mainnet; the post-genesis
  binding of validator keys to a TEE must close before staking opens to
  outside operators; root-key rotation must close before any purpose key
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
| holds the canonical `root_key` | the `tx_io_pk@0` commitment in [the attested addendum](network-manifest.md#the-attested-addendum) — specified, not built |
| economically admitted | the summit genesis at founding, the deposit path afterwards ([founding](network-founding.md)) |
| view of chain state is current | [the freshness gate](chain-backed-admission.md#the-readiness-and-freshness-gate) around the responder's policy read |

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
|  | fetch `root_key` at every boot — the joiner | is this the canonical network, not a clone | the POSTed manifest: `network_id` bound in both halves of the handshake, and the delivered key checked against the pinned `tx_io_pk@0` commitment | the binding is shipped; the commitment check is specified, not built — today the joiner appraises no responder measurements |
|  | stake for a seat | does a validator seat imply TEE custody of its keys | at founding, the harvest quote binds both pubkeys to the measured guest; post-genesis, the deposit path registers keys with no hardware binding | open |
|  | receive a snapshot at a resync | is this state the canonical network's | `K_snap` is derivable only from `root_key`, so a snapshot that decrypts came from inside the trust domain | designed; the purpose is ungranted and no process serves it |
| Client | submit a TxSeismic | is this `tx_io_pk` this network's recipient key | a quote over `tx_io_binding(network_id, tx_io_pk, epoch)` | evidence endpoint shipped; the SDKs verify no quote yet |
| Governance | change the accepted measurement set | is the change authorized | the manifest-pinned authority contract | a dev authority today; the mainnet authority is open |
|  | rotate `root_key` to fresh entropy | is the rotation authorized, and does the successor chain to the key it replaces | undecided — the candidates are the manifest-pinned authority contract and a consensus event, and a post-recovery rotation is the security council's, authorized by the recovery ceremony itself; in every case the published wrap-chain links each version to its predecessor, so holders verify continuity | open, and prerequisite to any nonzero purpose-key epoch |
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
the commitment check catches exactly that.

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
- **A chain view can be rewound to block 0.** That lands the responder's
  admission gate in its genesis window, where no timestamp check bites and
  "still at genesis" is indistinguishable from "chain withheld" from inside
  the guest. The genesis check bounds what the rewind buys to the founding
  accepted set — a reviewed list, never an image of the attacker's choosing —
  but a founding image deprecated for a vulnerability is exactly what it
  would revive. Nor does disaster recovery clear it: `network_id` and the
  genesis block survive recovery, so block-0 policy keeps listing what it
  listed. Hardening the gate further is open work.
- **An in-process latch is the strongest local defense available.** The gate
  latches the genesis window shut the first time it sees the chain past
  block 0 — in process memory, so a restart reopens it. That is not a
  shortcut to fix later: persisting the latch would store it on a disk the
  same host owns, so it would rewind with everything else. A hardware counter
  is not the alternative either — no property here rests on platform-specific
  durable state ([the host
  platform](#the-host-platform-and-what-it-is-trusted-for)).
- **TPM sealing was rejected partly on the same grounds.** Sealed durability
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
  Whatever wins, recovery rotates the key commitment so a recovered network
  is a client-visible event, never a silent fork
  ([the addendum's recovery rule](network-manifest.md#the-attested-addendum)).
- **Post-genesis binding of validator keys to a TEE** — must close before
  staking opens to outside operators. Founding validators have the binding:
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
- **Registry mutation authority** — must close before mainnet. The manifest
  pins which contract may change the accepted measurement set, and today
  that role is filled by a dev authority. Who holds it on mainnet — a
  multisig, a governance contract, a council — decides who can admit code
  into the trust domain, and the same decision gates the fork-schedule
  amendment path ([what the genesis hash does not cover](network-manifest.md#validation-gates)).
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
  ceremony itself. Whatever wins, a rotation republishes the key
  commitment at a bumped epoch under the same `network_id`, so it is
  client-visible, never silent — the same rule recovery follows
  ([the addendum's recovery rule](network-manifest.md#the-attested-addendum)).

## Design rationale

Alternatives weighed and set aside, with the reasons that decided them. Each
names the section whose rule it settles. The full options pass — every
candidate anchor, the contests they competed in, and the candidates weighed
for the open decisions — is captured in
[the roots-of-trust decision record](decisions/2026-08-roots-of-trust.md).

**A key commitment rather than a network identity key** ([the trust anchor,
per action](#the-trust-anchor-per-action)). The joiner's planned appraisal of
the responder is a commitment check: re-derive `tx_io_pk@0` from the
delivered `root_key` and compare against the addendum's pin. The alternative
is the shape of [CCF](https://microsoft.github.io/CCF/),
whose clients authenticate the service by its identity key — here, a
dedicated network identity keypair, private half in the custodian, signing
handshake transcripts so joiners and clients verify a signature instead of
evidence. Set aside because it is a second network-wide
impersonation-grade secret, with its own generation, custody, rotation, and
recovery story, while the commitment already exists: `tx_io_pk` is a binding,
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
