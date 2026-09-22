//! Stamps the binary with the commit it was built from.
//!
//! `--version` prints the crate version and then this commit. Every
//! `seismic-tee/main-<sha>` prerelease shares its crate version with the
//! release before it, so the version alone cannot say which build an operator
//! is running, and the record a founder keeps beside a founding wants the
//! exact commit. It is read from `git` in the checkout being built — a `cargo
//! install --path`, a `cargo install --git` checkout, a CI runner's clone —
//! with `-dirty` appended when tracked files differ from HEAD.
//! `SEISMIC_TEE_BUILD_COMMIT` in the environment wins outright, for a build
//! from a source archive with no `.git`; with neither, the stamp is `unknown`
//! rather than a failed build.
//!
//! The script re-runs when HEAD moves, when the branch it names moves, or
//! when the index changes (a `git add` or a commit) — not on every edit, so
//! `-dirty` can lag an unstaged change by one build. A release build is a
//! clean checkout at a fixed commit, where none of this arises.

use std::process::Command;

const VAR: &str = "SEISMIC_TEE_BUILD_COMMIT";

fn main() {
    println!("cargo:rerun-if-env-changed={VAR}");
    let commit = std::env::var(VAR)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(from_git)
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env={VAR}={commit}");
}

/// `git <args>`'s stdout, trimmed, when it exits 0.
fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|out| !out.is_empty())
}

fn from_git() -> Option<String> {
    let commit = git(&["rev-parse", "--short=9", "HEAD"])?;
    // Asked of git rather than spelled as `.git/...`: in a worktree `.git` is
    // a file pointing elsewhere, and `--git-path` follows it.
    let mut watched = vec![
        "HEAD".to_string(),
        "index".to_string(),
        "packed-refs".to_string(),
    ];
    if let Some(head_ref) = git(&["symbolic-ref", "-q", "HEAD"]) {
        watched.push(head_ref);
    }
    for path in watched {
        if let Some(path) = git(&["rev-parse", "--git-path", &path]) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    // Tracked files only, like `git describe --dirty`: an untracked scratch
    // file is not a change to what was built.
    let dirty = git(&["status", "--porcelain", "--untracked-files=no"]).is_some();
    Some(if dirty {
        format!("{commit}-dirty")
    } else {
        commit
    })
}
