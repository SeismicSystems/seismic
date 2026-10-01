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
| Founded | 2026-10-01, by hand, from [the devnet runbook](https://github.com/SeismicSystems/deploy/blob/8d0048de2a951044a48a3ff0215dc7b87fe012ab/tee/runbook-devnet.md), configured and smoke-tested before teardown |
| Image | [`seismic_2026-10-01.6a90ed`](https://github.com/SeismicSystems/seismic-images/releases/tag/seismic_2026-10-01.6a90ed) (`inputs/image.json`): summit `5eb9f47`, enclave `8b5833a`, seismic-reth `39d04d1` |
| `measurement_id` | `seismic_2026-10-01.6a90ed.vhd` (`inputs/measurements.json`) |
| Records | 4, `inputs/harvest/tee-devnet-{1..4}.json` (harvested as `samlaf-fixture-devnet-tee-devnet-{1..4}`; the operator prefix is dropped, the stem being only a label), record `version` 1 |
| Verifier at founding | dcap-qvl 0.5.2 (`trust_anchors.dcap_qvl_version`) |
| `network_id` | `0xfa2dd4423242449103f1057b82480c47bc7574e0d16a639e17009091c22e64ee` |

The cohort was founded under this directory's own name, so the
manifest's `name` and `namespace` are `fixture-devnet`. Both are part of
the manifest bytes: a refresh founded under another name keeps that name
as assembled ([../README.md](../README.md) on renaming a throwaway).
The cohort's descriptor (`nodes/`) is not here — a dead cohort's IPs,
gitignored like every network's.

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
3. Update the table above — the release tag the cohort booted
   (`inputs/image.json`'s `image`, which is also the release whose
   binaries the `drift` job in `.github/workflows/seismic-tee.yml` runs),
   the `measurement_id`, the manifest's name and the `network_id` — and
   the unit tests that embed this directory's files:
   `tee/cli/common/src/manifest.rs` asserts the manifest's name and pins
   its SHA-256 on purpose (that pin is what catches the enclave crate
   changing how `network_id` is derived), so both move with the fixture.
4. `make -C tee/cli check`.
