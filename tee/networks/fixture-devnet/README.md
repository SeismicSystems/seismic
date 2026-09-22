# fixture-devnet

The one committed network directory: one real four-node Azure TDX
cohort's founding, committed whole as `assemble` left it. It documents
the directory shape ([../README.md](../README.md)), a few of its files are
embedded by the unit tests, and the hermetic suite replays its archive
on every PR (`tee/cli/network/tests/replay.rs`, under `make -C tee/cli
test`). Not a network anyone runs — the cohort was torn down the day it
was founded — but the quotes its boxes produced are real, and
re-verifying them against the collateral archived beside them is what
catches a verifier, record-schema, policy-schema or manifest-schema
change in the pinned enclave crates before the next real founding does.

| | |
|---|---|
| Founded | 2026-09-15, by hand, from [the devnet runbook](https://github.com/SeismicSystems/deploy/blob/95127211106cee2d6855018a703fe6b44db4314f/tee/docs/runbook-devnet.md) |
| Image | `seismic-dev_2026-08-27.5c012e` (a `seismic-dev_*` build: no seismic-images release, so no release tag to pin) |
| `measurement_id` | `seismic-dev_2026-08-27.5c012e.vhd` (`inputs/measurements.json`) |
| Records | 4, `inputs/harvest/tmp-devnet-1-{1..4}.json`, record `version` 1 |
| Verifier at founding | dcap-qvl 0.5.2 (`trust_anchors.dcap_qvl_version`) |
| `network_id` | `0x6dc6adff3fe0aa9278dfbb3a1236af1ff855e32f6ceb754b8be8390879e06595` |

The manifest's `name` and `namespace` are `tmp-devnet-1`, the directory
the cohort was founded under: both are part of the manifest bytes, so
they stay as assembled ([../README.md](../README.md) on renaming a
throwaway). The cohort's descriptor (`nodes/`) is not here — a dead
cohort's IPs, gitignored like every network's.

Replay it by hand:

```bash
seismic-tee verify-founding tee/networks/fixture-devnet
```

## Refreshing it

The archive is what keeps the fixture verifiable: `verify-founding`
holds every freshness check to each record's own `verified_at`, so
Intel's TCB info aging past its `nextUpdate` never breaks it. What does
break it is a change to the record schema, the policy or manifest
schema, or a verifier that judges these quotes differently — the drift
signal this exists for. Then, or when a cohort on a released image is
to hand, refresh it as a PR of its own that says which image the new
cohort pinned:

1. Take a clean `Found a devnet` run's `founding-<stack>` artifact
   (`.github/workflows/found-devnet.yml` uploads the whole network
   directory), or found a two-node throwaway with the runbook's steps
   1–4 and 8 — `init`, `up`, `ctx set-nodes`, `harvest`, `assemble`,
   `destroy`; no `configure`.
2. Replace this directory's contents with it, dropping `nodes/`.
3. Update the table above — the release tag the cohort pinned (the
   stack's `image`), the `measurement_id`, the manifest's name and the
   `network_id` — and the unit tests that embed this directory's files:
   `tee/cli/common/src/manifest.rs` asserts the manifest's name and pins
   its SHA-256 on purpose (that pin is what catches the enclave crate
   changing how `network_id` is derived), so both move with the fixture.
4. `make -C tee/cli check`.
