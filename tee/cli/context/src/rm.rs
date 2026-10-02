//! `rm`: forget a context entry — a network's, or one node of its table —
//! and keep every file it points at.
//!
//! ```text
//! seismic-tee ctx rm testnet/my-node
//! seismic-tee ctx rm fixture-devnet
//! ```
//!
//! The counterpart of `set-network` and `set-nodes`, as `network rm` is
//! `network init`'s: whatever creates a thing removes it, and `ctx` only
//! ever writes pointers, so it only ever removes them. An entry has two
//! origins — `network init`, a directory the CLI made, and `ctx set-network
//! --dir`, one the operator pointed at — and the file does not record which,
//! so deleting a directory stays `network rm`'s.
//!
//! `<network>/<node>` drops the node from the table, and a selection on it
//! falls back to its network. `<network>` drops the whole entry, and a
//! selection naming it is cleared. Neither asks first, since both print the
//! commands that put the entry back, save in one case: a network whose
//! directory is still on disk. Once its entry is gone `network rm` cannot
//! reach that directory, because it takes names, not paths, so that removal
//! says so and waits for the name to be typed back.
//!
//! The target is spelled strictly. `ctx use`'s bare-name rule, where a lone
//! word may be a node, is a convenience for a selection, not for a removal.

use std::io::{BufRead, IsTerminal as _};
use std::path::Path;
use std::process::ExitCode;

use anyhow::bail;
use clap::Args;
use clap_complete::ArgValueCandidates;
use seismic_tee_common::{Descriptors, home, next_step};

use crate::config::Network;
use crate::{Context, Selection, complete, confirm, write};

#[derive(Debug, Args)]
pub struct RmArgs {
    /// <network> to forget a network, or <network>/<node> to drop one node
    /// from its table. No file is touched either way.
    #[arg(value_name = "NETWORK[/NODE]", add = ArgValueCandidates::new(complete::entries))]
    pub target: String,

    /// Forget a network whose directory is still on disk without asking for
    /// its name to be typed back — for a script, where nobody is there to
    /// type it.
    #[arg(long, short = 'y')]
    pub yes: bool,
}

pub fn run(args: RmArgs, config: Option<&Path>) -> anyhow::Result<ExitCode> {
    let stdin = std::io::stdin();
    let interactive = stdin.is_terminal();
    let undo = remove(&args, config, interactive, &mut stdin.lock())?;
    next_step::print_undo("", &undo);
    Ok(ExitCode::SUCCESS)
}

/// [`run`]'s body: the lookup, the confirmation when one is due, the write,
/// then the commands that undo it. `input` is read for the confirmation only
/// when `interactive` and not `--yes`.
fn remove(
    args: &RmArgs,
    config: Option<&Path>,
    interactive: bool,
    input: &mut impl BufRead,
) -> anyhow::Result<Vec<String>> {
    let target: Selection = args.target.parse()?;
    let context = Context::load(config)?;
    let selected = context.select(Some(&args.target))?;
    let network = selected.network;
    let name = target.network.as_str();
    let current = context.config().current.clone();
    let held = current
        .as_deref()
        .and_then(|raw| raw.parse::<Selection>().ok())
        .is_some_and(|selection| match &target.node {
            Some(_) => selection == target,
            None => selection.network == name,
        });
    let reselect = current
        .filter(|_| held)
        .map(|current| format!("seismic-tee ctx use {current}"));

    let Some(node) = &target.node else {
        if let Some(dir) = &network.dir {
            let root = home::expand_tilde(dir)?;
            if std::fs::symlink_metadata(&root).is_ok() {
                confirm_keeping(name, &root, args, interactive, input)?;
            }
        }
        write::remove_network(context.path(), name)?;
        println!(
            "Removed network {name} from {}{}.",
            context.path().display(),
            if held {
                ", and cleared the selection it held"
            } else {
                ""
            }
        );
        let mut undo: Vec<String> = set_network_command(name, network).into_iter().collect();
        if !network.nodes.is_empty() {
            undo.push(set_nodes_command(name, &network.nodes)?);
        }
        undo.extend(reselect);
        return Ok(undo);
    };

    let (node, _) = selected.node(Some(node))?;
    if network.nodes.len() == 1 && network.dir.is_none() {
        bail!(
            "`{node}` is the only node of network `{name}`, which holds nothing else — `seismic-tee \
             ctx rm {name}` forgets the network"
        );
    }
    write::remove_node(context.path(), name, node)?;
    println!(
        "Removed node {node} from network {name} in {}{}.",
        context.path().display(),
        if held {
            format!(", and moved the selection to {name}")
        } else {
            String::new()
        }
    );
    let mut undo = vec![set_nodes_command(name, &network.nodes)?];
    undo.extend(reselect);
    Ok(undo)
}

/// Say that `root` stays where it is and what would delete it, then wait
/// for `name` to be typed back unless `--yes`.
fn confirm_keeping(
    name: &str,
    root: &Path,
    args: &RmArgs,
    interactive: bool,
    input: &mut impl BufRead,
) -> anyhow::Result<()> {
    confirm::require_answerable(name, "forget", args.yes, interactive)?;
    confirm::danger(&format!(
        "warning: {} stays on disk, and once this entry is gone `network rm` cannot reach it — \
         it takes names, not paths.",
        root.display()
    ));
    confirm::danger(&format!(
        "warning: `seismic-tee network rm {name}` would delete both."
    ));
    if args.yes {
        return Ok(());
    }
    confirm::type_back(name, "nothing was removed", input)
}

/// The `ctx set-network` that restores `network`'s pointers, or nothing for
/// an entry that is a node table alone.
fn set_network_command(name: &str, network: &Network) -> Option<String> {
    let dir = network.dir.as_ref()?;
    let mut command = format!(
        "seismic-tee ctx set-network --name {} --dir {}",
        shell_word(name),
        shell_word(&dir.to_string_lossy())
    );
    if let Some(id) = &network.network_id {
        command.push_str(&format!(" --network-id {id}"));
    }
    Some(command)
}

/// The `ctx set-nodes` that restores `nodes` as the whole table, the map on
/// its stdin.
fn set_nodes_command(name: &str, nodes: &Descriptors) -> anyhow::Result<String> {
    Ok(format!(
        "echo {} | seismic-tee ctx set-nodes {}",
        shell_word(&serde_json::to_string(nodes)?),
        shell_word(name)
    ))
}

/// `word` as one POSIX shell word: bare when nothing in it is special,
/// else single-quoted.
fn shell_word(word: &str) -> String {
    let bare = !word.is_empty()
        && word
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-./:@%+=,~".contains(&b));
    if bare {
        word.to_string()
    } else {
        format!("'{}'", word.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::Config;

    use super::*;

    struct Sandbox {
        dir: tempfile::TempDir,
    }

    impl Sandbox {
        fn new() -> Self {
            Self {
                dir: tempfile::tempdir().unwrap(),
            }
        }

        fn config_path(&self) -> PathBuf {
            self.dir.path().join("config.toml")
        }

        fn write_config(&self, text: &str) {
            std::fs::write(self.config_path(), text).unwrap();
        }

        fn config(&self) -> Config {
            toml::from_str(&std::fs::read_to_string(self.config_path()).unwrap()).unwrap()
        }

        /// `rm` with no terminal and no `--yes`: what a removal that asks
        /// nothing must get through.
        fn rm(&self, target: &str) -> anyhow::Result<Vec<String>> {
            self.rm_with(target, false, None)
        }

        /// `rm` at a terminal where `typed` is the answer, or with no
        /// terminal when `None`.
        fn rm_with(
            &self,
            target: &str,
            yes: bool,
            typed: Option<&str>,
        ) -> anyhow::Result<Vec<String>> {
            let args = RmArgs {
                target: target.to_string(),
                yes,
            };
            let answer = format!("{}\n", typed.unwrap_or_default());
            remove(
                &args,
                Some(&self.config_path()),
                typed.is_some(),
                &mut answer.as_bytes(),
            )
        }
    }

    const NODES: &str = "[networks.testnet.nodes]\n\
        alpha = { public_ip = \"203.0.113.7\", fqdn = \"a.example.com\" }\n\
        my-node = { public_ip = \"203.0.113.8\", fqdn = \"m.example.com\" }\n";

    const TABLE: &str = r#"echo '{"alpha":{"public_ip":"203.0.113.7","fqdn":"a.example.com"},"my-node":{"public_ip":"203.0.113.8","fqdn":"m.example.com"}}' | seismic-tee ctx set-nodes testnet"#;

    /// `testnet` pointing at a directory that is not there, with two nodes.
    fn testnet(sandbox: &Sandbox, current: &str) -> PathBuf {
        let root = sandbox.dir.path().join("gone");
        sandbox.write_config(&format!(
            "{current}[networks.testnet]\ndir = {:?}\n\n{NODES}",
            root.to_str().unwrap()
        ));
        root
    }

    #[test]
    fn a_node_leaves_the_table_and_its_selection_falls_back_to_the_network() {
        let sandbox = Sandbox::new();
        testnet(
            &sandbox,
            "current = \"testnet/my-node\"\nprevious = \"testnet/alpha\"\n\n",
        );

        let undo = sandbox.rm("testnet/my-node").unwrap();

        let config = sandbox.config();
        assert_eq!(config.current.as_deref(), Some("testnet"));
        assert_eq!(config.previous.as_deref(), Some("testnet/alpha"));
        let network = &config.networks["testnet"];
        assert!(network.dir.is_some());
        assert_eq!(network.nodes.keys().collect::<Vec<_>>(), ["alpha"]);
        assert_eq!(undo, [TABLE, "seismic-tee ctx use testnet/my-node"]);
    }

    #[test]
    fn a_node_not_in_the_table_is_refused_naming_the_ones_that_are() {
        let sandbox = Sandbox::new();
        testnet(&sandbox, "");

        let err = sandbox.rm("testnet/gamma").unwrap_err().to_string();
        assert!(err.contains("no node `gamma`"), "{err}");
        assert!(err.contains("alpha, my-node"), "{err}");

        let err = sandbox.rm("devnet-1/alpha").unwrap_err().to_string();
        assert!(err.contains("no network `devnet-1`"), "{err}");
        assert!(err.contains("it holds testnet"), "{err}");
    }

    /// A strict spelling: no `-`, and a bare node name is not read as one.
    #[test]
    fn the_target_is_never_resolved_the_way_ctx_use_resolves_a_bare_word() {
        let sandbox = Sandbox::new();
        testnet(&sandbox, "previous = \"testnet\"\n");

        let err = sandbox.rm("alpha").unwrap_err().to_string();
        assert!(err.contains("no network `alpha`"), "{err}");
        let err = sandbox.rm("-").unwrap_err().to_string();
        assert!(err.contains("no network `-`"), "{err}");
        assert_eq!(sandbox.config().networks["testnet"].nodes.len(), 2);
    }

    #[test]
    fn the_only_node_of_an_entry_holding_nothing_else_is_refused() {
        let sandbox = Sandbox::new();
        sandbox.write_config(
            "[networks.testnet.nodes]\nalpha = { public_ip = \"203.0.113.7\", fqdn = \
             \"a.example.com\" }\n",
        );

        let err = sandbox.rm("testnet/alpha").unwrap_err().to_string();
        assert!(err.contains("seismic-tee ctx rm testnet"), "{err}");
        assert_eq!(sandbox.config().networks["testnet"].nodes.len(), 1);
    }

    /// No directory on disk: nothing to warn about, so no terminal is
    /// needed, and the undo restores the pointer, the table and the
    /// selection.
    #[test]
    fn a_network_whose_directory_is_gone_is_forgotten_without_asking() {
        let sandbox = Sandbox::new();
        let root = testnet(&sandbox, "current = \"testnet/alpha\"\n\n");

        let undo = sandbox.rm("testnet").unwrap();

        let config = sandbox.config();
        assert!(config.networks.is_empty());
        assert_eq!(config.current, None);
        assert_eq!(
            undo,
            [
                format!(
                    "seismic-tee ctx set-network --name testnet --dir {}",
                    root.display()
                ),
                TABLE.to_string(),
                "seismic-tee ctx use testnet/alpha".to_string(),
            ]
        );
    }

    #[test]
    fn a_pinned_networks_undo_quotes_its_dir_and_restores_its_pin() {
        let sandbox = Sandbox::new();
        let root = sandbox.dir.path().join("my net");
        let id = "ab".repeat(32);
        sandbox.write_config(&format!(
            "current = \"other\"\n\n[networks.partner-net]\ndir = {:?}\nnetwork_id = \
             \"{id}\"\n\n[networks.other]\ndir = \"/m\"\n",
            root.to_str().unwrap()
        ));

        let undo = sandbox.rm("partner-net").unwrap();

        let config = sandbox.config();
        assert_eq!(config.current.as_deref(), Some("other"));
        assert_eq!(config.networks.keys().collect::<Vec<_>>(), ["other"]);
        assert_eq!(
            undo,
            [format!(
                "seismic-tee ctx set-network --name partner-net --dir '{}' --network-id {id}",
                root.display()
            )]
        );
    }

    /// A directory still on disk waits for the name, as `network rm` does —
    /// and, past the confirmation, is left exactly where it was.
    #[test]
    fn a_network_whose_directory_is_on_disk_waits_for_its_name() {
        let sandbox = Sandbox::new();
        let root = sandbox.dir.path().join("fixture-devnet");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("network-manifest.json"), "{}").unwrap();
        sandbox.write_config(&format!(
            "[networks.fixture-devnet]\ndir = {:?}\n",
            root.to_str().unwrap()
        ));

        let err = sandbox.rm("fixture-devnet").unwrap_err().to_string();
        assert!(err.contains("pass --yes"), "{err}");
        let err = sandbox
            .rm_with("fixture-devnet", false, Some("fixture"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("nothing was removed"), "{err}");
        assert!(sandbox.config().networks.contains_key("fixture-devnet"));

        sandbox
            .rm_with("fixture-devnet", false, Some("fixture-devnet"))
            .unwrap();
        assert!(sandbox.config().networks.is_empty());
        assert!(root.join("network-manifest.json").exists());

        sandbox.write_config(&format!(
            "[networks.fixture-devnet]\ndir = {:?}\n",
            root.to_str().unwrap()
        ));
        sandbox.rm_with("fixture-devnet", true, None).unwrap();
        assert!(sandbox.config().networks.is_empty());
        assert!(root.exists());
    }

    #[test]
    fn a_shell_word_is_bare_unless_something_in_it_is_special() {
        assert_eq!(shell_word("/x/fixture-devnet"), "/x/fixture-devnet");
        assert_eq!(shell_word("~/n"), "~/n");
        assert_eq!(shell_word("a b"), "'a b'");
        assert_eq!(shell_word("it's"), r"'it'\''s'");
        assert_eq!(shell_word(""), "''");
    }
}
