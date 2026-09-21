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
    /// A network directory: the committed artifact set — manifest, genesis,
    /// policy, harvest records. Never the cohort; that is `nodes`.
    pub dir: Option<PathBuf>,
    /// The network manifest on its own, for one handed over as a loose file.
    pub manifest: Option<PathBuf>,
    /// A published artifact set. Fetching is not built yet.
    pub source: Option<String>,
    /// SHA-256 of network-manifest.json: the pin a fetched or cached
    /// artifact set is refused unless it matches.
    pub network_id: Option<String>,
    /// The cohort: node name → address, exactly as the provisioner reported
    /// it. Written by `ctx set-nodes` from `pulumi stack output nodes --json`.
    #[serde(default)]
    pub nodes: Descriptors,
}

/// Where a network's artifact set is, decided once at load. The cohort is
/// orthogonal: any shape may carry `nodes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape<'a> {
    Dir(&'a Path),
    /// A loose manifest, or nothing but nodes (enough for `status`, `env`,
    /// `exec`; not for anything that needs the manifest).
    Loose {
        manifest: Option<&'a Path>,
    },
    Published {
        source: &'a str,
        network_id: &'a str,
    },
}

impl Network {
    /// A network entry naming a directory, with no manifest, source or
    /// nodes — what `network init` registers for the directory it just
    /// scaffolded.
    pub fn of_dir(dir: &Path) -> Self {
        Self {
            dir: Some(dir.to_path_buf()),
            ..Default::default()
        }
    }

    /// Validate this entry's shape, naming `name` and `config_path` in every
    /// failure.
    pub fn validate(&self, name: &str, config_path: &Path) -> anyhow::Result<()> {
        let has_dir = self.dir.is_some();
        let has_manifest = self.manifest.is_some();
        let has_source = self.source.is_some();
        let has_nodes = !self.nodes.is_empty();

        if !has_dir && !has_manifest && !has_source && !has_nodes {
            bail!(
                "network `{name}` in {} is empty — set dir, manifest or source, or import its \
                 nodes with `seismic-tee ctx set-nodes {name}`",
                config_path.display(),
            );
        }
        if has_dir && has_manifest {
            bail!(
                "network `{name}` in {} sets both dir and manifest — a dir already names it",
                config_path.display(),
            );
        }
        if has_source && self.network_id.is_none() {
            bail!(
                "network `{name}` in {} has a source but no network_id — a source with no pin \
                 is not a weaker check, it is no check",
                config_path.display(),
            );
        }
        if let Some(id) = &self.network_id
            && !(id.len() == 64 && id.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            bail!(
                "network `{name}` in {}: network_id must be 32 bytes of hex, got {id}",
                config_path.display(),
            );
        }
        Ok(())
    }

    /// Which of the three shapes this entry is.
    ///
    /// Only meaningful once [`Self::validate`] has passed: an entry with a
    /// `source` and no `network_id` would panic in the `Published` arm below,
    /// but `validate` refuses to let one reach here.
    pub fn shape(&self) -> Shape<'_> {
        if let Some(dir) = &self.dir {
            return Shape::Dir(dir);
        }
        if let Some(source) = &self.source {
            return Shape::Published {
                source,
                network_id: self.network_id.as_deref().expect("validated"),
            };
        }
        Shape::Loose {
            manifest: self.manifest.as_deref(),
        }
    }
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

    const HEX32: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn network(toml: &str) -> Network {
        toml::from_str::<Network>(toml).unwrap()
    }

    fn one_node() -> &'static str {
        r#"[nodes.alpha]
public_ip = "203.0.113.7"
fqdn = "alpha.example.com""#
    }

    #[test]
    fn the_three_shapes_parse_with_and_without_a_nodes_table() {
        for nodes in ["", one_node()] {
            let dir = network(&format!("dir = \"/x\"\n{nodes}"));
            assert!(matches!(dir.shape(), Shape::Dir(p) if p == Path::new("/x")));
            assert_eq!(!dir.nodes.is_empty(), !nodes.is_empty());

            let loose = network(&format!("manifest = \"/y/network-manifest.json\"\n{nodes}"));
            assert!(matches!(
                loose.shape(),
                Shape::Loose { manifest: Some(p) } if p == Path::new("/y/network-manifest.json")
            ));

            let published = network(&format!(
                "source = \"https://example.com/bundle\"\nnetwork_id = \"{HEX32}\"\n{nodes}"
            ));
            assert!(matches!(
                published.shape(),
                Shape::Published { source, network_id }
                    if source == "https://example.com/bundle" && network_id == HEX32
            ));
        }
    }

    #[test]
    fn of_dir_names_a_dir_shaped_entry() {
        let n = Network::of_dir(Path::new("/nets/devnet-1"));
        n.validate("devnet-1", Path::new("config.toml")).unwrap();
        assert!(matches!(n.shape(), Shape::Dir(p) if p == Path::new("/nets/devnet-1")));
        assert!(n.nodes.is_empty());
    }

    #[test]
    fn a_nodes_only_entry_is_legal() {
        let n = network(one_node());
        n.validate("y", Path::new("config.toml")).unwrap();
        assert_eq!(n.nodes.len(), 1);
        assert!(matches!(n.shape(), Shape::Loose { manifest: None }));
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
    fn the_four_validation_failures_produce_their_message() {
        let path = Path::new("config.toml");

        let err = Network::default()
            .validate("empty", path)
            .unwrap_err()
            .to_string();
        assert!(err.contains("is empty"), "{err}");
        assert!(err.contains("ctx set-nodes empty"), "{err}");

        let err = network(r#"source = "https://example.com/bundle""#)
            .validate("no-pin", path)
            .unwrap_err()
            .to_string();
        assert!(err.contains("has a source but no network_id"), "{err}");

        let err = network(
            r#"dir = "/x"
manifest = "/x/network-manifest.json""#,
        )
        .validate("both", path)
        .unwrap_err()
        .to_string();
        assert!(err.contains("sets both dir and manifest"), "{err}");

        let err = network(
            r#"source = "https://example.com/bundle"
network_id = "not-hex""#,
        )
        .validate("bad-id", path)
        .unwrap_err()
        .to_string();
        assert!(err.contains("network_id must be 32 bytes of hex"), "{err}");
    }
}
