//! Replay the committed founding archives — a real Azure TDX cohort's quotes,
//! re-verified on every PR.
//!
//! The unit suite builds synthetic records and asserts the failure paths;
//! nothing in it verifies a quote a real box produced. This file does: for
//! every network directory git tracks that carries a founding archive under
//! `inputs/harvest/`, it runs the function behind `seismic-tee verify-founding`
//! — the manifest-committed policy, every archived record replayed against the
//! collateral and trust anchors archived inside it, the harvested key set
//! against the seated validator set. Today that is `tee/networks/fixture-devnet/`
//! alone. A committed directory without an archive is skipped by name; a run
//! that replays nothing at all fails, since a gate with no fixture guards
//! nothing.
//!
//! Hermetic: the archive is self-contained and the replay holds every
//! freshness check to each record's own `verified_at`, so this reaches no
//! collateral service, needs no `seismic-reth`, and belongs in the default
//! `cargo nextest run` under the required CI job. What it catches is drift in
//! the enclave crates the workspace pins — the verifier, the record schema, the
//! policy and manifest schemas — against quotes that were once good, and it
//! fires on the PR that bumps the pin. A change that fails it on purpose is a
//! fixture refresh (`tee/networks/README.md`), not a reason to loosen it.
//!
//! The cross-repo half of the same walk — the gates that need `seismic-reth`
//! and the sibling repos' current state — is `drift.rs`.

mod support;

use std::path::Path;

use seismic_tee_common::NetworkDir;
use seismic_tee_common::network_dir::{HARVEST_DIRNAME, INPUTS_DIRNAME};
use seismic_tee_network::verify_founding::audit_founding;
use support::committed_network_dirs;

/// The committed directories that carry a founding archive — at least one,
/// or every test here would pass over nothing.
fn committed_archives() -> Vec<NetworkDir> {
    let archives: Vec<NetworkDir> = committed_network_dirs()
        .into_iter()
        .filter(|dir| {
            let archived = dir.harvest().is_dir();
            if !archived {
                eprintln!(
                    "{}: no {INPUTS_DIRNAME}/{HARVEST_DIRNAME}/, nothing to replay — skipped",
                    dir.root().display()
                );
            }
            archived
        })
        .collect();
    assert!(
        !archives.is_empty(),
        "no committed network directory carries a founding archive — the replay fixture is \
         gone; see tee/networks/README.md"
    );
    archives
}

/// Every committed founding archive still re-verifies under the pinned
/// enclave crates.
#[test]
fn committed_founding_archives_replay() {
    for dir in committed_archives() {
        let audit = audit_founding(&dir, None)
            .unwrap_or_else(|e| panic!("{}: {e:?}", dir.root().display()));
        assert!(
            !audit.verified.is_empty(),
            "{}: an archive with no records",
            dir.root().display()
        );
        eprintln!(
            "{}: {} record(s) re-verified: {}",
            dir.root().display(),
            audit.verified.len(),
            audit.verified.join(", ")
        );
    }
}

/// A working copy of a committed directory, to tamper with.
fn copy_of(dir: &NetworkDir) -> (tempfile::TempDir, NetworkDir) {
    fn copy_tree(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), &target).unwrap();
            }
        }
    }
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("copy");
    copy_tree(dir.root(), &root);
    (tmp, NetworkDir::new(root))
}

/// The first archived record, by node name.
fn first_record(dir: &NetworkDir) -> String {
    let mut names: Vec<String> = std::fs::read_dir(dir.harvest())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| path.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names.remove(0)
}

/// Rewrite one string field of a record with a character in its middle
/// changed: a quote whose signature no longer covers it, a TCB info whose
/// signature does not either, a key the genesis does not seat.
fn alter_field(dir: &NetworkDir, record: &str, pointer: &str) {
    let path = dir.harvest_record(record);
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let field = document
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("{}: no {pointer}", path.display()));
    let text = field.as_str().expect("a string field").to_string();
    let middle = text.len() / 2;
    let flipped = if &text[middle..=middle] == "A" {
        "B"
    } else {
        "A"
    };
    *field = serde_json::Value::String(format!(
        "{}{flipped}{}",
        &text[..middle],
        &text[middle + 1..]
    ));
    std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();
}

/// The replay is a verification, not a checksum of the files: an altered
/// quote, altered collateral, and a key the genesis does not seat each fail
/// by record name, and an altered policy fails against the manifest's pin.
#[test]
fn an_altered_archive_fails_by_name() {
    for dir in committed_archives() {
        let record = first_record(&dir);
        let failure = |tampered: &NetworkDir| -> String {
            let err = audit_founding(tampered, None).expect_err("tampering must fail the replay");
            format!("{err:?}")
        };

        for (what, pointer) in [
            ("quote", "/evidence/attestation_evidence/quote"),
            ("collateral", "/dcap_collateral/tcb_info"),
        ] {
            let (_tmp, copy) = copy_of(&dir);
            alter_field(&copy, &record, pointer);
            let err = failure(&copy);
            assert!(
                err.contains(&format!("{record}:")),
                "{}: altered {what} did not fail by name: {err}",
                dir.root().display()
            );
        }

        let (_tmp, copy) = copy_of(&dir);
        let path = copy.harvest_record(&record);
        let mut document: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        document["node_public_key"] = serde_json::Value::String("00".repeat(32));
        std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();
        let err = failure(&copy);
        assert!(
            err.contains(&format!("does not seat: {record}")),
            "{}: altered key did not fail by name: {err}",
            dir.root().display()
        );

        let (_tmp, copy) = copy_of(&dir);
        let policy = std::fs::read(copy.policy()).unwrap();
        std::fs::write(copy.policy(), [policy.as_slice(), b"\n"].concat()).unwrap();
        let err = failure(&copy);
        assert!(
            err.contains("bootstrap_policy_hash mismatch"),
            "{}: altered policy did not fail against the pin: {err}",
            dir.root().display()
        );
    }
}
