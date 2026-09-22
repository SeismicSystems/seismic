//! `ctx`: name networks, and select which one — and which of its nodes — the
//! other commands act on.
//!
//! Eight verbs. [`CtxCommand::Use`] selects a context and refuses one that
//! points at nothing; [`CtxCommand::List`] reads the file back, marking the
//! selection, and [`CtxCommand::View`] prints it as it is on disk;
//! [`CtxCommand::Env`] and [`CtxCommand::Exec`] hand the selection
//! to a shell or a child process (their own modules, [`crate::env`] and
//! [`crate::exec`]); [`CtxCommand::SetNetwork`] registers or updates a network's
//! pointers; [`CtxCommand::SetNodes`] imports a network's cohort from stdin,
//! the way `aws eks update-kubeconfig` merges a cluster the cloud reported;
//! [`CtxCommand::Unset`] clears the selection. Nothing outside this module
//! writes the file.

use std::io::{IsTerminal as _, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context as _, bail};
use clap::{Args, Subcommand};
use clap_complete::ArgValueCandidates;
use seismic_tee_common::descriptor::parse_descriptors;
use seismic_tee_common::{NetworkDir, load_descriptors, next_step};

use crate::complete;
use crate::config::{Config, Network, Shape};
use crate::env::{self, EnvArgs};
use crate::exec::{self, ExecArgs};
use crate::{Context, ContextArgs, Selected, Selection, note, path, write};

/// The `ctx` command group: name networks, and select which one — and which
/// of its nodes — the commands act on.
#[derive(Debug, Subcommand)]
pub enum CtxCommand {
    /// Select the network, and node, the other commands act on:
    /// <network>, <network>/<node>, or `-` for the previous.
    Use(UseArgs),
    /// List every network and its nodes, marking the current selection — or,
    /// with --names, one network's bare node names for a shell loop.
    List(ListArgs),
    /// Print the context file as it is on disk, and its path on stderr.
    View(ConfigArgs),
    /// Print export lines for the selected node: eval "$(seismic-tee ctx env)".
    Env(EnvArgs),
    /// Run a command with the selected node's ETH_RPC_URL set.
    Exec(ExecArgs),
    /// Register a network by name, or update it: where its artifact set is
    /// (--dir, or --manifest, or --source with --network-id), and optionally
    /// its nodes from a file. Nodes can also be set separately, in either
    /// order.
    SetNetwork(SetNetworkArgs),
    /// Store a network's nodes from stdin, as the provisioner prints them:
    /// pulumi stack output nodes --json | seismic-tee ctx set-nodes <NETWORK>.
    /// Creates the network when it is not registered yet.
    SetNodes(SetNodesArgs),
    /// Clear the current selection.
    Unset(ConfigArgs),
}

/// Run one `ctx` command.
pub fn run(command: CtxCommand) -> anyhow::Result<ExitCode> {
    match command {
        CtxCommand::Use(args) => run_use(args),
        CtxCommand::List(args) => run_list(args),
        CtxCommand::View(args) => run_view(args),
        CtxCommand::Env(args) => env::run(args),
        CtxCommand::Exec(args) => exec::run(args),
        CtxCommand::SetNetwork(args) => run_set_network(args),
        CtxCommand::SetNodes(args) => run_set_nodes(args),
        CtxCommand::Unset(args) => run_unset(args),
    }
}

#[derive(Debug, Args)]
pub struct UseArgs {
    /// <network>, <network>/<node>, or `-` for the previous selection. A bare
    /// name with no `/` is a registered network when one is named that; else
    /// a node when exactly one network is registered; else a network.
    #[arg(value_name = "CONTEXT", add = ArgValueCandidates::new(complete::selections))]
    pub selection: Option<String>,

    /// Context file to write. Default: $XDG_CONFIG_HOME/seismic/config.toml,
    /// else ~/.config/seismic/config.toml.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

/// `--config` alone: what `view` and `unset` need.
#[derive(Debug, Clone, Default, Args)]
pub struct ConfigArgs {
    /// Context file to read. Default: $XDG_CONFIG_HOME/seismic/config.toml,
    /// else ~/.config/seismic/config.toml.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Args)]
#[command(after_help = "Examples:\n  \
    seismic-tee ctx list                 every network and its nodes, the selection starred\n  \
    seismic-tee ctx list devnet-1        one network's group\n  \
    seismic-tee ctx list --names         the selected network's node names, one per line\n  \
    for n in $(seismic-tee ctx list --names); do\n      \
        seismic-tee ctx exec --name \"$n\" -- scast block-number\n  \
    done")]
pub struct ListArgs {
    /// List this network alone. Default: every network — or, with --names,
    /// the selected one.
    #[arg(value_name = "NETWORK", add = ArgValueCandidates::new(complete::networks))]
    pub network: Option<String>,

    /// One bare node name per line and nothing else on stdout, for a shell
    /// loop (see the example below).
    #[arg(long)]
    pub names: bool,

    #[command(flatten)]
    pub context: ContextArgs,
}

#[derive(Debug, Args)]
pub struct SetNetworkArgs {
    /// The name to register the network under.
    #[arg(value_name = "NAME", add = ArgValueCandidates::new(complete::networks))]
    pub name: String,

    /// A network directory (from `network init`): the committed artifact
    /// set — manifest, genesis, policy, harvest records.
    #[arg(long, value_name = "DIR")]
    pub dir: Option<PathBuf>,
    /// The network manifest, for a network handed over as loose files.
    #[arg(long, value_name = "FILE")]
    pub manifest: Option<PathBuf>,
    /// A published artifact set's source. Fetching is not implemented yet;
    /// pairs with --network-id.
    #[arg(long, value_name = "URL")]
    pub source: Option<String>,
    /// SHA-256 of the network's manifest, pinned so a fetched or cached
    /// artifact set is refused unless it matches.
    #[arg(long, value_name = "ID")]
    pub network_id: Option<String>,
    /// The nodes too, from a descriptor-map file (the `pulumi stack output
    /// nodes --json` shape), for a network handed over as files. Otherwise
    /// `ctx set-nodes` stores them from stdin.
    #[arg(long, value_name = "FILE")]
    pub nodes: Option<PathBuf>,

    /// Context file to write. Default: $XDG_CONFIG_HOME/seismic/config.toml,
    /// else ~/.config/seismic/config.toml.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct SetNodesArgs {
    /// The network to import the cohort into. Created, with nodes only,
    /// when unregistered.
    #[arg(value_name = "NETWORK", add = ArgValueCandidates::new(complete::networks))]
    pub name: String,

    /// Context file to write. Default: $XDG_CONFIG_HOME/seismic/config.toml,
    /// else ~/.config/seismic/config.toml.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

fn run_use(args: UseArgs) -> anyhow::Result<ExitCode> {
    let raw = args.selection.ok_or_else(|| {
        anyhow::anyhow!(
            "ctx use needs a target: <network>, <network>/<node>, or `-` for the previous \
             selection — `seismic-tee ctx list` shows what is registered"
        )
    })?;

    let context = Context::load(args.config.as_deref())?;
    let target = resolve_target(&raw, &context)?;
    let selected = context.select(Some(&target.to_string()))?;

    // A network-only selection is legal with no cohort imported: a founder
    // who has not provisioned yet has no node to name. A node half is
    // validated against the table so a context that points at nothing is not
    // storable.
    if selected.selection.node.is_some() {
        selected.node(None)?;
    }

    write::set_current(context.path(), &selected.selection)?;

    println!("Selected {}.", selected.selection);
    print_resolution(&selected);
    Ok(ExitCode::SUCCESS)
}

/// The target `ctx use` resolves to, before it is validated against the
/// network's node table.
fn resolve_target(raw: &str, context: &Context) -> anyhow::Result<Selection> {
    if raw == "-" {
        let previous =
            context.config().previous.clone().ok_or_else(|| {
                anyhow::anyhow!("ctx use -: no previous selection to switch back to")
            })?;
        return previous.parse();
    }
    // A bare word names a network when one is registered under it. Otherwise,
    // with exactly one network registered, there is only one table it could
    // be a node of, so it is read as a node.
    if !raw.contains('/')
        && !context.config().networks.contains_key(raw)
        && context.config().networks.len() == 1
    {
        let network = context
            .config()
            .networks
            .keys()
            .next()
            .expect("len == 1")
            .clone();
        return Ok(Selection {
            network,
            node: Some(raw.to_string()),
        });
    }
    raw.parse()
}

/// What `ctx use` prints about the selection it just stored: one row per
/// thing that resolved. The state changed, so the command says what to —
/// reading it back later is `ctx list`'s marker.
fn print_resolution(selected: &Selected<'_>) {
    if let Ok(dir) = selected.dir() {
        println!("  dir       {}", dir.display());
    }
    if let Ok(manifest) = selected.manifest() {
        println!("  manifest  {}", manifest.display());
    }
    if selected.selection.node.is_some()
        && let Ok((_, descriptor)) = selected.node(None)
    {
        println!("  rpc       {}", descriptor.eth_rpc_url());
    }
}

/// The file's bytes, unparsed — a file the loader refuses is the one an
/// operator most wants to look at — with the path on stderr, so
/// `seismic-tee ctx view > backup.toml` is the file and nothing else.
fn run_view(args: ConfigArgs) -> anyhow::Result<ExitCode> {
    let path = match args.config {
        Some(path) => path,
        None => path::default_path()?,
    };
    if !path.is_file() {
        bail!(
            "no context file at {} — `seismic-tee network init` or `seismic-tee ctx set-nodes` \
             creates one",
            path.display()
        );
    }
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    note(&path.display());
    print!("{text}");
    Ok(ExitCode::SUCCESS)
}

fn run_list(args: ListArgs) -> anyhow::Result<ExitCode> {
    let context = Context::load(args.context.config.as_deref())?;
    let scope = list_scope(&context, &args)?;
    let lines = match &scope {
        Some(selected) if args.names => name_lines(selected)?,
        _ => {
            if context.config().current.is_none() {
                note(&"no context selected — seismic-tee ctx use <network>[/<node>]");
            }
            list_lines(
                &context,
                scope.as_ref().map(|s| s.selection.network.as_str()),
            )
        }
    };
    for line in lines {
        println!("{line}");
    }
    Ok(ExitCode::SUCCESS)
}

/// The one network `list` narrows to, resolved against the file: `NETWORK`,
/// else the `--context`/`SEISMIC_CONTEXT` selection's network. `--names`
/// must have one — its output feeds `--name`, which picks within a network,
/// so a bare name from an unspecified network would name nothing — and falls
/// back to the file's `current`. Without `--names`, no scope is the whole
/// file.
fn list_scope<'a>(context: &'a Context, args: &ListArgs) -> anyhow::Result<Option<Selected<'a>>> {
    let requested = args.network.as_deref().or(args.context.context.as_deref());
    if requested.is_none() {
        if !args.names {
            return Ok(None);
        }
        if context.config().current.is_none() {
            bail!(
                "ctx list --names needs a network: name one (`seismic-tee ctx list --names \
                 <network>`), or select one with `seismic-tee ctx use <network>`"
            );
        }
    }
    Ok(Some(context.select(requested)?))
}

/// `--names`: the scoped network's node names, bare, in table order. An
/// empty table is the same error every cohort read gives, naming `ctx
/// set-nodes` — a loop over nothing would pass silently otherwise.
fn name_lines(selected: &Selected<'_>) -> anyhow::Result<Vec<String>> {
    Ok(selected.nodes()?.keys().cloned().collect())
}

/// The human list: one group per network, `git branch` style. The network
/// line carries the marker when the selection stops at the network, and the
/// pointer to its artifact set; each node line carries the marker when the
/// selection names that node, and its fqdn. `scope` narrows it to one group.
/// Opens no file but the config: every field a line needs is already in it.
///
/// The marker is always the character left of the marked name, at either
/// depth, so a glance finds it without reading the names.
fn list_lines(context: &Context, scope: Option<&str>) -> Vec<String> {
    let config = context.config();
    let current = config
        .current
        .as_deref()
        .and_then(|raw| raw.parse::<Selection>().ok());
    let networks: Vec<(&String, &Network)> = config
        .networks
        .iter()
        .filter(|(name, _)| scope.is_none_or(|scope| scope == name.as_str()))
        .collect();
    let name_width = networks
        .iter()
        .map(|(name, _)| name.chars().count())
        .max()
        .unwrap_or(0);

    let mut lines = Vec::new();
    for (name, network) in networks {
        let selected_here = current
            .as_ref()
            .filter(|selection| &selection.network == name);
        let network_marker = match selected_here {
            Some(selection) if selection.node.is_none() => '*',
            _ => ' ',
        };
        lines.push(format!(
            "{network_marker} {name:<name_width$}  {}",
            pointer(network)
        ));

        if network.nodes.is_empty() {
            lines.push(format!(
                "    (no nodes: pulumi stack output nodes --json | seismic-tee ctx set-nodes {name})"
            ));
            continue;
        }
        let node_width = network
            .nodes
            .keys()
            .map(|node| node.chars().count())
            .max()
            .unwrap_or(0);
        for (node, descriptor) in &network.nodes {
            let node_marker = match selected_here {
                Some(selection) if selection.node.as_deref() == Some(node.as_str()) => '*',
                _ => ' ',
            };
            lines.push(format!(
                "  {node_marker} {node:<node_width$}  {}",
                descriptor.fqdn
            ));
        }
    }
    lines
}

/// Where a network's artifact set is, as the file spells it: the field and
/// its value, so what the line says is what `ctx set-network` would take.
fn pointer(network: &Network) -> String {
    match network.shape() {
        Shape::Dir(dir) => format!("dir {}", dir.display()),
        Shape::Loose {
            manifest: Some(manifest),
        } => format!("manifest {}", manifest.display()),
        Shape::Loose { manifest: None } => "nodes only".to_string(),
        Shape::Published { source, .. } => {
            format!("source {source} (fetching is not implemented)")
        }
    }
}

fn run_unset(args: ConfigArgs) -> anyhow::Result<ExitCode> {
    let context = Context::load(args.config.as_deref())?;
    write::clear_current(context.path())?;
    println!(
        "Cleared the current context in {}.",
        context.path().display()
    );
    Ok(ExitCode::SUCCESS)
}

fn run_set_network(args: SetNetworkArgs) -> anyhow::Result<ExitCode> {
    let context = Context::load(args.config.as_deref())?;
    // Stored absolute: the file is read from whatever directory the next
    // command runs in, so a path relative to this one would point nowhere.
    let network = Network {
        dir: args.dir.as_deref().map(path::absolute).transpose()?,
        manifest: args.manifest.as_deref().map(path::absolute).transpose()?,
        source: args.source,
        network_id: args.network_id,
        ..Default::default()
    };
    network.validate(&args.name, context.path())?;
    // The file is read before it is written, so a bad map leaves the
    // registration undone too.
    let nodes = args.nodes.as_deref().map(load_descriptors).transpose()?;
    write::set_network(context.path(), &args.name, &network)?;
    if let Some(nodes) = &nodes {
        write::set_nodes(context.path(), &args.name, nodes)?;
    }
    match nodes {
        Some(nodes) => println!(
            "Registered network {} with {} nodes in {}: {}.",
            args.name,
            nodes.len(),
            context.path().display(),
            nodes.keys().cloned().collect::<Vec<_>>().join(", "),
        ),
        None => println!(
            "Registered network {} in {}.",
            args.name,
            context.path().display()
        ),
    }
    let context = Context::load(args.config.as_deref())?;
    next_step::print("", &next_after_registration(context.config(), &args.name));
    Ok(ExitCode::SUCCESS)
}

fn run_set_nodes(args: SetNodesArgs) -> anyhow::Result<ExitCode> {
    let stdin = std::io::stdin();
    if stdin.is_terminal() {
        bail!(
            "set-nodes reads the map on stdin: `pulumi stack output nodes --json | seismic-tee \
             ctx set-nodes {}`, or `< nodes.json`",
            args.name
        );
    }
    let message = import_nodes(&args, &mut stdin.lock())?;
    println!("{message}");
    let context = Context::load(args.config.as_deref())?;
    next_step::print("", &next_after_registration(context.config(), &args.name));
    Ok(ExitCode::SUCCESS)
}

/// What follows registering network `name`, or importing its nodes, given
/// the file as it now stands.
///
/// A network directory whose harvest has not happened yet is a founding in
/// progress, and the cohort just imported is the one to harvest — so the next
/// command is `harvest`, after selecting the network if `init` did not
/// already. Anything else is a network to use: select it, naming a node when
/// the table has one to name, unless the selection already does. Nothing
/// when nothing is left to suggest.
fn next_after_registration(config: &Config, name: &str) -> Vec<String> {
    let Some(network) = config.networks.get(name) else {
        return Vec::new();
    };
    let current = config
        .current
        .as_deref()
        .and_then(|raw| raw.parse::<Selection>().ok())
        .filter(|selection| selection.network == name);
    let founding_unharvested = network
        .dir
        .as_deref()
        .is_some_and(|dir| !network.nodes.is_empty() && !NetworkDir::new(dir).harvest().exists());
    if founding_unharvested {
        let mut next = Vec::new();
        if current.is_none() {
            next.push(format!("seismic-tee ctx use {name}"));
        }
        next.push("seismic-tee network harvest".to_string());
        return next;
    }
    match (network.nodes.keys().next(), current) {
        (Some(_), Some(selection)) if selection.node.is_some() => Vec::new(),
        (Some(node), _) => vec![format!("seismic-tee ctx use {name}/{node}")],
        (None, Some(_)) => Vec::new(),
        (None, None) => vec![format!("seismic-tee ctx use {name}")],
    }
}

/// [`run_set_nodes`]'s body: read the whole of `input`, parse it as a
/// descriptor map naming `<stdin>` in every failure, and store it as
/// `[networks.<name>.nodes]` — creating the network entry when it is not yet
/// registered. `input` stands in for stdin so a test can hand this a reader
/// of its own bytes.
fn import_nodes(args: &SetNodesArgs, input: &mut impl Read) -> anyhow::Result<String> {
    let mut bytes = Vec::new();
    input
        .read_to_end(&mut bytes)
        .context("reading the descriptor map on stdin")?;
    let nodes = parse_descriptors(&"<stdin>", &bytes)?;

    let context = Context::load(args.config.as_deref())?;
    write::set_nodes(context.path(), &args.name, &nodes)?;

    let names = nodes.keys().cloned().collect::<Vec<_>>().join(", ");
    Ok(format!(
        "Imported {} nodes into network {} in {}: {names}.",
        nodes.len(),
        args.name,
        context.path().display(),
    ))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use clap::{CommandFactory, Parser};

    use super::*;

    /// The group as the binary mounts it.
    #[derive(Parser)]
    struct Probe {
        #[command(subcommand)]
        command: CtxCommand,
    }

    fn parse(argv: &[&str]) -> CtxCommand {
        Probe::try_parse_from(std::iter::once(&"probe").chain(argv))
            .expect("well-formed argv")
            .command
    }

    #[test]
    fn the_command_tree_is_well_formed() {
        Probe::command().debug_assert();
    }

    #[test]
    fn the_commands_are_listed_in_the_documented_order() {
        let names: Vec<_> = Probe::command()
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect();
        assert_eq!(
            names,
            [
                "use",
                "list",
                "view",
                "env",
                "exec",
                "set-network",
                "set-nodes",
                "unset"
            ]
        );
    }

    #[test]
    fn use_takes_exactly_one_positional() {
        assert!(Probe::try_parse_from(["probe", "use", "a"]).is_ok());
        assert!(Probe::try_parse_from(["probe", "use", "a", "b"]).is_err());
    }

    #[test]
    fn set_nodes_takes_exactly_one_positional() {
        assert!(Probe::try_parse_from(["probe", "set-nodes", "a"]).is_ok());
        assert!(Probe::try_parse_from(["probe", "set-nodes"]).is_err());
        assert!(Probe::try_parse_from(["probe", "set-nodes", "a", "b"]).is_err());
    }

    #[test]
    fn set_network_requires_one_of_the_three_shapes() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.toml");
        let command = parse(&[
            "set-network",
            "devnet-1",
            "--config",
            config.to_str().unwrap(),
        ]);
        let err = run(command).unwrap_err().to_string();
        assert!(err.contains("is empty"), "{err}");
        assert!(err.contains("ctx set-nodes devnet-1"), "{err}");
    }

    #[test]
    fn set_network_source_without_network_id_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.toml");
        let command = parse(&[
            "set-network",
            "devnet-1",
            "--source",
            "https://example.com/bundle",
            "--config",
            config.to_str().unwrap(),
        ]);
        let err = run(command).unwrap_err().to_string();
        assert!(err.contains("has a source but no network_id"), "{err}");
    }

    #[test]
    fn set_network_writes_a_dir_network() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.toml");
        let command = parse(&[
            "set-network",
            "devnet-1",
            "--dir",
            "/networks/devnet-1",
            "--config",
            config.to_str().unwrap(),
        ]);
        run(command).unwrap();

        let context = Context::load(Some(&config)).unwrap();
        assert_eq!(
            context.config().networks["devnet-1"].dir,
            Some(PathBuf::from("/networks/devnet-1"))
        );
    }

    #[test]
    fn set_network_with_nodes_file_registers_both_halves() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.toml");
        let nodes = dir.path().join("nodes.json");
        std::fs::write(&nodes, TWO_NODE_MAP).unwrap();
        run(parse(&[
            "set-network",
            "partner-net",
            "--manifest",
            "/m/network-manifest.json",
            "--nodes",
            nodes.to_str().unwrap(),
            "--config",
            config.to_str().unwrap(),
        ]))
        .unwrap();

        let context = Context::load(Some(&config)).unwrap();
        let network = &context.config().networks["partner-net"];
        assert_eq!(
            network.manifest.as_deref(),
            Some(Path::new("/m/network-manifest.json"))
        );
        assert_eq!(network.nodes.len(), 2);
        assert!(network.nodes.contains_key("alpha"));
    }

    /// A relative `--dir` is stored absolute, with `.` and `..` collapsed:
    /// the file is read from whatever directory the next command runs in.
    #[test]
    fn set_network_stores_a_relative_dir_as_absolute() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.toml");
        run(parse(&[
            "set-network",
            "devnet-1",
            "--dir",
            "./sub/../networks/devnet-1",
            "--config",
            config.to_str().unwrap(),
        ]))
        .unwrap();

        let context = Context::load(Some(&config)).unwrap();
        let stored = context.config().networks["devnet-1"].dir.clone().unwrap();
        assert_eq!(
            stored,
            std::env::current_dir().unwrap().join("networks/devnet-1")
        );
    }

    const TWO_NODE_MAP: &[u8] = br#"{
        "alpha": {"public_ip": "203.0.113.7", "fqdn": "alpha.example.com"},
        "beta": {"public_ip": "203.0.113.8", "fqdn": "beta.example.com"}
    }"#;

    /// Rewrite the config file: `extra` (e.g. `current = "..."` lines),
    /// followed by `devnet-1` registered with a two-node table.
    fn write_config(config_path: &Path, extra: &str) {
        std::fs::write(
            config_path,
            format!(
                "{extra}[networks.devnet-1]\ndir = \"/x\"\n\n[networks.devnet-1.nodes]\n\
                 alpha = {{ public_ip = \"203.0.113.7\", fqdn = \"alpha.example.com\" }}\n\
                 beta = {{ public_ip = \"203.0.113.8\", fqdn = \"beta.example.com\" }}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn use_with_network_and_node_writes_current() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        write_config(&config_path, "");

        run(parse(&[
            "use",
            "devnet-1/alpha",
            "--config",
            config_path.to_str().unwrap(),
        ]))
        .unwrap();

        let context = Context::load(Some(&config_path)).unwrap();
        assert_eq!(context.config().current.as_deref(), Some("devnet-1/alpha"));
    }

    #[test]
    fn use_refuses_a_node_the_table_does_not_hold() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        write_config(&config_path, "");

        let err = run(parse(&[
            "use",
            "devnet-1/gamma",
            "--config",
            config_path.to_str().unwrap(),
        ]))
        .unwrap_err()
        .to_string();
        assert!(err.contains("no node `gamma`"), "{err}");
        assert!(err.contains("alpha, beta"), "{err}");

        // Nothing was written.
        let context = Context::load(Some(&config_path)).unwrap();
        assert_eq!(context.config().current, None);
    }

    /// A second network is registered alongside `devnet-1` so the bare name
    /// resolves as a network (the "more than one registered" branch of the
    /// bare-name rule) rather than as a node of the lone network.
    #[test]
    fn use_accepts_a_network_only_selection_with_no_nodes_imported() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            "[networks.devnet-1]\ndir = \"/x\"\n\n[networks.devnet-2]\ndir = \"/y\"\n",
        )
        .unwrap();

        run(parse(&[
            "use",
            "devnet-2",
            "--config",
            config_path.to_str().unwrap(),
        ]))
        .unwrap();

        let context = Context::load(Some(&config_path)).unwrap();
        assert_eq!(context.config().current.as_deref(), Some("devnet-2"));
    }

    #[test]
    fn dash_swaps_current_and_previous() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        write_config(
            &config_path,
            "current = \"devnet-1/alpha\"\nprevious = \"devnet-1/beta\"\n\n",
        );

        run(parse(&[
            "use",
            "-",
            "--config",
            config_path.to_str().unwrap(),
        ]))
        .unwrap();

        let context = Context::load(Some(&config_path)).unwrap();
        assert_eq!(context.config().current.as_deref(), Some("devnet-1/beta"));
        assert_eq!(context.config().previous.as_deref(), Some("devnet-1/alpha"));
    }

    #[test]
    fn dash_with_no_previous_errors_naming_ctx_use() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        write_config(&config_path, "current = \"devnet-1/alpha\"\n\n");

        let err = run(parse(&[
            "use",
            "-",
            "--config",
            config_path.to_str().unwrap(),
        ]))
        .unwrap_err()
        .to_string();
        assert!(err.contains("ctx use"), "{err}");
    }

    #[test]
    fn a_bare_name_resolves_as_a_node_when_one_network_is_registered() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        write_config(&config_path, "");

        run(parse(&[
            "use",
            "alpha",
            "--config",
            config_path.to_str().unwrap(),
        ]))
        .unwrap();

        let context = Context::load(Some(&config_path)).unwrap();
        assert_eq!(context.config().current.as_deref(), Some("devnet-1/alpha"));
    }

    #[test]
    fn a_bare_name_that_is_a_registered_networks_name_selects_the_network() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        write_config(&config_path, "");

        run(parse(&[
            "use",
            "devnet-1",
            "--config",
            config_path.to_str().unwrap(),
        ]))
        .unwrap();

        let context = Context::load(Some(&config_path)).unwrap();
        assert_eq!(context.config().current.as_deref(), Some("devnet-1"));
    }

    #[test]
    fn a_bare_name_resolves_as_a_network_when_more_than_one_is_registered() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            "[networks.devnet-1]\ndir = \"/x\"\n\n[networks.partner-net]\ndir = \"/y\"\n",
        )
        .unwrap();

        run(parse(&[
            "use",
            "devnet-1",
            "--config",
            config_path.to_str().unwrap(),
        ]))
        .unwrap();

        let context = Context::load(Some(&config_path)).unwrap();
        assert_eq!(context.config().current.as_deref(), Some("devnet-1"));
    }

    #[test]
    fn use_with_no_argument_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.toml");
        let err = run(parse(&["use", "--config", config.to_str().unwrap()]))
            .unwrap_err()
            .to_string();
        assert!(err.contains("ctx use needs a target"), "{err}");
    }

    /// `view` reads nothing but bytes: a file `Context::load` would refuse
    /// still prints, and a missing one is an error naming the path and what
    /// creates it.
    #[test]
    fn view_reads_the_raw_file_or_names_the_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        let err = run(parse(&["view", "--config", config_path.to_str().unwrap()]))
            .unwrap_err()
            .to_string();
        assert!(err.contains("no context file at"), "{err}");
        assert!(err.contains("network init"), "{err}");

        std::fs::write(&config_path, "current = 3\nthis is not toml\n").unwrap();
        assert!(Context::load(Some(&config_path)).is_err());
        run(parse(&["view", "--config", config_path.to_str().unwrap()])).unwrap();
    }

    #[test]
    fn list_marks_the_current_entry() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        write_config(&config_path, "current = \"devnet-1/alpha\"\n\n");
        let context = Context::load(Some(&config_path)).unwrap();

        let lines = list_lines(&context, None);
        assert_eq!(
            lines,
            [
                "  devnet-1  dir /x",
                "  * alpha  alpha.example.com",
                "    beta   beta.example.com",
            ]
        );

        // A selection that stops at the network marks the network line.
        write_config(&config_path, "current = \"devnet-1\"\n\n");
        let context = Context::load(Some(&config_path)).unwrap();
        let lines = list_lines(&context, None);
        assert_eq!(lines[0], "* devnet-1  dir /x");
        assert!(
            lines[1..].iter().all(|line| line.starts_with("    ")),
            "{lines:?}"
        );

        // Nothing selected: no marker anywhere.
        write_config(&config_path, "");
        let context = Context::load(Some(&config_path)).unwrap();
        let lines = list_lines(&context, None);
        assert!(lines.iter().all(|line| !line.contains('*')), "{lines:?}");
    }

    /// `list` reaches only the config file: a `dir` that does not exist on
    /// disk does not stop its nodes from listing.
    #[test]
    fn list_reads_no_file_but_the_config() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        write_config(&config_path, "");

        let context = Context::load(Some(&config_path)).unwrap();
        let lines = list_lines(&context, None);
        assert_eq!(lines.len(), 3, "{lines:?}");
    }

    #[test]
    fn list_lists_a_network_with_no_nodes_as_one_line_naming_ctx_set_nodes() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(&config_path, "[networks.stale-net]\ndir = \"/x\"\n").unwrap();
        let context = Context::load(Some(&config_path)).unwrap();

        let lines = list_lines(&context, None);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert_eq!(lines[0], "  stale-net  dir /x");
        assert!(lines[1].contains("no nodes"), "{lines:?}");
        assert!(lines[1].contains("ctx set-nodes stale-net"), "{lines:?}");
    }

    /// `list`'s arguments as the binary would parse them.
    fn list_args(argv: &[&str]) -> ListArgs {
        match parse(&[&["list"], argv].concat()) {
            CtxCommand::List(args) => args,
            other => panic!("not a list: {other:?}"),
        }
    }

    /// The scoped network's node names, or the error resolving the scope or
    /// reading its table produced — what `ctx list --names` prints or refuses
    /// with, minus the printing.
    fn names_of(config_path: &Path, argv: &[&str]) -> anyhow::Result<Vec<String>> {
        let args = list_args(
            &[
                argv,
                &["--names", "--config", config_path.to_str().unwrap()],
            ]
            .concat(),
        );
        let context = Context::load(Some(config_path))?;
        let selected = list_scope(&context, &args)?.expect("--names always scopes");
        name_lines(&selected)
    }

    /// A second network beside `devnet-1`, so scoping has something to
    /// exclude.
    fn write_two_network_config(config_path: &Path, current: &str) {
        write_config(
            config_path,
            &format!(
                "{current}[networks.partner-net]\nmanifest = \"/y/network-manifest.json\"\n\n\
                 [networks.partner-net.nodes]\nmy-node = {{ public_ip = \"198.51.100.4\", \
                 fqdn = \"my-node.example.com\" }}\n\n"
            ),
        );
    }

    /// The bare names, in table order; the network is the positional, else
    /// the `--context` selection's, else `current`'s — a network-only
    /// `current` scopes as well as a node one.
    #[test]
    fn names_are_one_networks_bare_node_names() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        write_two_network_config(&config_path, "current = \"devnet-1/alpha\"\n\n");

        assert_eq!(names_of(&config_path, &[]).unwrap(), ["alpha", "beta"]);
        assert_eq!(
            names_of(&config_path, &["partner-net"]).unwrap(),
            ["my-node"]
        );
        assert_eq!(
            names_of(&config_path, &["--context", "partner-net/my-node"]).unwrap(),
            ["my-node"]
        );
        // The positional beats --context, as it beats the file's current.
        assert_eq!(
            names_of(&config_path, &["devnet-1", "--context", "partner-net"]).unwrap(),
            ["alpha", "beta"]
        );

        write_two_network_config(&config_path, "current = \"partner-net\"\n\n");
        assert_eq!(names_of(&config_path, &[]).unwrap(), ["my-node"]);
    }

    /// Nothing to scope to is a refusal naming both ways to supply one; a
    /// network the file does not hold names the ones it does; an empty table
    /// is the cohort read's own error, pointing at `ctx set-nodes`.
    #[test]
    fn names_refuses_rather_than_guessing_a_network() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        write_two_network_config(&config_path, "");

        let err = names_of(&config_path, &[]).unwrap_err().to_string();
        assert!(err.contains("ctx list --names <network>"), "{err}");
        assert!(err.contains("ctx use <network>"), "{err}");

        let err = names_of(&config_path, &["gone"]).unwrap_err().to_string();
        assert!(err.contains("no network `gone`"), "{err}");
        assert!(err.contains("devnet-1, partner-net"), "{err}");

        std::fs::write(&config_path, "[networks.stale-net]\ndir = \"/x\"\n").unwrap();
        let err = names_of(&config_path, &["stale-net"])
            .unwrap_err()
            .to_string();
        assert!(err.contains("has no nodes"), "{err}");
        assert!(err.contains("ctx set-nodes stale-net"), "{err}");
    }

    /// Without `--names`, a positional narrows the marked list to one network
    /// and nothing else changes; with neither, the whole file lists.
    #[test]
    fn list_narrows_to_the_named_network() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        write_two_network_config(&config_path, "current = \"devnet-1/alpha\"\n\n");
        let context = Context::load(Some(&config_path)).unwrap();

        let all = list_args(&["--config", config_path.to_str().unwrap()]);
        assert!(list_scope(&context, &all).unwrap().is_none());
        // Two groups: devnet-1 and its two nodes, partner-net and its one.
        assert_eq!(list_lines(&context, None).len(), 5);

        let one = list_args(&["partner-net", "--config", config_path.to_str().unwrap()]);
        let scope = list_scope(&context, &one).unwrap().unwrap();
        assert_eq!(scope.selection.network, "partner-net");
        assert_eq!(
            list_lines(&context, Some(&scope.selection.network)),
            [
                "  partner-net  manifest /y/network-manifest.json",
                "    my-node  my-node.example.com",
            ]
        );
    }

    #[test]
    fn unset_clears_current_and_leaves_previous_and_networks_intact() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        write_config(
            &config_path,
            "current = \"devnet-1/alpha\"\nprevious = \"devnet-1/beta\"\n\n",
        );

        run(parse(&["unset", "--config", config_path.to_str().unwrap()])).unwrap();

        let context = Context::load(Some(&config_path)).unwrap();
        assert_eq!(context.config().current, None);
        assert_eq!(context.config().previous.as_deref(), Some("devnet-1/beta"));
        assert!(context.config().networks.contains_key("devnet-1"));
    }

    #[test]
    fn set_nodes_imports_a_two_node_map_and_names_both_in_the_success_line() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let args = SetNodesArgs {
            name: "devnet-1".to_string(),
            config: Some(config_path.clone()),
        };

        let message = import_nodes(&args, &mut &TWO_NODE_MAP[..]).unwrap();
        assert!(message.contains("Imported 2 nodes"), "{message}");
        assert!(message.contains("alpha"), "{message}");
        assert!(message.contains("beta"), "{message}");

        let context = Context::load(Some(&config_path)).unwrap();
        assert_eq!(context.config().networks["devnet-1"].nodes.len(), 2);
    }

    #[test]
    fn set_nodes_refuses_a_malformed_map_naming_stdin_and_leaves_the_file_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let original = "current = \"devnet-1\"\n";
        std::fs::write(&config_path, original).unwrap();
        let args = SetNodesArgs {
            name: "devnet-1".to_string(),
            config: Some(config_path.clone()),
        };

        let err = import_nodes(&args, &mut &b"not json"[..])
            .unwrap_err()
            .to_string();
        assert!(err.contains("<stdin>"), "{err}");

        assert_eq!(std::fs::read_to_string(&config_path).unwrap(), original);
    }

    #[test]
    fn set_nodes_creates_an_unregistered_network_with_nodes_only_and_set_network_keeps_them() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let args = SetNodesArgs {
            name: "devnet-1".to_string(),
            config: Some(config_path.clone()),
        };
        import_nodes(&args, &mut &TWO_NODE_MAP[..]).unwrap();

        let context = Context::load(Some(&config_path)).unwrap();
        let network = &context.config().networks["devnet-1"];
        assert_eq!(network.dir, None);
        assert_eq!(network.nodes.len(), 2);

        run(parse(&[
            "set-network",
            "devnet-1",
            "--dir",
            "/networks/devnet-1",
            "--config",
            config_path.to_str().unwrap(),
        ]))
        .unwrap();

        let context = Context::load(Some(&config_path)).unwrap();
        let network = &context.config().networks["devnet-1"];
        assert_eq!(
            network.dir.as_deref(),
            Some(Path::new("/networks/devnet-1"))
        );
        assert_eq!(network.nodes.len(), 2);
    }

    #[test]
    fn a_second_import_with_one_node_fewer_drops_the_missing_node() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let args = SetNodesArgs {
            name: "devnet-1".to_string(),
            config: Some(config_path.clone()),
        };
        import_nodes(&args, &mut &TWO_NODE_MAP[..]).unwrap();

        const ONE_NODE_MAP: &[u8] =
            br#"{"alpha": {"public_ip": "203.0.113.7", "fqdn": "alpha.example.com"}}"#;
        import_nodes(&args, &mut &ONE_NODE_MAP[..]).unwrap();

        let context = Context::load(Some(&config_path)).unwrap();
        assert_eq!(
            context.config().networks["devnet-1"]
                .nodes
                .keys()
                .collect::<Vec<_>>(),
            ["alpha"]
        );
    }

    fn config(toml: &str) -> Config {
        toml::from_str(toml).unwrap()
    }

    const NODES: &str = r#"
[networks.devnet-1.nodes.beta]
public_ip = "10.0.0.2"
fqdn = "beta.example"
[networks.devnet-1.nodes.alpha]
public_ip = "10.0.0.1"
fqdn = "alpha.example"
"#;

    #[test]
    fn an_unharvested_founding_directory_is_next_harvested() {
        let dir = tempfile::tempdir().unwrap();
        let registered = format!(
            "[networks.devnet-1]\ndir = {:?}\n{NODES}",
            dir.path().to_str().unwrap()
        );
        // Selected by `init`: nothing to select again.
        assert_eq!(
            next_after_registration(
                &config(&format!("current = \"devnet-1\"\n{registered}")),
                "devnet-1"
            ),
            ["seismic-tee network harvest"]
        );
        // Not selected: select it first.
        assert_eq!(
            next_after_registration(&config(&registered), "devnet-1"),
            [
                "seismic-tee ctx use devnet-1",
                "seismic-tee network harvest"
            ]
        );
        // Harvested: the founding is under way or done, so this is a network
        // to use like any other.
        std::fs::create_dir_all(dir.path().join("inputs/harvest")).unwrap();
        assert_eq!(
            next_after_registration(&config(&registered), "devnet-1"),
            ["seismic-tee ctx use devnet-1/alpha"]
        );
    }

    #[test]
    fn a_network_to_use_is_next_selected_with_a_node_when_it_has_one() {
        let with_nodes = format!("[networks.devnet-1]\nmanifest = \"/m.json\"\n{NODES}");
        assert_eq!(
            next_after_registration(&config(&with_nodes), "devnet-1"),
            ["seismic-tee ctx use devnet-1/alpha"]
        );
        // The network is selected but no node is: still worth naming one.
        assert_eq!(
            next_after_registration(
                &config(&format!("current = \"devnet-1\"\n{with_nodes}")),
                "devnet-1"
            ),
            ["seismic-tee ctx use devnet-1/alpha"]
        );
        // A node is selected: nothing left to suggest.
        assert!(
            next_after_registration(
                &config(&format!("current = \"devnet-1/beta\"\n{with_nodes}")),
                "devnet-1"
            )
            .is_empty()
        );
        let without_nodes = "[networks.partner]\nmanifest = \"/m.json\"\n";
        assert_eq!(
            next_after_registration(&config(without_nodes), "partner"),
            ["seismic-tee ctx use partner"]
        );
        assert!(
            next_after_registration(
                &config(&format!("current = \"partner\"\n{without_nodes}")),
                "partner"
            )
            .is_empty()
        );
        assert!(next_after_registration(&config(without_nodes), "unregistered").is_empty());
    }
}
