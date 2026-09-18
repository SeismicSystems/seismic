//! Shared by the network crate's integration tests: which network directories
//! this repo commits to keeping valid.
//!
//! Two suites walk the same list. `drift.rs` runs the gates that reach outside
//! the workspace over it (its own non-required CI job); `replay.rs` replays
//! each committed founding archive offline (the hermetic suite, under the
//! required job). Enumerating in one place is what keeps a directory from
//! being checked by one and forgotten by the other.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use seismic_tee_common::NetworkDir;
use seismic_tee_common::network_dir::MANIFEST_FILENAME;

/// The repo root: this crate is `tee/cli/network/`.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("network/ sits three levels under the repo root")
        .to_path_buf()
}

pub fn networks_dir() -> PathBuf {
    repo_root().join("tee").join("networks")
}

/// Network directories git tracks — every tracked directory under
/// `tee/networks/` carrying a manifest.
///
/// A real deployment writes its network directory here too, so enumerating
/// the filesystem would validate whichever devnet the developer last founded.
/// Only the committed ones are this repo's to keep passing. `git ls-files`
/// reads the index, so a directory staged for its first commit is already on
/// the list.
pub fn committed_network_dirs() -> Vec<NetworkDir> {
    let listed = std::process::Command::new("git")
        .args(["ls-files", "-z", "--"])
        .arg(networks_dir())
        .current_dir(repo_root())
        .output()
        .expect("git ls-files");
    assert!(listed.status.success(), "git ls-files failed");
    let dirs: BTreeSet<PathBuf> = String::from_utf8_lossy(&listed.stdout)
        .split('\0')
        .filter(|path| path.ends_with(&format!("/{MANIFEST_FILENAME}")))
        .map(|path| repo_root().join(path).parent().unwrap().to_path_buf())
        .collect();
    assert!(
        !dirs.is_empty(),
        "no committed network directory under {}",
        networks_dir().display()
    );
    dirs.into_iter().map(NetworkDir::new).collect()
}
