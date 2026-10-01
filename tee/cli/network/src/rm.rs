//! `rm`: the counterpart of [`crate::init`] — delete a network directory and
//! its entry in the context file.
//!
//! ```text
//! seismic-tee network rm tmp-devnet-1
//! ```
//!
//! `init` registers every directory it creates, so every network on a
//! machine is already named in the context file, and `rm` takes that name
//! rather than a path. It is in this group, not `ctx`, because this group
//! owns directories — `init` makes them — and `ctx` never deletes data:
//! `ctx rm` is the removal that keeps the directory. The directory goes first
//! and the entry second: a deletion that fails partway leaves the entry
//! pointing at what is left, so running `rm` again finishes the job.
//!
//! Deleting a directory waits for the network's name to be typed back, as
//! `pulumi stack rm` does; `--yes` skips that, and with no terminal it is
//! required. A directory in a git work tree is treated like any other: what
//! is committed git restores, and the typed name guards what is not.
//!
//! Registered nodes are a warning, not a refusal. The stack that provisioned
//! them should be destroyed first, since its Pulumi program cannot preview
//! without the network directory, but `pulumi destroy` never touches the
//! context file, so a table is still registered after every teardown and
//! cannot tell a live stack from a destroyed one. Whether the nodes' FQDNs
//! still resolve is the closer hint — `destroy` deletes the records — and
//! is said alongside, as a hint only: a resolver may answer from cache.
//!
//! A directory holding anything the layout does not own is refused: `rm`
//! deletes a network directory, so an entry pointing at some other directory
//! — a mistyped `ctx set-network --dir` — deletes nothing.
//!
//! A directory already gone is not an error: the entry is removed and that is
//! said, so the stale entry a hand `rm -rf` left behind is collected too. So
//! is an entry with no directory at all, a bare node table: only the entry
//! goes.

use std::io::{BufRead, IsTerminal as _};
use std::net::ToSocketAddrs as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, bail};
use clap::Args;
use clap_complete::ArgValueCandidates;
use seismic_tee_common::{Descriptors, NetworkDir, next_step};
use seismic_tee_context::{Context, Selection, complete, confirm, path, write};

#[derive(Debug, Args)]
pub struct RmArgs {
    /// The network to remove, by its name in the context file (`ctx list`
    /// names them all).
    #[arg(value_name = "NAME", add = ArgValueCandidates::new(complete::networks))]
    pub name: String,

    /// Delete the directory without asking for the name to be typed back —
    /// for a script, where nobody is there to type it.
    #[arg(long, short = 'y')]
    pub yes: bool,

    /// Context file to write. Default: $XDG_CONFIG_HOME/seismic/config.toml,
    /// else ~/.config/seismic/config.toml.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

pub async fn run(args: RmArgs) -> anyhow::Result<ExitCode> {
    let stdin = std::io::stdin();
    let interactive = stdin.is_terminal();
    let (lead, next) = remove(&args, interactive, &mut stdin.lock(), resolving)?;
    next_step::print(lead, &next);
    Ok(ExitCode::SUCCESS)
}

/// [`run`]'s body: the refusals, the confirmation, the deletion and the
/// context write, then the next step to print. `input` is read for the
/// confirmation only when `interactive` and not `--yes`; `resolving` is
/// [`resolving`] outside tests.
fn remove(
    args: &RmArgs,
    interactive: bool,
    input: &mut impl BufRead,
    resolving: impl FnOnce(&Descriptors) -> Vec<String>,
) -> anyhow::Result<(&'static str, Vec<String>)> {
    let context = Context::load(args.config.as_deref())?;
    let config = context.config();
    let name = &args.name;
    let Some(network) = config.networks.get(name) else {
        let names: Vec<&str> = config.networks.keys().map(String::as_str).collect();
        bail!(
            "no network `{name}` in {}; it holds {}",
            context.path().display(),
            if names.is_empty() {
                "none".to_string()
            } else {
                names.join(", ")
            }
        );
    };
    match &network.dir {
        Some(dir) => remove_dir(&path::expand_tilde(dir)?, name, |root| {
            confirm(
                root,
                name,
                &network.nodes,
                args,
                interactive,
                input,
                resolving,
            )
        })?,
        None => eprintln!("network `{name}` has no directory; only its entry goes"),
    }

    let selected = config
        .current
        .as_deref()
        .and_then(|raw| raw.parse::<Selection>().ok())
        .is_some_and(|selection| selection.network == *name);
    write::remove_network(context.path(), name)?;
    println!(
        "Removed network {name} from {}{}.",
        context.path().display(),
        if selected {
            ", and cleared the selection it held"
        } else {
            ""
        }
    );

    let others = config.networks.keys().any(|other| other != name);
    if selected && others {
        return Ok((
            "pick another network to select:",
            vec!["seismic-tee ctx list".to_string()],
        ));
    }
    Ok(("", Vec::new()))
}

/// Delete the network directory `root`, registered as `name`, unless one of
/// the refusals in the module docs applies or `confirm` declines. Says what
/// it did on stderr.
fn remove_dir(
    root: &Path,
    name: &str,
    confirm: impl FnOnce(&Path) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let metadata = match std::fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("{} is already gone", root.display());
            return Ok(());
        }
        Err(e) => return Err(e).with_context(|| format!("reading {}", root.display())),
    };
    if !metadata.is_dir() {
        bail!(
            "{} is not a directory — `network rm` deletes only a network directory, not a file \
             or a symlink; `seismic-tee ctx rm {name}` forgets the entry and leaves it be",
            root.display()
        );
    }
    let foreign = foreign_entries(&NetworkDir::new(root))?;
    if !foreign.is_empty() {
        bail!(
            "refusing to delete {}: it holds {}, which no network directory does — move them \
             out first, or `seismic-tee ctx rm {name}` forgets the entry and keeps the directory",
            root.display(),
            foreign.join(", ")
        );
    }
    confirm(root)?;
    std::fs::remove_dir_all(root).with_context(|| format!("removing {}", root.display()))?;
    eprintln!("removed {}", root.display());
    Ok(())
}

/// Wait for `name` to be typed back before `root` goes, unless `--yes`,
/// warning first about `nodes`. A wrong answer is an error, so nothing after
/// it runs; no terminal without `--yes` is one too — a script that means it
/// says so with the flag, rather than a redirected stdin deciding.
fn confirm(
    root: &Path,
    name: &str,
    nodes: &Descriptors,
    args: &RmArgs,
    interactive: bool,
    input: &mut impl BufRead,
    resolving: impl FnOnce(&Descriptors) -> Vec<String>,
) -> anyhow::Result<()> {
    confirm::require_answerable(name, "delete", args.yes, interactive)?;
    if !args.yes {
        confirm::danger(&format!(
            "This will permanently delete {}. (`seismic-tee ctx rm {name}` forgets the entry \
             and keeps the directory.)",
            root.display()
        ));
    }
    if !nodes.is_empty() {
        nodes_warning(nodes, &resolving(nodes))
            .lines()
            .for_each(confirm::danger);
    }
    if args.yes {
        return Ok(());
    }
    confirm::type_back(name, "nothing was deleted", input)
}

/// What to say about a network with `nodes` registered, given the FQDNs
/// among theirs that still `resolve`.
fn nodes_warning(nodes: &Descriptors, resolve: &[String]) -> String {
    let mut warning = format!(
        "warning: nodes are registered for it ({}). Destroy their stack first (`pulumi \
         destroy`, then `pulumi stack rm`) — its program cannot preview without this directory.",
        nodes.keys().cloned().collect::<Vec<_>>().join(", ")
    );
    if !resolve.is_empty() {
        warning.push_str(&format!(
            "\nwarning: {} still {} — the stack may still be live, or a resolver is answering \
             from cache.",
            resolve.join(", "),
            if resolve.len() == 1 {
                "resolves"
            } else {
                "resolve"
            }
        ));
    }
    warning
}

/// The FQDNs among `nodes`' that resolve, in node-name order, looked up in
/// parallel.
fn resolving(nodes: &Descriptors) -> Vec<String> {
    std::thread::scope(|scope| {
        let lookups: Vec<_> = nodes
            .values()
            .map(|node| {
                let fqdn = node.fqdn.as_str();
                scope.spawn(move || {
                    (fqdn, 443)
                        .to_socket_addrs()
                        .is_ok_and(|mut addrs| addrs.next().is_some())
                        .then(|| fqdn.to_string())
                })
            })
            .collect();
        lookups
            .into_iter()
            .filter_map(|lookup| lookup.join().ok().flatten())
            .collect()
    })
}

/// The names directly under `dir` that its layout does not own, sorted.
fn foreign_entries(dir: &NetworkDir) -> anyhow::Result<Vec<String>> {
    let owned = dir.top_level();
    let mut foreign = Vec::new();
    for entry in std::fs::read_dir(dir.root())
        .with_context(|| format!("reading {}", dir.root().display()))?
    {
        let entry = entry.with_context(|| format!("reading {}", dir.root().display()))?;
        if !owned.contains(&entry.path()) {
            foreign.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    foreign.sort();
    Ok(foreign)
}

#[cfg(test)]
mod tests {
    use seismic_tee_context::config::Config;

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

        /// A network directory with an input and an artifact in it, as
        /// `init` and `assemble` leave one.
        fn network_dir(&self, relative: &str) -> PathBuf {
            let root = self.dir.path().join(relative);
            let dir = NetworkDir::new(&root);
            std::fs::create_dir_all(dir.harvest()).unwrap();
            std::fs::write(dir.input_measurements(), "{}").unwrap();
            std::fs::write(dir.manifest(), "{}").unwrap();
            root
        }

        fn write_config(&self, text: &str) {
            std::fs::write(self.config_path(), text).unwrap();
        }

        fn config(&self) -> Config {
            toml::from_str(&std::fs::read_to_string(self.config_path()).unwrap()).unwrap()
        }

        fn rm(&self, name: &str) -> anyhow::Result<(&'static str, Vec<String>)> {
            self.rm_typing(name, Some(name))
        }

        /// `rm` at a terminal where `typed` is the answer, or with no
        /// terminal when `None`.
        fn rm_typing(
            &self,
            name: &str,
            typed: Option<&str>,
        ) -> anyhow::Result<(&'static str, Vec<String>)> {
            let args = RmArgs {
                name: name.to_string(),
                yes: false,
                config: Some(self.config_path()),
            };
            let answer = format!("{}\n", typed.unwrap_or_default());
            remove(&args, typed.is_some(), &mut answer.as_bytes(), |_| {
                Vec::new()
            })
        }
    }

    fn entry(name: &str, dir: &Path) -> String {
        format!("[networks.{name}]\ndir = {:?}\n", dir.to_str().unwrap())
    }

    const NODES: &str = "alpha = { public_ip = \"203.0.113.7\", fqdn = \"a.example.com\" }\n";

    #[test]
    fn the_directory_and_the_entry_go_and_the_selection_is_cleared() {
        let sandbox = Sandbox::new();
        let root = sandbox.network_dir("tmp-devnet-1");
        sandbox.write_config(&format!(
            "current = \"tmp-devnet-1\"\n\n{}\n[networks.partner-net.nodes]\nmy-node = {{ \
             public_ip = \"198.51.100.4\", fqdn = \"n.example.com\" }}\n",
            entry("tmp-devnet-1", &root)
        ));

        let (_, next) = sandbox.rm("tmp-devnet-1").unwrap();

        assert!(!root.exists());
        let config = sandbox.config();
        assert_eq!(config.current, None);
        assert_eq!(config.networks.keys().collect::<Vec<_>>(), ["partner-net"]);
        assert_eq!(next, ["seismic-tee ctx list"]);
    }

    #[test]
    fn nothing_is_suggested_when_no_network_is_left() {
        let sandbox = Sandbox::new();
        let root = sandbox.network_dir("tmp-devnet-1");
        sandbox.write_config(&format!(
            "current = \"tmp-devnet-1\"\n\n{}",
            entry("tmp-devnet-1", &root)
        ));

        let (_, next) = sandbox.rm("tmp-devnet-1").unwrap();
        assert!(next.is_empty());
    }

    #[test]
    fn registered_nodes_do_not_refuse() {
        let sandbox = Sandbox::new();
        let root = sandbox.network_dir("tmp-devnet-1");
        sandbox.write_config(&format!(
            "{}\n[networks.tmp-devnet-1.nodes]\n{NODES}",
            entry("tmp-devnet-1", &root)
        ));

        sandbox.rm("tmp-devnet-1").unwrap();
        assert!(!root.exists());
        assert!(sandbox.config().networks.is_empty());
    }

    #[test]
    fn the_nodes_warning_names_what_still_resolves() {
        let nodes: Descriptors = toml::from_str(NODES).unwrap();

        let warning = nodes_warning(&nodes, &[]);
        assert!(warning.contains("registered for it (alpha)"), "{warning}");
        assert!(warning.contains("pulumi destroy"), "{warning}");
        assert!(!warning.contains("resolve"), "{warning}");

        let warning = nodes_warning(&nodes, &["a.example.com".to_string()]);
        assert!(
            warning.contains("a.example.com still resolves"),
            "{warning}"
        );
        assert!(warning.contains("from cache"), "{warning}");
    }

    #[test]
    fn a_directory_already_gone_still_removes_the_entry() {
        let sandbox = Sandbox::new();
        let root = sandbox.dir.path().join("gone");
        sandbox.write_config(&entry("gone", &root));

        sandbox.rm("gone").unwrap();
        assert!(sandbox.config().networks.is_empty());
    }

    #[test]
    fn an_entry_with_no_directory_loses_only_the_entry() {
        let sandbox = Sandbox::new();
        sandbox.write_config(
            "[networks.partner-net.nodes]\nmy-node = { public_ip = \"198.51.100.4\", fqdn = \
             \"n.example.com\" }\n",
        );

        sandbox.rm("partner-net").unwrap();
        assert!(sandbox.config().networks.is_empty());
    }

    #[test]
    fn a_file_the_layout_does_not_own_refuses_and_deletes_nothing() {
        let sandbox = Sandbox::new();
        let root = sandbox.network_dir("tmp-devnet-1");
        std::fs::write(root.join("notes.txt"), "mine").unwrap();
        sandbox.write_config(&entry("tmp-devnet-1", &root));

        let err = sandbox.rm("tmp-devnet-1").unwrap_err().to_string();
        assert!(err.contains("it holds notes.txt"), "{err}");
        assert!(root.join("inputs").exists());
        assert!(sandbox.config().networks.contains_key("tmp-devnet-1"));
    }

    #[test]
    fn a_non_directory_is_refused() {
        let sandbox = Sandbox::new();
        let file = sandbox.dir.path().join("a-file");
        std::fs::write(&file, "").unwrap();
        sandbox.write_config(&entry("odd", &file));

        let err = sandbox.rm("odd").unwrap_err().to_string();
        assert!(err.contains("is not a directory"), "{err}");
        assert!(file.exists());
    }

    #[test]
    fn an_unknown_name_lists_the_registered_ones() {
        let sandbox = Sandbox::new();
        sandbox.write_config("[networks.devnet-1]\ndir = \"/m\"\n");

        let err = sandbox.rm("devnet-2").unwrap_err().to_string();
        assert!(err.contains("no network `devnet-2`"), "{err}");
        assert!(err.contains("it holds devnet-1"), "{err}");
    }

    #[test]
    fn a_wrong_name_typed_back_deletes_nothing() {
        let sandbox = Sandbox::new();
        let root = sandbox.network_dir("tmp-devnet-1");
        sandbox.write_config(&entry("tmp-devnet-1", &root));

        let err = sandbox
            .rm_typing("tmp-devnet-1", Some("tmp-devnet-2"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("nothing was deleted"), "{err}");
        assert!(root.exists());
        assert!(sandbox.config().networks.contains_key("tmp-devnet-1"));
    }

    #[test]
    fn no_terminal_needs_yes() {
        let sandbox = Sandbox::new();
        let root = sandbox.network_dir("tmp-devnet-1");
        sandbox.write_config(&entry("tmp-devnet-1", &root));

        let err = sandbox
            .rm_typing("tmp-devnet-1", None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("pass --yes"), "{err}");
        assert!(root.exists());

        let args = RmArgs {
            name: "tmp-devnet-1".to_string(),
            yes: true,
            config: Some(sandbox.config_path()),
        };
        remove(&args, false, &mut std::io::empty(), |_| Vec::new()).unwrap();
        assert!(!root.exists());
    }
}
