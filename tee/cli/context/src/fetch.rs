//! `ctx set-network --dir <URL>`: a network directory published in a GitHub
//! repository, fetched into a local directory every command then reads like
//! any other.
//!
//! The fetch is git's — shallow, blobless and sparse — so of the repository
//! only the one directory comes down, and a private repository works with
//! whatever credentials git already has. A tree URL's `<ref>/<path>` is
//! ambiguous once a branch name holds a `/` (`sl/x/tee/networks/devnet`), so
//! the split is resolved against the remote's branches and tags, longest
//! first; a ref that is none of them must be a full commit hash.
//!
//! The directory is staged under `networks/` and renamed into place as
//! `networks/<name>/` — `name` the caller's, else the manifest's own — only
//! once its manifest hashes to the pin, when one is given.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context as _, bail};
use seismic_tee_common::{Manifest, NetworkDir};

use crate::config::check_network_name;

/// The networks `ctx set-network` offers when asked for its directory: the
/// ones committed to the monorepo.
pub const PUBLISHED_NETWORKS: &str =
    "https://github.com/SeismicSystems/seismic/tree/main/tee/networks";

const GITHUB: &str = "https://github.com/";
const TREE_URL: &str = "https://github.com/<owner>/<repo>/tree/<ref>/<path>";

/// A network directory at a GitHub tree URL, not yet fetched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Published {
    url: String,
    /// What git fetches from: the repository's URL.
    remote: String,
    /// `<ref>/<path>`, split only once the remote's refs are known.
    rest: String,
}

/// What a fetch brought down, and where it put it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    /// The name it was installed under: the caller's, else the manifest's.
    pub name: String,
    pub dir: PathBuf,
    pub commit: String,
    /// The manifest's network_id, bare hex.
    pub network_id: String,
}

impl Published {
    /// `location` as a URL to fetch, or `None` for a local path.
    pub fn parse(location: &str) -> anyhow::Result<Option<Self>> {
        if !location.contains("://") {
            return Ok(None);
        }
        if location.starts_with("http://") {
            bail!("insecure URL rejected (use https://): {location}");
        }
        let Some(after) = location.strip_prefix(GITHUB) else {
            bail!(
                "{location}: only a GitHub directory can be fetched ({TREE_URL}) — fetch \
                 anything else yourself and pass its local path"
            );
        };
        let after = after.split(['?', '#']).next().unwrap_or_default();
        let parts: Vec<&str> = after.trim_end_matches('/').splitn(4, '/').collect();
        let [owner, repo, "tree" | "blob", rest] = parts[..] else {
            bail!("{location}: not a GitHub directory URL — expected {TREE_URL}");
        };
        if !rest.contains('/') {
            bail!(
                "{location} names a repository's root — name the network directory inside it \
                 ({TREE_URL})"
            );
        }
        Ok(Some(Self {
            url: location.to_string(),
            remote: format!("{GITHUB}{owner}/{repo}"),
            rest: rest.to_string(),
        }))
    }

    /// The URL's last path segment, which names the staging directory.
    fn last_segment(&self) -> &str {
        self.rest.rsplit('/').next().expect("rsplit yields one")
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    /// Fetch the directory to `<networks>/<name>/` — `name`, else the
    /// manifest's — which must not exist, refusing it unless its manifest
    /// hashes to `pin` when one is given.
    pub fn fetch(
        &self,
        networks: &Path,
        name: Option<&str>,
        pin: Option<&str>,
    ) -> anyhow::Result<Fetched> {
        // A name known up front is checked before anything is fetched.
        if let Some(name) = name {
            free_destination(networks, name)?;
        }
        let (git_ref, path) = self.split()?;
        let staging = Staging::new(networks, self.last_segment())?;
        let root = staging.0.as_path();
        git(None, &["init", "-q", &root.to_string_lossy()])?;
        git(
            Some(root),
            &["sparse-checkout", "set", "--no-cone", &format!("/{path}/")],
        )?;
        git(
            Some(root),
            &[
                "fetch",
                "-q",
                "--depth",
                "1",
                "--filter=blob:none",
                &self.remote,
                &git_ref,
            ],
        )
        .with_context(|| format!("fetching {}", self.url))?;
        git(
            Some(root),
            &[
                "-c",
                "advice.detachedHead=false",
                "checkout",
                "-q",
                "FETCH_HEAD",
            ],
        )?;
        let commit = git(Some(root), &["rev-parse", "FETCH_HEAD"])?;

        let fetched_dir = root.join(&path);
        if !fetched_dir.is_dir() {
            bail!("{} names no directory at commit {commit}", self.url);
        }
        let manifest_path = NetworkDir::new(&fetched_dir).manifest();
        if !manifest_path.is_file() {
            bail!(
                "{} is not a network directory: it has no {} at commit {commit}",
                self.url,
                seismic_tee_common::network_dir::MANIFEST_FILENAME,
            );
        }
        let manifest = Manifest::load(&manifest_path)
            .with_context(|| format!("{}: invalid manifest", self.url))?;
        let derived = manifest.network_id().to_string();
        let network_id = derived.strip_prefix("0x").unwrap_or(&derived).to_string();
        if let Some(pin) = pin
            && pin != network_id
        {
            bail!(
                "network_id mismatch: {} hashes to {derived}, not the 0x{pin} --network-id \
                 pins — nothing was registered",
                self.url,
            );
        }
        let name = match name {
            Some(name) => name.to_string(),
            None => {
                check_network_name(&manifest.name)
                    .with_context(|| format!("{}: the manifest's name", self.url))?;
                manifest.name.clone()
            }
        };
        let dest = free_destination(networks, &name)?;
        std::fs::rename(&fetched_dir, &dest)
            .with_context(|| format!("moving the fetched directory to {}", dest.display()))?;
        Ok(Fetched {
            name,
            dir: dest,
            commit,
            network_id,
        })
    }

    /// The network directories directly under this one — those with a
    /// manifest — as `(name, tree URL)`, read from a scratch repository that
    /// fetches the commit's trees and none of its files.
    pub fn networks(&self) -> anyhow::Result<Vec<(String, String)>> {
        let (git_ref, path) = self.split()?;
        let scratch = tempfile::tempdir().context("creating a scratch repository")?;
        let root = scratch.path();
        git(None, &["init", "-q", &root.to_string_lossy()])?;
        git(
            Some(root),
            &[
                "fetch",
                "-q",
                "--depth",
                "1",
                "--filter=blob:none",
                &self.remote,
                &git_ref,
            ],
        )
        .with_context(|| format!("fetching {}", self.url))?;
        let listing = git(
            Some(root),
            &[
                "ls-tree",
                "-r",
                "--name-only",
                "FETCH_HEAD",
                "--",
                &format!("{path}/"),
            ],
        )?;
        let manifest = format!("/{}", seismic_tee_common::network_dir::MANIFEST_FILENAME);
        Ok(listing
            .lines()
            .filter_map(|file| {
                file.strip_prefix(&path)?
                    .strip_prefix('/')?
                    .strip_suffix(&manifest)
            })
            .filter(|name| !name.contains('/'))
            .map(|name| {
                let url = format!("{}/tree/{git_ref}/{path}/{name}", self.remote);
                (name.to_string(), url)
            })
            .collect())
    }

    /// `rest` as `(ref, path)`: the longest branch or tag `rest` starts
    /// with, else a leading full commit hash.
    fn split(&self) -> anyhow::Result<(String, String)> {
        let listing = git(None, &["ls-remote", &self.remote])
            .with_context(|| format!("listing the refs of {}", self.remote))?;
        let mut refs: Vec<&str> = listing
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .filter_map(|(_, name)| {
                name.strip_prefix("refs/heads/")
                    .or_else(|| name.strip_prefix("refs/tags/"))
            })
            .map(|name| name.trim_end_matches("^{}"))
            .collect();
        refs.sort_by_key(|name| std::cmp::Reverse(name.len()));
        if let Some(path) = refs
            .iter()
            .find_map(|name| self.rest.strip_prefix(name)?.strip_prefix('/'))
        {
            let git_ref = &self.rest[..self.rest.len() - path.len() - 1];
            return Ok((git_ref.to_string(), path.to_string()));
        }
        let (head, path) = self.rest.split_once('/').expect("parse checked a `/`");
        if head.len() == 40 && head.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Ok((head.to_string(), path.to_string()));
        }
        bail!(
            "{}: `{head}` is no branch or tag of {}, nor a full commit hash",
            self.url,
            self.remote
        )
    }

    /// A URL's parts as given, for a test fetching from a local repository.
    #[cfg(test)]
    pub(crate) fn at(remote: &str, rest: &str) -> Self {
        Self {
            url: format!("{remote}/tree/{rest}"),
            remote: remote.to_string(),
            rest: rest.to_string(),
        }
    }
}

/// `<networks>/<name>`, refused when something is already there.
fn free_destination(networks: &Path, name: &str) -> anyhow::Result<PathBuf> {
    let dest = networks.join(name);
    if dest.exists() {
        bail!(
            "{} already exists — remove it (`seismic-tee network rm {name}` when it is \
             registered), or pass --name to fetch under another name",
            dest.display()
        );
    }
    Ok(dest)
}

/// The staging directory under `networks/`, removed on drop: a failed fetch
/// leaves nothing behind, a finished one only what it moved out. Dot-led, so
/// no network name can collide with it.
struct Staging(PathBuf);

impl Staging {
    fn new(networks: &Path, label: &str) -> anyhow::Result<Self> {
        std::fs::create_dir_all(networks)
            .with_context(|| format!("creating {}", networks.display()))?;
        let staging = networks.join(format!(".{label}.fetch"));
        if staging.exists() {
            std::fs::remove_dir_all(&staging)
                .with_context(|| format!("removing a stale {}", staging.display()))?;
        }
        Ok(Self(staging))
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Run git, in `dir` when given, returning its trimmed stdout. Never
/// prompts: a repository git has no credentials for fails instead.
fn git(dir: Option<&Path>, args: &[&str]) -> anyhow::Result<String> {
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.arg("-C").arg(dir);
    }
    let output = command
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .context("running git — fetching a network directory needs git on PATH")?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_is_no_url() {
        assert_eq!(Published::parse("tee/networks/x").unwrap(), None);
        assert_eq!(Published::parse("/abs/x").unwrap(), None);
    }

    #[test]
    fn a_tree_or_blob_url_parses_to_its_repository() {
        for kind in ["tree", "blob"] {
            let url = format!(
                "https://github.com/SeismicSystems/seismic/{kind}/main/tee/networks/fixture-devnet/"
            );
            let published = Published::parse(&url).unwrap().unwrap();
            assert_eq!(
                published.remote,
                "https://github.com/SeismicSystems/seismic"
            );
            assert_eq!(published.rest, "main/tee/networks/fixture-devnet");
            assert_eq!(published.last_segment(), "fixture-devnet");
        }
    }

    #[test]
    fn a_url_that_cannot_be_fetched_is_refused_by_kind() {
        for (url, expected) in [
            ("http://github.com/o/r/tree/main/x", "insecure URL"),
            (
                "https://example.com/o/r/tree/main/x",
                "only a GitHub directory",
            ),
            ("https://github.com/o/r", "not a GitHub directory URL"),
            (
                "https://github.com/o/r/releases/tag/v1",
                "not a GitHub directory URL",
            ),
            (
                "https://github.com/o/r/tree/main",
                "names a repository's root",
            ),
        ] {
            let err = Published::parse(url).unwrap_err().to_string();
            assert!(err.contains(expected), "{url}: {err}");
        }
    }

    /// Run git hermetically for a fixture: no global or system config, so a
    /// developer's signing setup never fires.
    fn fixture_git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.com")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.com")
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    }

    /// A repository with the fixture manifest at `tee/networks/net-1` on
    /// branch `sl/feature`, beside a file the sparse fetch must leave out.
    fn remote_repo(root: &Path) -> String {
        let repo = root.join("remote");
        let net = repo.join("tee/networks/net-1");
        std::fs::create_dir_all(net.join("inputs/harvest")).unwrap();
        std::fs::write(
            net.join("network-manifest.json"),
            seismic_tee_common::test_support::manifest_pinning(b"{}"),
        )
        .unwrap();
        std::fs::write(net.join("inputs/harvest/node-1.json"), "{}").unwrap();
        std::fs::write(repo.join("unrelated"), "x").unwrap();
        fixture_git(&repo, &["init", "-q", "-b", "sl/feature"]);
        fixture_git(&repo, &["config", "uploadpack.allowFilter", "true"]);
        fixture_git(&repo, &["add", "-A"]);
        fixture_git(&repo, &["commit", "-q", "-m", "net-1"]);
        format!("file://{}", repo.display())
    }

    fn manifest_id(dir: &Path) -> String {
        let id = Manifest::load(&NetworkDir::new(dir).manifest())
            .unwrap()
            .network_id()
            .to_string();
        id.trim_start_matches("0x").to_string()
    }

    /// The fixture manifest's `name`, which differs from its directory's.
    const MANIFEST_NAME: &str = "seismic-devnet-3";

    /// Only a directory holding a manifest is a network; one scaffolded but
    /// not yet assembled, or nested deeper, is not offered.
    #[test]
    fn the_networks_under_a_directory_are_those_with_a_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let remote = remote_repo(tmp.path());
        let repo = tmp.path().join("remote");
        std::fs::create_dir_all(repo.join("tee/networks/scaffolded/inputs")).unwrap();
        std::fs::write(repo.join("tee/networks/scaffolded/inputs/x.json"), "{}").unwrap();
        fixture_git(&repo, &["add", "-A"]);
        fixture_git(&repo, &["commit", "-q", "-m", "scaffolded"]);

        let networks = Published::at(&remote, "sl/feature/tee/networks")
            .networks()
            .unwrap();
        assert_eq!(
            networks,
            [(
                "net-1".to_string(),
                format!("{remote}/tree/sl/feature/tee/networks/net-1")
            )]
        );
    }

    #[test]
    fn a_slashed_branch_fetches_the_one_directory_named_after_its_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let remote = remote_repo(tmp.path());
        let networks = tmp.path().join("networks");

        let fetched = Published::at(&remote, "sl/feature/tee/networks/net-1")
            .fetch(&networks, None, None)
            .unwrap();

        assert_eq!(fetched.name, MANIFEST_NAME);
        assert_eq!(fetched.dir, networks.join(MANIFEST_NAME));
        assert!(fetched.dir.join("inputs/harvest/node-1.json").is_file());
        assert!(!fetched.dir.join(".git").exists());
        assert!(!networks.join(".net-1.fetch").exists());
        assert_eq!(fetched.commit.len(), 40);
        assert_eq!(fetched.network_id, manifest_id(&fetched.dir));
    }

    #[test]
    fn a_given_name_beats_the_manifests() {
        let tmp = tempfile::tempdir().unwrap();
        let remote = remote_repo(tmp.path());
        let networks = tmp.path().join("networks");

        let fetched = Published::at(&remote, "sl/feature/tee/networks/net-1")
            .fetch(&networks, Some("mine"), None)
            .unwrap();

        assert_eq!(fetched.name, "mine");
        assert!(networks.join("mine/network-manifest.json").is_file());
    }

    #[test]
    fn a_mismatched_pin_leaves_nothing_behind() {
        let tmp = tempfile::tempdir().unwrap();
        let remote = remote_repo(tmp.path());
        let networks = tmp.path().join("networks");

        let err = Published::at(&remote, "sl/feature/tee/networks/net-1")
            .fetch(&networks, None, Some(&"ab".repeat(32)))
            .unwrap_err()
            .to_string();

        assert!(err.contains("network_id mismatch"), "{err}");
        assert_eq!(std::fs::read_dir(&networks).unwrap().count(), 0);
    }

    #[test]
    fn an_existing_destination_or_unknown_ref_is_refused() {
        let tmp = tempfile::tempdir().unwrap();
        let remote = remote_repo(tmp.path());
        let networks = tmp.path().join("networks");

        let err = Published::at(&remote, "nope/tee/networks/net-1")
            .fetch(&networks, None, None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("`nope` is no branch or tag"), "{err}");

        // Named by the manifest: found taken only after the fetch, which
        // then leaves nothing staged.
        std::fs::create_dir_all(networks.join(MANIFEST_NAME)).unwrap();
        let err = Published::at(&remote, "sl/feature/tee/networks/net-1")
            .fetch(&networks, None, None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("already exists"), "{err}");
        assert!(!networks.join(".net-1.fetch").exists());
    }

    #[test]
    fn a_directory_without_a_manifest_is_no_network() {
        let tmp = tempfile::tempdir().unwrap();
        let remote = remote_repo(tmp.path());

        let err = Published::at(&remote, "sl/feature/tee/networks/net-1/inputs")
            .fetch(&tmp.path().join("networks"), None, None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("is not a network directory"), "{err}");
    }
}
