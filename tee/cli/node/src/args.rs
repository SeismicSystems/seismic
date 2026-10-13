//! How a command names its nodes. Every command reads the same cohort, the
//! operator's nodes as a node table: the descriptor map `--node <file>` names
//! (see [`seismic_tee_common::descriptor`]), else the current context's
//! network (`seismic-tee ctx use <network>[/<node>]`), so a command line that
//! names neither still has a definite target.
//!
//! - [`NodeArgs`] picks one node of the cohort: `--name`, else the node the
//!   context names, else the cohort's only node. Shared by `verify` and
//!   `status`, so an operator who learns one command's way of pointing at a
//!   node has learned both.
//! - [`CohortArgs`] takes the whole cohort, for `configure`; `harvest` spells
//!   its cohort the same way.

use std::path::{Path, PathBuf};

use clap::Args;
use clap_complete::ArgValueCandidates;
use seismic_tee_common::{Descriptors, NodeDescriptor, select_descriptor};
use seismic_tee_context::{
    Context, ContextArgs, Selection, complete, echo, load_nodes, read_nodes,
};

/// `--node <file>`, or the current context's network: the whole cohort.
#[derive(Debug, Clone, Default, Args)]
pub struct CohortArgs {
    /// Cohort's node table: `pulumi stack output nodes --json`, i.e.
    /// {<name>: {public_ip, fqdn}, …}. Omit it to use the current context's
    /// network (`seismic-tee ctx use`).
    #[arg(long, value_name = "FILE")]
    pub node: Option<PathBuf>,

    #[command(flatten)]
    pub context: ContextArgs,
}

impl CohortArgs {
    /// The cohort's node table: `--node FILE`, else the context's network,
    /// echoed.
    pub fn load(&self, config: Option<&Path>) -> anyhow::Result<Descriptors> {
        load_nodes(self.node.as_deref(), &self.context, config, "--node")
    }

    /// These flags as they were given, for a suggested follow-up command on
    /// the same cohort: `--node FILE`, else an explicit `--context`. Empty
    /// when the persisted selection supplied the table — it is still in
    /// force for the next command, so nothing needs repeating; an explicit
    /// `--context` is not persisted anywhere, so it is repeated like
    /// `--node` is. Leading space included, so it splices into a command
    /// line.
    pub fn as_flags(&self) -> String {
        if let Some(node) = &self.node {
            format!(" --node {}", node.display())
        } else if let Some(context) = &self.context.context {
            format!(" --context {context}")
        } else {
            String::new()
        }
    }
}

/// The cohort, and which one of its nodes to act on.
#[derive(Debug, Clone, Args)]
pub struct NodeArgs {
    #[command(flatten)]
    pub cohort: CohortArgs,

    /// Which node of the cohort to act on (its key). Optional when the
    /// cohort holds exactly one, or when the context names it.
    #[arg(long, value_name = "NAME", add = ArgValueCandidates::new(complete::nodes))]
    pub name: Option<String>,
}

impl NodeArgs {
    /// Resolve the flags to the one node, with its name.
    ///
    /// An explicit `--node` wins outright and the context is never read, so a
    /// command line remains a complete record of what it acted on; every
    /// failure then names the file, absent, malformed, or not singling out a
    /// node. Otherwise the context supplies the node table — `--context`,
    /// else `SEISMIC_CONTEXT`, else the file's `current` — and the resolved
    /// target is echoed on stderr before anything happens.
    ///
    /// (No clap `conflicts_with` between `--node` and `--context`: clap
    /// counts an env-sourced value as present, so a shell that exported
    /// `SEISMIC_CONTEXT` could never pass `--node`. Precedence is decided
    /// here instead.)
    pub fn load(&self, config: Option<&Path>) -> anyhow::Result<(String, NodeDescriptor)> {
        if let Some(path) = self.cohort.node.as_deref() {
            let nodes = read_nodes(path, "--node")?;
            let (name, descriptor) =
                select_descriptor(&nodes, self.name.as_deref(), &path.display())?;
            return Ok((name.to_string(), descriptor.clone()));
        }
        let context = Context::load(config)?;
        let selected = context.select(self.cohort.context.context.as_deref())?;
        // --name composes with a network-only context: the network half
        // supplies the table, the flag supplies the key.
        let (name, descriptor) = selected.node(self.name.as_deref())?;
        // Echoed pinned to the node it resolved to, so a network-only
        // selection plus --name still names the node in the echo — what the
        // command acts on is what it says it acts on.
        let selection = Selection {
            network: selected.selection.network.clone(),
            node: Some(name.to_string()),
        };
        echo(&selection, &descriptor.eth_rpc_url());
        Ok((name.to_string(), descriptor.clone()))
    }

    /// [`CohortArgs::as_flags`] plus `--name`, for a suggested follow-up
    /// command that must name the same node.
    pub fn as_flags(&self) -> String {
        let mut flags = self.cohort.as_flags();
        if let Some(name) = &self.name {
            flags.push_str(&format!(" --name {name}"));
        }
        flags
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use seismic_tee_context::ConfigArgs;

    use super::*;

    /// The flags as a command mounts them, beside the global `--config`.
    #[derive(Parser)]
    struct Probe {
        #[command(flatten)]
        node: NodeArgs,
        #[command(flatten)]
        config: ConfigArgs,
    }

    impl Probe {
        fn load(&self) -> anyhow::Result<(String, NodeDescriptor)> {
            self.node.load(self.config.config.as_deref())
        }

        fn as_flags(&self) -> String {
            self.node.as_flags()
        }
    }

    fn args(argv: &[&str]) -> Probe {
        Probe::try_parse_from(std::iter::once(&"probe").chain(argv)).expect("well-formed argv")
    }

    /// Guards a config-fallback test against an ambient `SEISMIC_CONTEXT`:
    /// `make -C tee/cli check` runs `cargo nextest run`, which gives each
    /// test its own process, so clearing it here does not race any other
    /// test.
    fn clear_context_env() {
        // SAFETY: this process runs this one test; nextest does not share a
        // process across tests.
        unsafe { std::env::remove_var("SEISMIC_CONTEXT") };
    }

    const TWO: &str = r#"{
        "node-1": {"public_ip": "203.0.113.7", "fqdn": "node1.example.com"},
        "node-2": {"public_ip": "203.0.113.99", "fqdn": "other.example"}
    }"#;

    const TWO_NODE_CONFIG: &str = r#"
current = "devnet-1/alpha"

[networks.devnet-1]
dir = "/x"

[networks.devnet-1.nodes]
alpha = { public_ip = "203.0.113.7", fqdn = "alpha.example.com" }
beta = { public_ip = "203.0.113.8", fqdn = "beta.example.com" }
"#;

    const NETWORK_ONLY_CONFIG: &str = r#"
current = "devnet-1"

[networks.devnet-1]
dir = "/x"

[networks.devnet-1.nodes]
alpha = { public_ip = "203.0.113.7", fqdn = "alpha.example.com" }
beta = { public_ip = "203.0.113.8", fqdn = "beta.example.com" }
"#;

    #[test]
    fn name_picks_the_node_out_of_a_cohort_map() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nodes.json");
        std::fs::write(&path, TWO).unwrap();

        let (name, descriptor) = args(&["--node", path.to_str().unwrap(), "--name", "node-2"])
            .load()
            .unwrap();
        assert_eq!(name, "node-2");
        assert_eq!(descriptor.fqdn, "other.example");
        assert_eq!(descriptor.public_ip, "203.0.113.99");
    }

    #[test]
    fn a_cohort_map_without_name_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nodes.json");
        std::fs::write(&path, TWO).unwrap();

        let err = args(&["--node", path.to_str().unwrap()])
            .load()
            .unwrap_err()
            .to_string();
        assert!(err.contains("--name"), "{err}");
    }

    #[test]
    fn a_missing_file_is_named_by_flag() {
        let err = args(&["--node", "/absent/nodes.json"])
            .load()
            .unwrap_err()
            .to_string();
        assert!(err.contains("--node descriptor file not found"), "{err}");
        assert!(err.contains("/absent/nodes.json"), "{err}");
    }

    /// An explicit `--node` wins outright: the context is never consulted,
    /// even one pointed at by `--config`.
    #[test]
    fn an_explicit_node_wins_over_the_context() {
        clear_context_env();
        let dir = tempfile::tempdir().unwrap();
        let nodes_path = dir.path().join("nodes.json");
        std::fs::write(&nodes_path, TWO).unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(&config_path, TWO_NODE_CONFIG).unwrap();

        let (name, descriptor) = args(&[
            "--node",
            nodes_path.to_str().unwrap(),
            "--name",
            "node-1",
            "--config",
            config_path.to_str().unwrap(),
        ])
        .load()
        .unwrap();
        assert_eq!(name, "node-1");
        assert_eq!(descriptor.fqdn, "node1.example.com");
    }

    /// Neither flag: the context's `current` supplies the node.
    #[test]
    fn neither_flag_falls_back_to_the_selected_context() {
        clear_context_env();
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(&config_path, TWO_NODE_CONFIG).unwrap();

        let (name, descriptor) = args(&["--config", config_path.to_str().unwrap()])
            .load()
            .unwrap();
        assert_eq!(name, "alpha");
        assert_eq!(descriptor.fqdn, "alpha.example.com");
    }

    /// `--name` composes with a network-only context: the network supplies
    /// the table, `--name` supplies the key.
    #[test]
    fn name_composes_with_a_network_only_context() {
        clear_context_env();
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(&config_path, NETWORK_ONLY_CONFIG).unwrap();

        let (name, descriptor) =
            args(&["--config", config_path.to_str().unwrap(), "--name", "beta"])
                .load()
                .unwrap();
        assert_eq!(name, "beta");
        assert_eq!(descriptor.fqdn, "beta.example.com");
    }

    /// Neither flag and no context selected anywhere: the error names both
    /// escapes — pass `--node`, or run `ctx use`.
    #[test]
    fn neither_flag_and_no_context_names_both_escapes() {
        clear_context_env();
        let dir = tempfile::tempdir().unwrap();
        // A config path that names no file: Context::load treats an absent
        // file as an empty config, with no `current`.
        let config_path = dir.path().join("config.toml");

        let err = args(&["--config", config_path.to_str().unwrap()])
            .load()
            .unwrap_err()
            .to_string();
        assert!(err.contains("--node"), "{err}");
        assert!(err.contains("ctx use"), "{err}");
    }

    #[test]
    fn the_flags_splice_into_a_suggested_command() {
        clear_context_env();
        assert_eq!(args(&["--node", "n.json"]).as_flags(), " --node n.json");
        assert_eq!(
            args(&["--node", "n.json", "--name", "dev-2"]).as_flags(),
            " --node n.json --name dev-2"
        );
        assert_eq!(
            args(&["--context", "devnet-1/alpha"]).as_flags(),
            " --context devnet-1/alpha"
        );
        assert_eq!(args(&[]).as_flags(), "");
    }
}
