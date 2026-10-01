//! The context file's schema: named networks, and which one — and which node
//! — the commands act on.
//!
//! Every network entry is validated once, in [`Config::validate`], so a
//! malformed file fails before any command acts on it rather than partway
//! through one. A network's cohort lives inline, under
//! `[networks.<name>.nodes]`, the way a kubeconfig stores each cluster's
//! endpoint inline rather than pointing at a file: the provisioner writes it
//! (`pulumi stack output nodes --json | seismic-tee ctx set-nodes <network>`)
//! and every command here only reads it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::bail;
use serde::Deserialize;

use seismic_tee_common::Descriptors;

/// The context file: named networks, and which one (and which of its nodes)
/// the commands act on.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The selection: `<network>` or `<network>/<node>`.
    pub current: Option<String>,
    /// What `current` was before the last `ctx use`, for `ctx use -`.
    pub previous: Option<String>,
    #[serde(default)]
    pub networks: BTreeMap<String, Network>,
}

/// One network: where its artifact set is, and its cohort.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    /// A network directory: the artifact set — manifest, genesis, policy,
    /// harvest records — local, or fetched by `ctx set-network --dir <URL>`.
    /// Never the cohort; that is `nodes`. Absent for a network that is
    /// nothing but nodes (enough for `status`, `env`, `exec`).
    pub dir: Option<PathBuf>,
    /// The URL `dir` was fetched from, kept as provenance: nothing reads
    /// the network from it again.
    pub source: Option<String>,
    /// SHA-256 of network-manifest.json, bare hex: the pin a directory is
    /// refused unless it matches.
    pub network_id: Option<String>,
    /// The cohort: node name → address, exactly as the provisioner reported
    /// it. Written by `ctx set-nodes` from `pulumi stack output nodes --json`.
    #[serde(default)]
    pub nodes: Descriptors,
}

impl Network {
    /// A network entry naming a directory, with no pin or nodes — what
    /// `network init` registers for the directory it just scaffolded.
    pub fn of_dir(dir: &Path) -> Self {
        Self {
            dir: Some(dir.to_path_buf()),
            ..Default::default()
        }
    }

    /// Validate this entry, naming `name` and `config_path` in every
    /// failure.
    pub fn validate(&self, name: &str, config_path: &Path) -> anyhow::Result<()> {
        // Before the emptiness check: without nodes, a source-only entry would
        // read as empty, hiding the URL that fills it.
        if let Some(source) = &self.source
            && self.dir.is_none()
        {
            bail!(
                "network `{name}` in {} has a source ({source}) but no dir — source records where \
                 a fetched dir came from; `seismic-tee ctx set-network --name {name} --dir \
                 {source}` fetches it",
                config_path.display(),
            );
        }
        if self.dir.is_none() && self.nodes.is_empty() {
            bail!(
                "network `{name}` in {} is empty — register its directory with `seismic-tee ctx \
                 set-network --name {name} --dir <PATH|URL>`, or import its nodes with \
                 `seismic-tee ctx set-nodes {name}`",
                config_path.display(),
            );
        }
        if let Some(id) = &self.network_id
            && !is_network_id(id)
        {
            bail!(
                "network `{name}` in {}: network_id must be 32 bytes of hex, got {id}",
                config_path.display(),
            );
        }
        Ok(())
    }
}

/// Refuse `name` unless it can be both a context key — the `<network>` of
/// `<network>/<node>` — and one directory under `networks/`. A fetched
/// manifest's name is the remote's to choose, so `..` or `a/b` must not
/// reach a path.
pub fn check_network_name(name: &str) -> anyhow::Result<()> {
    if name.is_empty() || name.starts_with('.') || name.contains(['/', '\\']) {
        bail!(
            "`{name}` cannot name a network: a name is one path segment, not starting with `.` \
             — pass --name"
        );
    }
    Ok(())
}

/// Whether `id` is a network_id as the file stores it: 32 bytes of bare hex.
pub fn is_network_id(id: &str) -> bool {
    id.len() == 64 && id.bytes().all(|b| b.is_ascii_hexdigit())
}

impl Config {
    /// Validate every network entry's shape, naming `path` in every failure.
    pub fn validate(&self, path: &Path) -> anyhow::Result<()> {
        for (name, network) in &self.networks {
            network.validate(name, path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn network(toml: &str) -> Network {
        toml::from_str::<Network>(toml).unwrap()
    }

    fn one_node() -> &'static str {
        r#"[nodes.alpha]
public_ip = "203.0.113.7"
fqdn = "alpha.example.com""#
    }

    #[test]
    fn of_dir_names_a_dir_entry() {
        let n = Network::of_dir(Path::new("/nets/devnet-1"));
        n.validate("devnet-1", Path::new("config.toml")).unwrap();
        assert_eq!(n.dir.as_deref(), Some(Path::new("/nets/devnet-1")));
        assert!(n.nodes.is_empty());
    }

    #[test]
    fn a_network_name_is_one_segment_not_starting_with_a_dot() {
        for good in ["fixture-devnet", "seismic-devnet-3", "a.b"] {
            check_network_name(good).unwrap();
        }
        for bad in ["", ".", "..", ".hidden", "a/b", "a\\b"] {
            let err = check_network_name(bad).unwrap_err().to_string();
            assert!(err.contains("cannot name a network"), "{bad}: {err}");
        }
    }

    #[test]
    fn a_nodes_only_entry_is_legal() {
        let n = network(one_node());
        n.validate("y", Path::new("config.toml")).unwrap();
        assert_eq!(n.nodes.len(), 1);
        assert_eq!(n.dir, None);
    }

    #[test]
    fn an_unknown_key_is_rejected_by_name_on_a_network_and_on_a_node() {
        let err = toml::from_str::<Config>("bogus = 1")
            .unwrap_err()
            .to_string();
        assert!(err.contains("bogus"), "{err}");

        let err = toml::from_str::<Network>(
            r#"dir = "/x"
bogus = 1"#,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("bogus"), "{err}");

        let err = toml::from_str::<Network>(
            r#"[nodes.alpha]
public_ip = "203.0.113.7"
fqdn = "alpha.example.com"
bogus = 1"#,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("bogus"), "{err}");
    }

    #[test]
    fn the_validation_failures_produce_their_message() {
        let path = Path::new("config.toml");

        let err = Network::default()
            .validate("empty", path)
            .unwrap_err()
            .to_string();
        assert!(err.contains("is empty"), "{err}");
        assert!(err.contains("set-network --name empty --dir"), "{err}");
        assert!(err.contains("ctx set-nodes empty"), "{err}");

        let err = network(r#"source = "https://github.com/o/r/tree/main/net""#)
            .validate("orphan", path)
            .unwrap_err()
            .to_string();
        assert!(err.contains("has a source"), "{err}");
        assert!(
            err.contains("--dir https://github.com/o/r/tree/main/net"),
            "{err}"
        );

        let err = network(
            r#"dir = "/x"
network_id = "not-hex""#,
        )
        .validate("bad-id", path)
        .unwrap_err()
        .to_string();
        assert!(err.contains("network_id must be 32 bytes of hex"), "{err}");
    }
}
