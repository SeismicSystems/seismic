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
│   │                               seismic-images (`init --image-json`): which
│   │                               image, built from what, bytes where
│   ├── reth-genesis.json           authored: policy-free EL genesis
│   ├── summit-genesis.toml         authored: consensus parameter choices
│   ├── measurements.json           authored: the image's measurements as
│   │                               policy records, one per platform admitted
│   ├── founder-withdrawal-credentials.json
│   │                               authored: one address per founding
│   │                               node, in node-name order
│   └── harvest/                    written by `node harvest`
│       └── <node>.json             the founding archive: pubkeys + quote,
│                                   the DCAP collateral that verification
│                                   used, the instant it judged at, and the
│                                   trust anchors it judged with — so the
│                                   quote stays verifiable once Intel's live
│                                   collateral ages past it
├── nodes/                        the records `configure` writes as it runs
│   │                               (gitignored: per-deploy output, not identity)
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
ed25519 keys. `seismic-tee node configure` therefore splices each
box's current descriptor IP into the copy it delivers, touching no other
field, and then asserts the launch against the pins (reth block 0,
summit keys). The committed file itself never changes after assemble.

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

Throwaway foundings live outside every checkout:
[deploy's `tee/devnet.sh`](https://github.com/SeismicSystems/deploy/blob/main/tee/devnet.sh)
founds one under `~/seismic-networks/<name>` and deletes it on `down`
(the step-by-step recipe is
[the devnet runbook](https://github.com/SeismicSystems/deploy/blob/main/tee/runbook-devnet.md)).
One founded in this checkout goes in a `tmp-*` directory, gitignored
wholesale. The directory's name is the network's: `assemble` writes it
into the manifest as `name`, and `init` into the summit genesis as
`namespace`, so moving a directory in under another name commits a
manifest that names something else, and `assemble --check` refuses it.
A network that may be committed is founded under its final name —
`tee/devnet.sh up --name <name>`.

## fixture-devnet

[fixture-devnet/](fixture-devnet/) is the one committed directory: a real
founding — a TDX cohort's archive, each box's founding quote with
the collateral and trust anchors it was verified against, and the
artifact set assembled from it — committed whole as `devnet.sh` left it
(minus the gitignored `nodes/`). It is two things at once.

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
drift` adds the gates that need `seismic-reth` and `summit`.

**Not a network anyone runs.** The cohort was torn down the day it was
founded, and nothing can be brought up from its identity: the keys the
archive vouches for existed only in those boxes' RAM.

| | |
|---|---|
| Founded | 2026-10-30, with [deploy's `tee/devnet.sh`](https://github.com/SeismicSystems/deploy/blob/main/tee/devnet.sh) `up --name fixture-devnet --nodes 2` |
| Image | [`seismic_2026-10-30.3534db`](https://github.com/SeismicSystems/seismic-images/releases/tag/seismic_2026-10-30.3534db) (`inputs/image.json`): summit `69ca74d`, enclave `e87485e`, seismic-reth `a79dd29` |
| `measurement_id` | `seismic_2026-10-30.3534db.vhd` (azure-tdx) and `seismic_2026-10-30.3534db.tar.gz` (gcp-tdx) (`inputs/measurements.json`) |
| Records | 2, `inputs/harvest/samlaf-fixture-devnet-tee-devnet-{1,2}.json` (named after the nodes, operator prefix included; the stem is only a label): `-1` on azure-tdx, `-2` on gcp-tdx, record `version` 1 |
| `founding_tx_io_pk` | `samlaf-fixture-devnet-tee-devnet-1`'s candidate, the first record by name |
| Verifier at founding | dcap-qvl 0.5.2 (`trust_anchors.dcap_qvl_version`) |
| `network_id` | `0x31f2ba7ac91a4a9a560c2a4bcd9cee348a76d980537448d85924a1507cc989fc` |

The cohort was founded under the directory's own name, so the
manifest's `name` and `namespace` are `fixture-devnet`, and a refresh is
founded under that name too.

Replay it by hand:

```bash
seismic-tee verify-founding tee/networks/fixture-devnet
```

### Refreshing fixture-devnet

Aging never breaks the fixture: `verify-founding` holds every freshness
check to each record's own `verified_at`. A change to the record, policy
or manifest schema, or to the verifier, does — the drift signal this
exists for. Refresh it then, in the PR that bumps the enclave pin: the
old CLI can't read the new fixture, nor the new CLI the old one, so
neither passes CI alone. Keep the refresh its own commit and merge the
PR as two commits (rebase, not squash), so history tells the fixture
apart from the code.

From this checkout, with [deploy](https://github.com/SeismicSystems/deploy)
checked out beside it:

```shell
mise run seismic-tee::install-local   # devnet.sh runs the seismic-tee on PATH
../deploy/tee/devnet.sh up --name fixture-devnet --nodes 2
rm -rf tee/networks/fixture-devnet
cp -r ~/seismic-networks/fixture-devnet tee/networks/   # nodes/ stays gitignored
../deploy/tee/devnet.sh down --name fixture-devnet      # also deletes ~/seismic-networks/fixture-devnet
sha256sum tee/networks/fixture-devnet/network-manifest.json   # the new network_id
```

Then put the new `network_id` in `tee/cli/common/src/manifest.rs`
(`the_id_is_the_hash_of_the_file_bytes`) and the table above, update the
table's other rows from `inputs/image.json`, `inputs/measurements.json`
and `inputs/harvest/`, and run `make -C tee/cli check`.

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
