//! Cross-repo drift guards (run via `make -C tee/cli drift`).
//!
//! These check this repo against the current state of its sibling repos,
//! reached either by fetching a pinned artifact over HTTP or by running a
//! binary built from a sibling branch. Every test needing something outside
//! this workspace belongs here, so `make test` stays hermetic with nothing to
//! skip and every test runs in exactly one CI job. CI runs this module as its
//! own non-required job, where a failure names the exact cross-repo check.
//!
//! `#[ignore]` marks the whole file: the default `cargo nextest run` skips it,
//! and the drift target runs it with `--run-ignored only`. The suite never
//! skips *within* itself — a missing prerequisite is a failure, because a
//! guard that quietly passes when its tooling is missing is how a committed
//! artifact goes stale unnoticed. It needs:
//!
//! - network reach to raw.githubusercontent.com (the cross-repo tests fetch
//!   pinned artifacts from sibling repos);
//! - `seismic-reth` and `summit` on PATH, for the `genesis-hash` and `genesis
//!   digest` subcommands (CI installs a prebuilt release of each, with the
//!   setup-sreth and setup-summit actions).
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

use std::collections::BTreeSet;

use seismic_manifest::render;
use seismic_tee_network::gates::{ArtifactSet, run_validation_gates};
use seismic_tee_network::shell_outs::ShellOuts;
use support::{committed_network_dirs, networks_dir};

/// Fetch a cross-repo artifact, failing the calling test if it can't.
///
/// An HTTP 4xx/5xx means the artifact moved or the ref is gone — a real drift
/// signal, not flaky network — so it fails at once. A transport-level failure
/// is retried once, then fails: this suite never skips.
async fn fetch_live(url: &str) -> Vec<u8> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let mut last = None;
    for _ in 0..2 {
        match client.get(url).send().await {
            Ok(response) => {
                let response = response
                    .error_for_status()
                    .unwrap_or_else(|e| panic!("cross-repo artifact gone: {e}"));
                match response.bytes().await {
                    Ok(bytes) => return bytes.to_vec(),
                    Err(e) => last = Some(e),
                }
            }
            Err(e) => last = Some(e),
        }
    }
    panic!("cross-repo artifact unreachable after retry: {last:?}");
}

/// The committed starter summit genesis tracks summit's parameter set.
///
/// `tee/networks/summit-genesis-starter.toml` carries every founder-reviewable
/// summit genesis parameter, explicitly — defaults included, so the founder
/// reviews each one. Summit owns the schema, and its `example_genesis.toml` is
/// a complete rendering of it, so a parameter summit adds or renames shows up
/// as a key-set mismatch here. Values are not compared: each is a per-network
/// choice.
#[tokio::test]
#[ignore = "cross-repo: fetches summit's example genesis"]
async fn starter_carries_summits_parameter_set() {
    const URL: &str =
        "https://raw.githubusercontent.com/SeismicSystems/summit/main/example_genesis.toml";
    // Not parameters: the two fields assemble derives per network.
    let derived: BTreeSet<&str> = ["eth_genesis_hash", "validators"].into();

    let example: toml::Table =
        toml::from_str(std::str::from_utf8(&fetch_live(URL).await).unwrap()).unwrap();
    let starter: toml::Table = toml::from_str(
        &std::fs::read_to_string(networks_dir().join("summit-genesis-starter.toml")).unwrap(),
    )
    .unwrap();
    let starter_keys: BTreeSet<&str> = starter.keys().map(String::as_str).collect();
    let example_keys: BTreeSet<&str> = example.keys().map(String::as_str).collect();
    assert!(starter_keys.is_disjoint(&derived), "{starter_keys:?}");
    assert_eq!(&starter_keys | &derived, example_keys);
    // The namespace slot ships empty: the visible fill-me init replaces with
    // the network name (unique per network — replay domain).
    assert_eq!(starter["namespace"].as_str(), Some(""));
}

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
