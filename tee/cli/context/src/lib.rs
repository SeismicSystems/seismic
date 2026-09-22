//! The context file: which network — and, optionally, which of its nodes —
//! the deploy CLI's commands act on.
//!
//! A context is a network plus an optional default node, addressed as
//! `<network>` or `<network>/<node>`. There is no `[contexts]` table because
//! there is nothing to name beyond that: the node is a key inside the
//! network's own node table, so `<network>/<node>` is already the context's
//! whole name.
//!
//! A network entry holds pointers to its artifact set — a directory, a
//! manifest, a `network_id` pin — and its cohort's node table, inline under
//! `[networks.<name>.nodes]`. That is the kubeconfig shape: a kubeconfig
//! stores each cluster's endpoint inline rather than pointing at a file
//! elsewhere, a provisioner writes the entry (`aws eks update-kubeconfig`,
//! `kind create cluster`), and the CLI only reads it. Here the provisioner is
//! Pulumi, `ctx set-nodes` is the write, and the file never holds a
//! credential or a policy default — everything in it is *where to look*,
//! nothing is *what to do* once a command gets there.
//!
//! This crate depends on `seismic-tee-common` alone, so every command group —
//! operator or founder side — may read it.

pub mod args;
pub mod cmd;
pub mod complete;
pub mod config;
pub mod env;
pub mod exec;
pub mod path;
pub mod write;

use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use seismic_tee_common::{
    Descriptors, NetworkDir, NodeDescriptor, load_descriptors, select_descriptor,
};

pub use args::ContextArgs;
use config::{Config, Network, Shape};

/// A selection: a network, and optionally one of its nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub network: String,
    pub node: Option<String>,
}

impl std::str::FromStr for Selection {
    type Err = anyhow::Error;

    /// Splits on the one `/`. `a/b/c`, `a/` and `/b` are all rejected: a
    /// selection is `<network>` or `<network>/<node>`, never fewer or more.
    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s.split_once('/') {
            None if s.is_empty() => bail!("a context selection cannot be empty"),
            None => Ok(Self {
                network: s.to_string(),
                node: None,
            }),
            Some((network, node))
                if network.is_empty() || node.is_empty() || node.contains('/') =>
            {
                bail!(
                    "`{s}` is not `<network>` or `<network>/<node>` — it must have exactly one \
                     `/`, with a name on each side"
                )
            }
            Some((network, node)) => Ok(Self {
                network: network.to_string(),
                node: Some(node.to_string()),
            }),
        }
    }
}

impl std::fmt::Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.node {
            Some(node) => write!(f, "{}/{node}", self.network),
            None => write!(f, "{}", self.network),
        }
    }
}

/// The context file, loaded.
#[derive(Debug)]
pub struct Context {
    path: PathBuf,
    config: Config,
}

impl Context {
    /// Read the file, or an empty config when it does not exist.
    pub fn load(explicit: Option<&Path>) -> anyhow::Result<Self> {
        let path = match explicit {
            Some(path) => path.to_path_buf(),
            None => path::default_path()?,
        };
        let config = if path.is_file() {
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            let config: Config = toml::from_str(&text)
                .with_context(|| format!("{} is not a valid context file", path.display()))?;
            config.validate(&path)?;
            config
        } else {
            Config::default()
        };
        Ok(Self { path, config })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// The selection this invocation acts on: `requested` (from `--context`
    /// or `SEISMIC_CONTEXT`), else the file's `current`.
    pub fn select(&self, requested: Option<&str>) -> anyhow::Result<Selected<'_>> {
        let selection: Selection = match requested {
            Some(s) => s.parse()?,
            None => match &self.config.current {
                Some(current) => current.parse()?,
                None => bail!(
                    "no context selected — pass --node, or run `seismic-tee ctx use \
                     <network>/<node>`"
                ),
            },
        };
        let network = self
            .config
            .networks
            .get(&selection.network)
            .ok_or_else(|| {
                let names = self
                    .config
                    .networks
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(", ");
                anyhow::anyhow!(
                    "no network `{}` in {}; it holds {names}",
                    selection.network,
                    self.path.display(),
                )
            })?;
        Ok(Selected {
            selection,
            network,
            config_path: &self.path,
        })
    }
}

/// A selection resolved against the file: the network entry it names, with
/// the node half carried alongside.
#[derive(Debug)]
pub struct Selected<'a> {
    pub selection: Selection,
    pub network: &'a Network,
    config_path: &'a Path,
}

impl Selected<'_> {
    /// The cohort's node table, imported by `ctx set-nodes`. An error naming
    /// `ctx set-nodes` when the table is empty — a network registered with
    /// only a `dir` (or nothing provisioned yet) has nothing here.
    pub fn nodes(&self) -> anyhow::Result<&Descriptors> {
        if self.network.nodes.is_empty() {
            bail!(
                "network `{}` has no nodes — import the cohort's map:\n  pulumi stack output \
                 nodes --json | seismic-tee ctx set-nodes {}\n  (a bring-your-own-infra operator \
                 pipes the same shape in by hand: {{<name>: {{public_ip, fqdn}}, …}})",
                self.selection.network,
                self.selection.network,
            );
        }
        Ok(&self.network.nodes)
    }

    /// The one node this selection acts on, through
    /// [`seismic_tee_common::select_descriptor`]: `name` when given, else the
    /// selection's node half, else the table's only entry.
    pub fn node(&self, name: Option<&str>) -> anyhow::Result<(&str, &NodeDescriptor)> {
        let nodes = self.nodes()?;
        let name = name.or(self.selection.node.as_deref());
        let holder = format!(
            "network `{}` in {}",
            self.selection.network,
            self.config_path.display(),
        );
        Ok(select_descriptor(nodes, name, &holder)?)
    }

    /// `<dir>/network-manifest.json`, or the `manifest` path.
    pub fn manifest(&self) -> anyhow::Result<PathBuf> {
        match self.network.shape() {
            Shape::Dir(dir) => Ok(NetworkDir::new(path::expand_tilde(dir)?).manifest()),
            Shape::Loose {
                manifest: Some(manifest),
            } => path::expand_tilde(manifest),
            Shape::Loose { manifest: None } => bail!(
                "network `{}` has no manifest — set `manifest` or `dir` in {}, or pass --manifest",
                self.selection.network,
                self.config_path.display(),
            ),
            Shape::Published { .. } => Err(self.published_error()),
        }
    }

    /// The network directory, for the founder commands' positional DIR.
    pub fn dir(&self) -> anyhow::Result<PathBuf> {
        match self.network.shape() {
            Shape::Dir(dir) => path::expand_tilde(dir),
            Shape::Loose { .. } => bail!(
                "network `{}` has no dir — it is registered as loose files; pass DIR",
                self.selection.network
            ),
            Shape::Published { .. } => Err(self.published_error()),
        }
    }

    /// The pinned network_id, when the entry carries one.
    pub fn network_id(&self) -> Option<&str> {
        self.network.network_id.as_deref()
    }

    fn published_error(&self) -> anyhow::Error {
        anyhow::anyhow!(
            "network `{}` is a published artifact set, and fetching is not implemented; \
             download it and set `dir` in {}",
            self.selection.network,
            self.config_path.display(),
        )
    }
}

/// The style of the context narration: dimmed, so what the file resolved to
/// reads as background beside the command's own report rather than as part
/// of it. Dimmed rather than a fixed grey because it follows the terminal's
/// foreground, so it stays legible on light and dark themes alike. Rendered
/// only when stderr is a terminal that wants colour (see `anstream` in the
/// workspace manifest).
pub const NOTE: anstyle::Style = anstyle::Style::new().dimmed();

/// A line of context narration on stderr, in [`NOTE`] style.
pub fn note(line: &dyn std::fmt::Display) {
    anstream::eprintln!("{NOTE}{line}{NOTE:#}");
}

/// What a context-resolved command is acting on, on stderr — never stdout,
/// so `eval "$(seismic-tee ctx env)"` stays evaluable.
pub fn echo(selection: &Selection, resolved: &dyn std::fmt::Display) {
    note(&format_args!("context {selection} → {resolved}"));
}

/// A cohort's node table: `flag` when given, else the selected network's.
///
/// The shared resolution behind every founder command that needs a whole
/// cohort rather than one node — `harvest`, `network configure` — each with
/// its own escape-hatch flag (`--nodes FILE`, the `pulumi stack output nodes
/// --json` shape); `flag_name` is spelled into the "no context selected"
/// error, naming the one flag this particular caller actually has.
pub fn load_nodes(
    flag: Option<&Path>,
    args: &ContextArgs,
    flag_name: &str,
) -> anyhow::Result<Descriptors> {
    if let Some(path) = flag {
        if !path.is_file() {
            bail!("{flag_name} descriptor file not found: {}", path.display());
        }
        return Ok(load_descriptors(path)?);
    }
    let context = Context::load(args.config.as_deref())?;
    if args.context.is_none() && context.config().current.is_none() {
        bail!(
            "no context selected — pass {flag_name}, or run `seismic-tee ctx use \
             <network>/<node>`"
        );
    }
    let selected = context.select(args.context.as_deref())?;
    let nodes = selected.nodes()?;
    echo(&selected.selection, &format!("{} node(s)", nodes.len()));
    Ok(nodes.clone())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn config_with(networks: BTreeMap<String, Network>) -> Context {
        Context {
            path: PathBuf::from("/config.toml"),
            config: Config {
                current: None,
                previous: None,
                networks,
            },
        }
    }

    fn descriptor(public_ip: &str, fqdn: &str) -> NodeDescriptor {
        NodeDescriptor {
            public_ip: public_ip.to_string(),
            fqdn: fqdn.to_string(),
        }
    }

    fn two_node_network() -> Network {
        let mut nodes = Descriptors::new();
        nodes.insert(
            "alpha".to_string(),
            descriptor("203.0.113.7", "alpha.example.com"),
        );
        nodes.insert(
            "beta".to_string(),
            descriptor("203.0.113.8", "beta.example.com"),
        );
        Network {
            dir: Some(PathBuf::from("/x")),
            nodes,
            ..Default::default()
        }
    }

    #[test]
    fn selection_round_trips_both_forms() {
        let network_only: Selection = "devnet-1".parse().unwrap();
        assert_eq!(network_only.to_string(), "devnet-1");

        let with_node: Selection = "devnet-1/alpha".parse().unwrap();
        assert_eq!(with_node.to_string(), "devnet-1/alpha");
        assert_eq!(with_node.network, "devnet-1");
        assert_eq!(with_node.node.as_deref(), Some("alpha"));
    }

    #[test]
    fn malformed_selections_are_rejected() {
        for bad in ["a/b/c", "a/", "/b", ""] {
            assert!(bad.parse::<Selection>().is_err(), "{bad}");
        }
    }

    #[test]
    fn select_with_an_unknown_name_lists_the_names() {
        let mut networks = BTreeMap::new();
        networks.insert(
            "devnet-1".to_string(),
            Network {
                dir: Some(PathBuf::from("/x")),
                ..Default::default()
            },
        );
        networks.insert(
            "partner-net".to_string(),
            Network {
                dir: Some(PathBuf::from("/y")),
                ..Default::default()
            },
        );
        let context = config_with(networks);

        let err = context.select(Some("no-such-net")).unwrap_err().to_string();
        assert!(err.contains("no network `no-such-net`"), "{err}");
        assert!(err.contains("devnet-1"), "{err}");
        assert!(err.contains("partner-net"), "{err}");
    }

    #[test]
    fn nodes_on_an_empty_table_names_ctx_set_nodes() {
        let network = Network {
            dir: Some(PathBuf::from("/x")),
            ..Default::default()
        };
        let config_path = PathBuf::from("/config.toml");
        let selected = Selected {
            selection: Selection {
                network: "devnet-1".to_string(),
                node: None,
            },
            network: &network,
            config_path: &config_path,
        };

        let err = selected.nodes().unwrap_err().to_string();
        assert!(err.contains("ctx set-nodes devnet-1"), "{err}");
    }

    #[test]
    fn node_none_on_a_two_node_table_asks_for_name_naming_the_holder() {
        let network = two_node_network();
        let config_path = PathBuf::from("/config.toml");
        let selected = Selected {
            selection: Selection {
                network: "devnet-1".to_string(),
                node: None,
            },
            network: &network,
            config_path: &config_path,
        };

        let err = selected.node(None).unwrap_err().to_string();
        assert!(err.contains("network `devnet-1` in /config.toml"), "{err}");
        assert!(err.contains("pass --name"), "{err}");
    }

    #[test]
    fn node_some_composes_with_a_network_only_selection() {
        let network = two_node_network();
        let config_path = PathBuf::from("/config.toml");
        let selected = Selected {
            selection: Selection {
                network: "devnet-1".to_string(),
                node: None,
            },
            network: &network,
            config_path: &config_path,
        };

        let (name, descriptor) = selected.node(Some("beta")).unwrap();
        assert_eq!(name, "beta");
        assert_eq!(descriptor.fqdn, "beta.example.com");
    }

    #[test]
    fn node_falls_back_to_the_selections_own_node_half() {
        let network = two_node_network();
        let config_path = PathBuf::from("/config.toml");
        let selected = Selected {
            selection: Selection {
                network: "devnet-1".to_string(),
                node: Some("alpha".to_string()),
            },
            network: &network,
            config_path: &config_path,
        };

        let (name, _) = selected.node(None).unwrap();
        assert_eq!(name, "alpha");
    }

    #[test]
    fn manifest_and_dir_each_refuse_a_published_network() {
        // `nodes` is orthogonal to shape — a published network importing its
        // cohort ahead of a fetch is legal — so only `manifest` and `dir`
        // are shape-gated here.
        let network = Network {
            source: Some("https://example.com/bundle".to_string()),
            network_id: Some("a".repeat(64)),
            ..Default::default()
        };
        let config_path = PathBuf::from("/config.toml");
        let selected = Selected {
            selection: Selection {
                network: "some-fork".to_string(),
                node: None,
            },
            network: &network,
            config_path: &config_path,
        };

        for err in [
            selected.manifest().unwrap_err().to_string(),
            selected.dir().unwrap_err().to_string(),
        ] {
            assert!(err.contains("is a published artifact set"), "{err}");
            assert!(err.contains("fetching is not implemented"), "{err}");
        }
    }

    #[test]
    fn manifest_names_the_flag_for_a_loose_network_with_neither_dir_nor_manifest() {
        let network = two_node_network();
        let mut network = network;
        network.dir = None;
        let config_path = PathBuf::from("/config.toml");
        let selected = Selected {
            selection: Selection {
                network: "partner-net".to_string(),
                node: None,
            },
            network: &network,
            config_path: &config_path,
        };

        let err = selected.manifest().unwrap_err().to_string();
        assert!(err.contains("has no manifest"), "{err}");
        assert!(err.contains("--manifest"), "{err}");
    }

    const TWO_NODE_CONFIG: &str = r#"
current = "devnet-1"

[networks.devnet-1]
dir = "/x"

[networks.devnet-1.nodes]
alpha = { public_ip = "203.0.113.7", fqdn = "alpha.example.com" }
beta = { public_ip = "203.0.113.8", fqdn = "beta.example.com" }
"#;

    #[test]
    fn load_nodes_the_flag_wins_and_the_context_is_not_read() {
        let dir = tempfile::tempdir().unwrap();
        let nodes_path = dir.path().join("nodes.json");
        std::fs::write(
            &nodes_path,
            r#"{"solo": {"public_ip": "203.0.113.9", "fqdn": "solo.example.com"}}"#,
        )
        .unwrap();
        // A config path that would resolve a different cohort if read.
        let config_path = dir.path().join("config.toml");
        std::fs::write(&config_path, TWO_NODE_CONFIG).unwrap();

        let args = ContextArgs {
            context: None,
            config: Some(config_path),
        };
        let nodes = load_nodes(Some(&nodes_path), &args, "--nodes").unwrap();
        assert_eq!(nodes.keys().collect::<Vec<_>>(), ["solo"]);
    }

    #[test]
    fn load_nodes_with_no_flag_reads_the_selected_networks_table() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(&config_path, TWO_NODE_CONFIG).unwrap();

        let args = ContextArgs {
            context: None,
            config: Some(config_path),
        };
        let nodes = load_nodes(None, &args, "--nodes").unwrap();
        assert_eq!(nodes.keys().collect::<Vec<_>>(), ["alpha", "beta"]);
    }

    #[test]
    fn load_nodes_on_an_empty_table_names_ctx_set_nodes() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            "current = \"devnet-1\"\n\n[networks.devnet-1]\ndir = \"/x\"\n",
        )
        .unwrap();

        let args = ContextArgs {
            context: None,
            config: Some(config_path),
        };
        let err = load_nodes(None, &args, "--nodes").unwrap_err().to_string();
        assert!(err.contains("ctx set-nodes devnet-1"), "{err}");
    }

    #[test]
    fn load_nodes_with_no_context_names_the_flag_it_was_given() {
        let dir = tempfile::tempdir().unwrap();
        // A config path that names no file: an empty config, no `current`.
        let config_path = dir.path().join("config.toml");

        let args = ContextArgs {
            context: None,
            config: Some(config_path),
        };
        let err = load_nodes(None, &args, "--nodes").unwrap_err().to_string();
        assert!(err.contains("--nodes"), "{err}");
        assert!(err.contains("ctx use"), "{err}");
    }
}
