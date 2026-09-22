# Network directories

One directory per network: the committable identity of a founded
network. The authored inputs live under `inputs/`, joined mid-founding by
the harvested facts (`inputs/harvest/` — the cohort's TEE-born founding
keys and their quotes); `seismic-tee network assemble` derives
the artifact set from them at the top level. Everything top-level is
hash-pinned by `network-manifest.json` — whose SHA-256 is the network's
`network_id` — and everything under `inputs/` is provenance. What the
manifest's fields mean, what `network_id` transitively commits to, and why it
hashes the exact file bytes:
[the network manifest doc](https://github.com/SeismicSystems/seismic/blob/main/docs/tee/network-manifest.md).

A committed directory is auditable as a whole, by anyone, offline:
`seismic-tee verify-founding <dir>` re-verifies its founding — every
archived quote against the bundle archived with it and the policy the
manifest pins, and the archive against the validator set the summit
genesis seats.

```text
tee/networks/<name>/
├── inputs/                       provenance (authored + harvested)
│   ├── image.json                  the image's release record, verbatim from
│   │                               seismic-images (`init --image`): which
│   │                               image, built from what, bytes where
│   ├── reth-genesis.json           authored: policy-free EL genesis
│   ├── summit-genesis.toml         authored: consensus parameter choices
│   ├── measurements.json           authored: raw PCRs from `make measure`,
│   │                               carrying its measurement_id
│   ├── founder-withdrawal-credentials.json
│   │                               authored: one address per founding
│   │                               node, in node-name order
│   └── harvest/                    written by `network harvest`
│       └── <node>.json             the founding archive: pubkeys + quote,
│                                   the DCAP collateral that verification
│                                   used, the instant it judged at, and the
│                                   trust anchors it judged with — so the
│                                   quote stays verifiable once Intel's live
│                                   collateral ages past it
├── nodes/                        the records `configure` writes as it runs
│   │                               (gitignored: per-deploy output, not identity)
│   ├── bootnodes.json              founding enode set from `configure`
│   └── <node>.init-config.toml     the config `configure` POSTed to that
│                                   node, byte-exact
│
│                                 artifact set: derived by `assemble`, every
│                                 file below hash-pinned by the manifest
├── network-manifest.json           SHA-256 of these bytes = network_id
├── reth-genesis.json               input + compiled registry storage
├── summit-genesis.toml             input + eth_genesis_hash + validator set
└── measurement-policy-bootstrap.json
                                    allowlist promoted from measurements
```

The committed `summit-genesis.toml` is a founding-era snapshot: its
validator entries carry the IPs the cohort had at assemble time, which
are network topology, not identity — summit's config digest (the
manifest's pin) excludes them, and peers authenticate by the pinned
ed25519 keys. `seismic-tee network configure` therefore splices each
box's current descriptor IP into the copy it delivers, touching no other
field, and then asserts the launch against the pins (reth block 0,
holder keys). The committed file itself never changes after assemble.

![How assemble derives the artifact set, what pins what, and how each
configure run delivers and asserts it](network-dir.svg)

The same founding from each node's side — why keys are born before the
manifest, and how the boot chain is sequenced to allow it — is
[the network founding doc](https://github.com/SeismicSystems/seismic/blob/main/docs/tee/network-founding.md).

Directories are committed because the directory is everything needed to
(re)configure, join, or debug that network later, and its manifest is the
network's immutable identity — a founded network's `network_id` must
never drift. The cohort's addresses (live IPs) are not part of that
identity, so they don't live in the directory at all: `seismic-tee ctx
set-nodes <network>` imports them straight from the Pulumi stack's
`nodes` output into the context file (see the tee README's "The context
file"). `nodes/` keeps only the records `configure` writes as it runs,
which is why it stays gitignored.

Throwaway foundings go in a `tmp-*` directory instead — those are
gitignored wholesale, so a scratch cohort can be founded, torn down, and
`rm -rf`'d without touching git (the copy-pasteable recipe is
[the devnet runbook](https://github.com/SeismicSystems/deploy/blob/main/tee/docs/runbook-devnet.md)). If a throwaway
turns out to matter, renaming the directory is enough to commit it —
`network_id` is minted from the manifest bytes, not the path — but the
manifest keeps the `tmp-*` name it was assembled under (the name is part
of those bytes), so a network you already suspect will matter deserves a
real directory name from the start.

## fixture-devnet

[fixture-devnet/](fixture-devnet/) is the one committed directory: a real
founding — a four-node Azure TDX cohort's archive, each box's founding
quote with the collateral and trust anchors it was verified against, and
the artifact set assembled from it — committed whole (minus the
gitignored `nodes/`). It is two things at once.

The documented shape: every file the tree above names, as `init`,
`harvest` and `assemble` actually wrote them. The unit tests embed a few
of them (the manifest, its policy, the reth genesis) as the real thing
to parse and pin against.

The replay fixture: `make -C tee/cli test` runs `verify-founding` over
every committed directory that carries an archive
(`tee/cli/network/tests/replay.rs`), so a change to the verifier, the
record schema, or the policy or manifest schema in the pinned enclave
crates goes red on the PR that bumps the pin, against quotes a real TDX
cohort produced, rather than at the next founding. `make -C tee/cli
drift` adds the gates that need `seismic-reth` and the sibling repos'
current state.

**Not a network anyone runs.** The cohort was torn down the day it was
founded, and nothing can be brought up from its identity: the keys the
archive vouches for existed only in those boxes' RAM. Its
[README](fixture-devnet/README.md) records the image it booted and how
to refresh it from a fresh founding.

To found any network, throwaway or real, don't reuse or copy this
directory: run `init <new-dir>`, author fresh inputs, and follow
the founding workflow in the tee README. Start both genesis files from
the image's seismic-images release: it carries `reth-genesis.json` and
`summit-genesis-starter.toml` as the image's own code has them, the
starter's parameter set checked against the image's `summit` when the
image is built.
`namespace` (the BLS signature domain separator) and `chainId` must be
unique per network that matters (cohorts sharing them can cross-replay
signatures).
