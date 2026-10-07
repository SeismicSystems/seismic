# Network Founding <!-- omit in toc -->

**Status**: shipped. The one config POST per box and the harvest → assemble →
configure flow are how the four-node devnet was founded. The candidate
`root_key` and its pin are built in the enclave and the image; seismic-tee's
side, harvesting the candidate and pinning it in assemble, is
[SEI-643](https://linear.app/seismic-systems/issue/SEI-643)'s remaining work.

- [Summary](#summary)
- [The founding flow](#the-founding-flow)
- [Why founding-time quote verification is load-bearing](#why-founding-time-quote-verification-is-load-bearing)
- [Summit's keys before LUKS](#summits-keys-before-luks)
- [Key custody: guest RAM until LUKS, no TPM sealing](#key-custody-guest-ram-until-luks-no-tpm-sealing)
- [Founding-window security](#founding-window-security)
- [Design rationale](#design-rationale)

## Summary

How a Seismic network is founded: where validator keys are born, how they get
into the manifest, and how the node boot chain is sequenced to allow it. What
the manifest's fields commit to, and the byte-exact rules, belong to
[the network manifest](network-manifest.md); the harvest's wire format and
quote binding belong to the attestation service's code in the
[enclave](https://github.com/SeismicSystems/enclave/tree/seismic/bin/attestation-service)
repo.

A network is named by one hash, `network_id = SHA-256(network-manifest.json)`,
and the manifest commits to the network's first value: the reth genesis, the
summit genesis, and the bootstrap measurement policy. Everything after founding
is a later value, reached through finalized blocks
([one identity, a succession of values](trust-model.md#one-identity-a-succession-of-values)).
The manifest commits to every founding validator's public keys, which the
summit genesis carries, and to the network's `root_key` through `tx_io_pk@0`,
so all of them must exist before the manifest does
([design rationale](#design-rationale)).

Boxes boot the measured image **identity-free**. A boot-time oneshot,
**`summit-keygen`**, generates summit keypairs into guest RAM, the custodian
mints a **candidate `root_key`**, and the attestation service, the only process
that can mint a quote, proves the public halves of all three with one. The
founder harvests and DCAP-verifies those quotes, and assemble pins the complete
validator set and one box's `tx_io_pk@0` before minting `network_id`. Per
validator, the pin covers both pubkeys and the withdrawal address, never the IP
([what summit's digest covers](network-manifest.md#what-summitgenesis_config_digest-covers)).
Configure then delivers everything a node needs in one POST.

Founding is a fragile, supervised operation. It happens rarely, while joining
and verifying happen forever, and those check exactly one hash.

## The founding flow

[The node lifecycle](architecture.md#node-lifecycle-power-on-to-serving)
draws these steps against what runs inside each box, from power-on to
serving.

Each step is one command, run by the founder:

1. **Provision.** The cohort's Pulumi stack in the
   [deploy](https://github.com/SeismicSystems/deploy) repo (`pulumi up`) boots
   every box on the measured image. No box holds any network identity yet.
2. **Harvest** (`seismic-tee node harvest`). For each box, fetch its summit
   pubkeys, its candidate's `tx_io_pk@0`, and a TDX quote over them and a fresh
   per-box nonce from the attestation service's harvest port, DCAP-verify the
   quote against the network's intended measurements, and archive the result
   under the network directory's `inputs/harvest/`. The archive holds the DCAP
   collateral the verification used, so the quote stays verifiable after
   Intel's live collateral moves on.
3. **Assemble** (`seismic-tee network assemble`). Replay every archived quote
   offline against the policy compiled from the authored measurements. Have
   summit emit the validator list into the summit genesis, pin summit's
   `config_digest` of it, pin the `tx_io_pk@0` of the first box by name as
   `founding_tx_io_pk`, and write the manifest. `network_id` is the SHA-256
   of the manifest bytes. The validators' IPs come from the cohort's node
   descriptors, not from the harvest.
4. **Configure** (`seismic-tee node configure --genesis-node <name>`, which
   must name the pinned box). POST each box its configuration: the manifest,
   the reth genesis, and the summit genesis with each box's current IP spliced
   in, which leaves `network_id` unchanged
   ([what summit's digest covers](network-manifest.md#what-summitgenesis_config_digest-covers)).
   The pinned box goes first and alone: it is the only box that will hold
   `root_key`, and its reth enode is the joiners' bootnode. If it fails, the
   run stops and the founding is redone
   ([founding-window security](#founding-window-security)). The joiners follow.
   Each node is deploy-verified as soon as it is ready.
5. **Launch checks**, at the end of configure and again on demand with
   `seismic-tee node configure --check`. Every box must serve exactly the
   pubkeys harvested from it, and every reth must serve the manifest's
   `eth.genesis_hash` as block 0.

An auditor re-asks the founding's question later, offline, with
`seismic-tee verify-founding <dir>`: every archived quote against its archived
collateral and the pinned policy, and every archived key against the
validator set the summit genesis seats.

A joiner needs none of this. It checks `sha256(manifest) == network_id`, checks
each artifact against the manifest — which covers the summit genesis's full
consensus content, parameters plus every validator's keys and withdrawal
credentials — and then provisions a box and runs
`seismic-tee node configure --bootnode`. Runtime admission and the deposit
contract do the rest.

**Who keeps `root_key`.** Every box mints a candidate at boot, and the
manifest's pin picks one. No flag in the POST says who mints: each custodian
reads the pin from the manifest it was POSTed, the pinned box keeps its
candidate, and every other box discards its own and fetches the pinned key
([the node lifecycle](architecture.md#node-lifecycle-power-on-to-serving)).

## Why founding-time quote verification is load-bearing

Two independent gates protect two different things:

- **root_key admission** (attestation-service verifies evidence
  enclave-to-enclave before the custodian wraps `root_key`) gates the
  *privacy* trust: decryption keys, LUKS.
- **Consensus membership** is gated by whose pubkeys are in the validator
  set. Summit never talks to the custodian — its keys are per-validator, not
  network-shared — so an unverified founding pubkey could vote from outside a
  TEE. Post-genesis validators get TEE-ness enforced on the way in by
  deposit/registry admission; founding keys bypass that path by construction.
  Founding-time DCAP verification is the founding analogue of contract
  admission: the same check, done once, by the tool that pins the set.

The harvest quote is also a stronger statement than a BLS proof-of-possession:
measured code generated the keypair, kept the private halves where only the
summit user can read them, and quoted the public halves, so possession, TEE
custody, and honest generation (no rogue-key choice) all follow from the
measurement. Post-genesis joiners still provide
signature-based possession proofs through the deposit path.

## Summit's keys before LUKS

Summit's keys, an ed25519 node identity and a BLS12-381 consensus key, must
exist before the manifest, which pins them. The keystore summit reads them
from lives on the LUKS volume, which opens only after the config POST. Two
oneshots carry the keys across that gap through tmpfs, both built on summit's
own binary, and the attestation service serves their public halves. No
process holds them in between ([design rationale](#design-rationale)).
[The node lifecycle](architecture.md#node-lifecycle-power-on-to-serving)
shows where each step falls.

The three summit units form one group
([unit groups](architecture.md#unit-groups)). Every member is
`PartOf=summit.target`, so restarting the target re-runs the setup units,
while a restart of `summit.service` alone does not.

| Unit | Kind | Runs |
| --- | --- | --- |
| `summit-keygen.service` | setup, oneshot | at boot, in parallel with tdx-init's wait |
| `summit-persist.service` | setup, oneshot | once per boot, after LUKS setup and `summit-keygen` |
| `summit.service` | the service | after `summit-persist`, which it requires |

- **`summit-keygen` writes the keys at boot.** It runs
  `summit keys generate --key-store-path /run/seismic/summit/keys --no-overwrite` as
  the summit user, and depends on nothing the POST produces. Summit writes
  both keypairs in its own keystore format (directory 0700, files 0600, owned
  by `summit`). A step after it writes their public halves, from
  `summit keys show --json`, to the public-keys file
  `/run/seismic/summit/public-keys.json`, which only summit's setup units
  write and the attestation service reads. `show` fails on a half-written
  set, which fails the unit.
- **The key files are their own write-once marker.** With `--no-overwrite`,
  summit exits without writing when either key file exists, so every later
  run in the same boot is a no-op: a restart of `summit.target`, which re-runs
  the setup units, cannot replace the keys the manifest pins. That holds
  because nothing deletes the tmpfs keys before the next reboot empties tmpfs.
- **The attestation service serves the harvest on `:7879`.** It starts at boot,
  and until the POST arrives this is all it serves. The port is plain HTTP:
  nginx and TLS certificates exist only after the POST, and deploy tooling
  already polls raw ports during first boot. `GET /v1/keys` returns both
  pubkeys from the public-keys file. `GET /v1/quote?nonce=…` adds the
  custodian's candidate `tx_io_pk@0`, from the candidate file the custodian
  writes when it mints, and a quote whose `report_data` is a domain-separated
  binding over the deploy-supplied nonce, both pubkeys and that key, which the
  attestation service builds itself, as it builds every binding it quotes ([one
  process opens the TPM](architecture.md#one-process-opens-the-tpm)). The nonce
  prevents replay of quotes from earlier harvests. The binding cannot include
  `network_id`, which does not exist yet; the pin itself provides the intent
  binding, and [founding-window security](#founding-window-security) says what
  that costs.
- **Quotes stop once the manifest file appears**, answering `410 Gone`. This is
  per boot, not permanent: the config lives on tmpfs and is re-POSTed each
  boot, so a rebooted node briefly serves quotes over that boot's fresh keys,
  which `summit-persist` then passes over for the keystore. That is harmless,
  since nothing ever signs with them, but it is why the harvest port's network
  restriction is permanent rather than founding-only. `/v1/keys` keeps
  serving for life: `summit-persist` rewrites the public-keys file from the
  keystore, which makes it the source for the launch-time continuity check.
- **`summit-persist` copies the keys into the keystore** once LUKS opens. It
  is a script in the image, and it decides from the keystore on disk, not
  from tmpfs:

  | Keystore | Keys in tmpfs | `summit-persist` |
  | --- | --- | --- |
  | complete | any | `summit keys show` on the keystore must succeed; the tmpfs keys are a reboot's throwaway set |
  | absent | present | first boot: copies each file into `/persistent/summit/keys` |
  | absent | absent | fails, so summit never starts on keys the manifest did not pin |
  | one of its two files | — | finishes the copy only if that file is byte-identical to its tmpfs counterpart, an interrupted first copy; anything else is refused |

  Each file lands atomically, copied under a temporary name and renamed, and
  every successful run rewrites the public-keys file from the keystore. It is
  its own unit rather than a pre-start step of `summit.service`, so it runs
  once per boot rather than at every summit restart, and so only it can write
  the keystore.
- **Summit only reads its keystore.** `summit.service` gets
  `/persistent/summit/keys` read-only and `/run/seismic/summit/keys` inaccessible.
  It has no keygen step of its own, and must never gain one as a fallback:
  one would silently mint fresh, unpinned keys, deferring the failure from a
  loud startup error to a launch-check mismatch.

Summit's [`keys`](https://github.com/SeismicSystems/summit/blob/main/node/src/keys.rs)
subcommands write and read the keystore, so its format — hex-encoded keys in
`node_key.pem` and `consensus_key.pem` — is summit's by construction, and no
other repo carries summit-key code. The image depends on that CLI instead:
`generate --no-overwrite` leaving existing keys alone, the two file names,
and `show`'s output, which the public-keys step reads through
`summit keys show --json`.

The summit genesis rides the config POST. tdx-init's `summit_genesis_base64`
field is written to `/run/seismic/conf/summit-genesis.toml`, and summit reads
it from there through `--genesis-path`: the same re-POSTed-per-boot lifecycle
as the reth genesis. Summit's
[`acquire_genesis`](https://github.com/SeismicSystems/summit/blob/main/node/src/args.rs)
returns immediately when a valid file is present, so summit's pre-genesis
`sendGenesis` RPC is never reached on a TEE node; deleting it is
[SEI-139](https://linear.app/seismic-systems/issue/SEI-139).

## Key custody: guest RAM until LUKS, no TPM sealing

Founding keys live in tmpfs from boot, and their copy there stays until the
next reboot, also after LUKS opens. tmpfs is guest RAM, which TDX encrypts
against the host just as it does process memory. Inside the guest, the summit
user and root can read the files: the same parties that can read the keystore
once LUKS opens, or the memory of a process holding the keys. The image has no
swap, so neither tmpfs nor process memory ever leaves guest RAM.

No process holds the keys, so a crash cannot lose them. A reboot or box loss
in the harvest → LUKS-open window can, since tmpfs starts empty, and that
destroys a pinned key and forces a **re-found**: destroy the stacks and start
over (`pulumi destroy` and a fresh `pulumi up`). Destroying the stacks deletes
the data disks, which is the LUKS wipe; fresh boxes mean fresh IPs and a fresh
harvest, so nothing stale can leak into the new identity. Nothing of value
exists pre-genesis, and founding is a rare, short, supervised internal act.
The keys are never sealed to the TPM ([design rationale](#design-rationale)).

The guard is the **launch-time pubkey-continuity assertion**. A rebooted box
generates fresh keys and passes admission fine, so without the check the
network would launch with a silent dead founding slot. Configure retries a
mismatch until its deadline, because `/v1/keys` serves this boot's fresh keys
until `summit-persist` rewrites the public-keys file from the keystore; a
mismatch that persists is the dead-slot case,
and the fix is a re-found, never launching around it. The response could be
graded — BFT tolerates f dead of 3f+1, and the deposit path can eventually
replace a slot — so a devnet may accept a degraded launch where mainnet
re-founds.

## Founding-window security

The identity-free window is an attack surface, not just an availability risk:
tdx-init accepts the *first* config POST, and a waiting box holds pinnable
key material. An attacker who POSTs first enrolls the box into *their*
network — their manifest, their measurement policy, their responders — and
can deliver a `root_key` they know, after which `summit-persist` would write
the harvested keys onto a LUKS volume the attacker can read. On the pinned box
the stake is higher: its candidate is the network's future `root_key`, and an
attacker's manifest that pins it hands the attacker's network that key. The
failure is bounded — tdx-init is one-shot, so the real configure then fails
loudly and a re-found discards those pubkeys and that candidate before
anything launches — but only if the process treats it that way, which is why
configure stops when the pinned box fails. Guards:

- **Network-level**: the cloud firewall restricts the config port (`:8080`)
  and the harvest port (`:7879`) to the operator's source CIDR, permanently,
  since both come back on every boot
  ([what the outside can reach](architecture.md#what-the-outside-can-reach)).
- **Burned-key rule**: a harvested key is trustworthy only if the same box
  later accepts the real configure cleanly. Any anomaly — a quote window
  already closed, a failed verification, a POST rejected, an unexpected
  reboot or custodian restart — burns the whole harvest: re-found, never
  retry-around. Harvest enforces its half by aborting and writing nothing.
- **Window length**: the rootfs is measured at boot but not (yet)
  integrity-protected at runtime, so a harvest quote attests boot-time state
  only. Window length is a security parameter: keep founding short and
  supervised. It has a floor: first-boot disk provisioning (encryption plus
  integrity setup) can run an hour-plus on multi-TB disks, and a re-found
  repeats it.

## Design rationale

Alternatives weighed and set aside, with the reasons that decided them. Each
names the section whose rule it settles.

**Every box mints a candidate `root_key`, rather than one designated minter**
([the founding flow](#the-founding-flow)). The key must exist before the
manifest for `network_id` to commit to it, and before the POST no box knows its
role; the reasoning is in [the architecture's design
rationale](architecture.md#design-rationale) and [the root-key commitment
decision record](decisions/2026-09-root-key-commitment.md).

**Keys born before the manifest, rather than a ceremony after it**
([summary](#summary)). Every future joiner needs the founding summit genesis
verbatim, validator set included, and a genesis whose keys post-date the
manifest can never be hash-pinned by `network_id`. Keeping late-born keys
leaves a second, unpinned trust stratum that a quote sidecar can prove
membership of but never completeness, plus the machinery that injects the
late facts. Generating the keys outside a TEE would delete the keygen step, but
consensus signatures are verified offline, so a key that ever existed outside
a TEE lets its holder forge signed histories for every future verifier. The
pass that settled this, with the boot chain it replaced and every alternative
weighed, is [the founding-reorder decision
record](decisions/2026-07-founding-reorder.md).

**Setup units and the attestation service, rather than a key-holder daemon**
([summit's keys before LUKS](#summits-keys-before-luks)). The keys need a
home from boot until LUKS opens, and their public halves need a quote. The
shapes set aside:

- **A daemon that holds the keys and quotes them itself.** It must run as the
  summit user to write summit's keystore, so the TPM group would land on that
  user, and `summit.service` runs as it too: the consensus daemon, network-facing
  for the node's whole life, could quote any `report_data`. That includes a
  root-key request for an ephemeral key of its own, which any responder would
  answer with `root_key`. Quoting belongs to the one process that opens the
  TPM ([one process opens the TPM](architecture.md#one-process-opens-the-tpm)).
- **A daemon that holds the keys and serves them on a local socket**, with the
  quote left to the attestation service. Its only job would be carrying state
  across the gap, which costs a process for the node's life, an IPC protocol
  the attestation service can speak without linking commonware, a per-method
  peer check on its socket, and a founding burned by any crash of it. A tmpfs
  file carries the same state with none of that: it is as confidential to the
  host as process memory, and readable in the guest by the same parties.
- **Summit generating its own keys.** Summit already loads a keystore and
  waits for its genesis, but it would also have to know the founding window,
  serve public keys before LUKS opens, and order itself against the disk.
  That is orchestration the image's units express, moved into the
  application ([unit groups](architecture.md#unit-groups)). Running summit's
  *binary* in a setup unit is a different thing: the daemon still knows
  nothing of the founding window or the disk, and the setup units only borrow
  two of its CLI commands.
- **An existing process generating them.** tdx-init is the unauthenticated
  first-POST listener, the most exposed process before the POST. The
  attestation service is the most exposed after it, and holds no key material.
  The custodian would invert its own design: it is deliberately the most
  isolated process on the box, with no network listener, no async runtime and
  no BLS dependency, and it mediates one secret it never releases, while
  summit's keys are handed to summit.

**RAM-only rather than TPM-sealed founding keys** ([key
custody](#key-custody-guest-ram-until-luks-no-tpm-sealing)). Sealing would
survive a reboot in the window, but it adds a second sealing policy that must
stay in lockstep with the measurement policy, unknown vTPM clone and rollback
semantics (a duplicated consensus key is accidental equivocation), and
sealed-blob migration and scrubbing machinery. Sealing is not what SGX
networks use to escape this either: the host stores the sealed blob and can
serve an old one back. Revisit only if RAM-only proves unacceptable for
mainnet founding.
