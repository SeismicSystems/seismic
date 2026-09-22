//! Tab-completion candidates for the names an operator types most: contexts,
//! networks and nodes.
//!
//! Static completion knows flags and subcommand names; what an operator is
//! actually struggling to type is a value — `tmp-devnet-1/tmp-devnet-1-2` —
//! and every such value is already in the context file, which `ctx list`
//! reads to print the same list. So the rule here is the one `ctx list`
//! keeps: **read nothing but the config file.** No RPC, no Pulumi, no walk of
//! a network directory. A tab press must never hang on a network timeout.
//!
//! Two limits follow from how completion is invoked. A candidate function
//! runs before clap has parsed the rest of the command line, so it cannot
//! see a `--config` or `--context` typed on the same line: the file is the
//! default one, and the network `nodes` scopes to is the shell's
//! `SEISMIC_CONTEXT`, else the file's `current`. And a candidate function
//! must never fail loudly — the shell shows an error as garbage in the
//! prompt — so an unreadable file completes to nothing.

use std::collections::BTreeSet;

use clap_complete::CompletionCandidate;

use crate::config::Config;
use crate::{Context, Selection};

/// Every selection `ctx use` and `--context` accept: each network, each
/// `<network>/<node>`, and `-` for the previous selection when there is one.
pub fn selections() -> Vec<CompletionCandidate> {
    load()
        .map(|config| selections_of(&config))
        .unwrap_or_default()
}

/// The registered network names: `ctx set-nodes <NETWORK>`, `ctx set-network
/// <NAME>`.
pub fn networks() -> Vec<CompletionCandidate> {
    load()
        .map(|config| networks_of(&config))
        .unwrap_or_default()
}

/// The node names a `--name` (or `--genesis-node`, `--join`) may pick: the
/// selected network's, else every network's when nothing is selected.
pub fn nodes() -> Vec<CompletionCandidate> {
    let requested = std::env::var("SEISMIC_CONTEXT").ok();
    load()
        .map(|config| nodes_of(&config, requested.as_deref()))
        .unwrap_or_default()
}

/// The default context file, or nothing: a completion never reports.
fn load() -> Option<Config> {
    Context::load(None)
        .ok()
        .map(|context| context.config().clone())
}

fn selections_of(config: &Config) -> Vec<CompletionCandidate> {
    let current = config.current.as_deref();
    let mark = |value: &str| {
        let candidate = CompletionCandidate::new(value);
        if Some(value) == current {
            candidate.help(Some("current".into()))
        } else {
            candidate
        }
    };
    let mut candidates = Vec::new();
    for (name, network) in &config.networks {
        candidates.push(mark(name));
        for node in network.nodes.keys() {
            candidates.push(mark(&format!("{name}/{node}")));
        }
    }
    if let Some(previous) = &config.previous {
        candidates.push(
            CompletionCandidate::new("-")
                .help(Some(format!("the previous selection: {previous}").into())),
        );
    }
    candidates
}

fn networks_of(config: &Config) -> Vec<CompletionCandidate> {
    config
        .networks
        .keys()
        .map(CompletionCandidate::new)
        .collect()
}

/// `requested` is `SEISMIC_CONTEXT`; the fallback is the file's `current`.
/// A selection naming a network the file does not hold scopes to nothing,
/// so the union is offered instead — the same set a stale selection would
/// have picked from once corrected.
fn nodes_of(config: &Config, requested: Option<&str>) -> Vec<CompletionCandidate> {
    let network = requested
        .or(config.current.as_deref())
        .and_then(|selection| selection.parse::<Selection>().ok())
        .and_then(|selection| config.networks.get(&selection.network));
    let names: BTreeSet<&String> = match network {
        Some(network) => network.nodes.keys().collect(),
        None => config
            .networks
            .values()
            .flat_map(|network| network.nodes.keys())
            .collect(),
    };
    names.into_iter().map(CompletionCandidate::new).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"
current = "devnet-1/alpha"
previous = "devnet-1"

[networks.devnet-1]
dir = "/x"

[networks.devnet-1.nodes]
alpha = { public_ip = "203.0.113.7", fqdn = "alpha.example.com" }
beta = { public_ip = "203.0.113.8", fqdn = "beta.example.com" }

[networks.partner-net]
manifest = "/y/network-manifest.json"

[networks.partner-net.nodes]
my-node = { public_ip = "198.51.100.4", fqdn = "my-node.example.com" }
"#;

    fn config() -> Config {
        toml::from_str(CONFIG).unwrap()
    }

    fn values(candidates: &[CompletionCandidate]) -> Vec<String> {
        candidates
            .iter()
            .map(|c| c.get_value().to_string_lossy().into_owned())
            .collect()
    }

    /// Networks and their nodes in file order, `-` last, the current one
    /// alone carrying a `current` note.
    #[test]
    fn selections_are_every_network_and_node_plus_previous() {
        let candidates = selections_of(&config());
        assert_eq!(
            values(&candidates),
            [
                "devnet-1",
                "devnet-1/alpha",
                "devnet-1/beta",
                "partner-net",
                "partner-net/my-node",
                "-"
            ]
        );
        let helps: Vec<Option<String>> = candidates
            .iter()
            .map(|c| c.get_help().map(ToString::to_string))
            .collect();
        assert_eq!(helps[1].as_deref(), Some("current"));
        assert_eq!(helps[0], None);
        assert_eq!(
            helps[5].as_deref(),
            Some("the previous selection: devnet-1")
        );
    }

    /// No `previous` in the file, no `-` offered: `ctx use -` would refuse.
    #[test]
    fn no_previous_no_dash() {
        let mut config = config();
        config.previous = None;
        assert!(!values(&selections_of(&config)).contains(&"-".to_string()));
    }

    #[test]
    fn networks_are_the_registered_names() {
        assert_eq!(values(&networks_of(&config())), ["devnet-1", "partner-net"]);
    }

    /// The env var scopes first, then `current`; a network-only selection
    /// scopes as well as a node one does.
    #[test]
    fn nodes_scope_to_the_selected_network() {
        let config = config();
        assert_eq!(values(&nodes_of(&config, None)), ["alpha", "beta"]);
        assert_eq!(values(&nodes_of(&config, Some("partner-net"))), ["my-node"]);
        assert_eq!(
            values(&nodes_of(&config, Some("partner-net/my-node"))),
            ["my-node"]
        );
    }

    /// Nothing selected, or a selection the file cannot resolve: every node
    /// of every network, once each.
    #[test]
    fn nodes_fall_back_to_the_union() {
        let mut config = config();
        config.current = None;
        assert_eq!(
            values(&nodes_of(&config, None)),
            ["alpha", "beta", "my-node"]
        );
        assert_eq!(
            values(&nodes_of(&config, Some("gone"))),
            ["alpha", "beta", "my-node"]
        );
        assert_eq!(
            values(&nodes_of(&config, Some("not/a/selection"))),
            ["alpha", "beta", "my-node"]
        );
    }
}
