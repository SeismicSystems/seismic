//! Cross-repo drift guards (run via `make -C tee/cli drift`).
//!
//! These check this repo against the current state of its sibling repos, by
//! running binaries built from a sibling branch. Every test needing something
//! outside this workspace belongs here, so `make test` stays hermetic with
//! nothing to skip and every test runs in exactly one CI job. CI runs this
//! module as its own non-required job, where a failure names the exact
//! cross-repo check.
//!
//! `#[ignore]` marks the whole file: the default `cargo nextest run` skips it,
//! and the drift target runs it with `--run-ignored only`. The suite never
//! skips *within* itself — a missing prerequisite is a failure, because a
//! guard that quietly passes when its tooling is missing is how a committed
//! artifact goes stale unnoticed. It needs `seismic-reth` and `summit` on
//! PATH, for the `genesis-hash` and `genesis digest` subcommands (CI installs
//! a prebuilt release of each, with the setup-sreth and setup-summit actions).
//!
//! The enclave crates under test — the admission compiler, the manifest
//! renderer and schema — are linked at the rev the workspace pins, so moving
//! them is a PR that bumps the pin, not something CI discovers.
//!
//! Two more checks over the pinned enclave crates are *not* here because they
//! reach nothing outside the workspace, so they run in the hermetic suite:
//! replaying the committed founding archives through the pinned verifier
//! (`replay.rs`, walking the same `support::committed_network_dirs` list),
//! and holding the admission crate's registry runtime-code pin to this repo's
//! contract artifact (`registry_pin.rs`).

mod support;

use seismic_manifest::render;
use seismic_tee_network::gates::{ArtifactSet, run_validation_gates};
use seismic_tee_network::shell_outs::ShellOuts;
use support::committed_network_dirs;

/// A committed network directory's manifest is exactly what the renderer
/// renders from its own values: a rendering change would re-found every
/// existing network on its next assemble.
#[test]
#[ignore = "cross-repo: the committed network directories against the pinned renderer"]
fn committed_manifests_are_the_renderers_bytes() {
    for dir in committed_network_dirs() {
        let committed = std::fs::read(dir.manifest()).unwrap();
        let parsed = seismic_manifest::NetworkManifestV1::from_json_bytes(&committed).unwrap();
        assert_eq!(
            render(&parsed),
            committed,
            "{} is not its own rendering",
            dir.manifest().display()
        );
    }
}

/// Committed network directories still pass their own gates.
///
/// `tee/networks/fixture-devnet/` is a real founding's artifact set, committed
/// as the documented shape and the hermetic suite's replay fixture; the unit
/// tests embed a few of its files, but nothing recomputes its gates. Re-running
/// the real ones over it keeps the fixture honest, and turns a semantic change
/// in the admission compiler or in reth's genesis-header encoding into a
/// failure here rather than a surprise at the next `assemble`.
///
/// Every gate recomputes for real — genesis hash, summit's genesis config
/// digest, chain id, policy hash, contract accounts, and the exact
/// registry-account storage — against the sibling binaries on PATH, which is
/// what makes this a cross-repo guard rather than a re-read of the manifest.
#[tokio::test]
#[ignore = "cross-repo: needs seismic-reth and summit on PATH"]
async fn committed_network_dirs_pass_their_gates() {
    for dir in committed_network_dirs() {
        let set = ArtifactSet::load(&dir).unwrap();
        let derive = ShellOuts::default();
        let warnings = run_validation_gates(&set, &derive)
            .await
            .unwrap_or_else(|e| panic!("{}: {e:?}", dir.root().display()));
        assert!(
            warnings.is_empty(),
            "{}: {warnings:?}",
            dir.root().display()
        );
    }
}
