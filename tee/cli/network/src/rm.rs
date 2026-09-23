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
//! owns directories — `init` makes them — and `ctx` never deletes data. The
//! directory goes first and the entry second: a deletion that fails partway
//! leaves the entry pointing at what is left, so running `rm` again finishes
//! the job.
//!
//! Three refusals, each for something the CLI cannot tell is safe to lose:
//!
//! - **A directory git does not ignore.** Inside a work tree that is a
//!   committed network (a monorepo `tee/networks/<name>`) or one on its way
//!   to being committed, and git — not this CLI — decides whether it goes. A
//!   throwaway the work tree ignores (`tee/networks/tmp-*`) is not refused.
//! - **An entry with nodes registered**, unless `--force`: a `ctx set-nodes`
//!   table means a stack may still be live, and its Pulumi program refuses to
//!   preview without the network directory, so deleting the directory first
//!   strands the stack. `pulumi destroy` comes first.
//! - **A directory holding anything the layout does not own.** `rm` deletes a
//!   network directory, so an entry pointing at some other directory — a
//!   mistyped `ctx set-network --dir` — deletes nothing.
//!
//! A directory already gone is not an error: the entry is removed and that is
//! said, so the stale entry a hand `rm -rf` left behind is collected too. So
//! is an entry with no directory at all (a loose manifest, a bare node
//! table): those files were never the CLI's, and only the entry goes.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use anyhow::{Context as _, bail};
use clap::Args;
use clap_complete::ArgValueCandidates;
use seismic_tee_common::{NetworkDir, next_step};
use seismic_tee_context::{Context, Selection, complete, path, write};

#[derive(Debug, Args)]
pub struct RmArgs {
    /// The network to remove, by its name in the context file (`ctx list`
    /// names them all).
    #[arg(value_name = "NAME", add = ArgValueCandidates::new(complete::networks))]
    pub name: String,

    /// Remove it although nodes are registered for it — once the stack that
    /// provisioned them is destroyed, which the context file cannot tell.
    #[arg(long)]
    pub force: bool,

    /// Context file to write. Default: $XDG_CONFIG_HOME/seismic/config.toml,
    /// else ~/.config/seismic/config.toml.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

pub async fn run(args: RmArgs) -> anyhow::Result<ExitCode> {
    let (lead, next) = remove(&args)?;
    next_step::print(lead, &next);
    Ok(ExitCode::SUCCESS)
}

/// [`run`]'s body: the refusals, the deletion and the context write, then
/// the next step to print.
fn remove(args: &RmArgs) -> anyhow::Result<(&'static str, Vec<String>)> {
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
    if !network.nodes.is_empty() && !args.force {
        bail!(
            "network `{name}` has {} node(s) registered ({}) — the stack that provisioned them \
             may still be live, and its Pulumi program refuses to preview without the network \
             directory. Tear the stack down first (`pulumi destroy`, then `pulumi stack rm`), \
             then pass --force",
            network.nodes.len(),
            network.nodes.keys().cloned().collect::<Vec<_>>().join(", "),
        );
    }
    match &network.dir {
        Some(dir) => remove_dir(&path::expand_tilde(dir)?, name)?,
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
/// the refusals in the module docs applies. Says what it did on stderr.
fn remove_dir(root: &Path, name: &str) -> anyhow::Result<()> {
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
             or a symlink; remove it by hand, and `seismic-tee network rm {name}` then removes \
             the entry alone",
            root.display()
        );
    }
    if let Some(work_tree) = unignored_work_tree(root)? {
        bail!(
            "{} is in the git work tree {} and git does not ignore it: the network is committed, \
             or is on its way to being. Git, not this CLI, decides whether it goes — remove it \
             there (`git rm -r`, or `rm -r` if it was never committed), and `seismic-tee network \
             rm {name}` then removes the entry alone",
            root.display(),
            work_tree.display()
        );
    }
    let foreign = foreign_entries(&NetworkDir::new(root))?;
    if !foreign.is_empty() {
        bail!(
            "refusing to delete {}: it holds {}, which no network directory does — move them \
             out first, or remove the directory by hand and `seismic-tee network rm {name}` \
             then removes the entry alone",
            root.display(),
            foreign.join(", ")
        );
    }
    std::fs::remove_dir_all(root).with_context(|| format!("removing {}", root.display()))?;
    eprintln!("removed {}", root.display());
    Ok(())
}

/// The git work tree `root` is in, when it is in one and git does not
/// ignore it. Asking git rather than reading `.gitignore` files is what
/// makes the answer git's; the walk for a `.git` first is what keeps a
/// directory outside every repo from needing git installed at all.
fn unignored_work_tree(root: &Path) -> anyhow::Result<Option<PathBuf>> {
    let root =
        std::fs::canonicalize(root).with_context(|| format!("resolving {}", root.display()))?;
    let Some(work_tree) = root.ancestors().find(|dir| dir.join(".git").exists()) else {
        return Ok(None);
    };
    let status = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["check-ignore", "--quiet", "--"])
        .arg(&root)
        .stdin(Stdio::null())
        .status();
    match status.as_ref().map(std::process::ExitStatus::code) {
        Ok(Some(0)) => Ok(None),
        Ok(Some(1)) => Ok(Some(work_tree.to_path_buf())),
        _ => bail!(
            "{} is in the git work tree {}, and asking git whether it ignores the directory \
             failed ({}) — refusing rather than guessing",
            root.display(),
            work_tree.display(),
            match status {
                Ok(status) => status.to_string(),
                Err(e) => e.to_string(),
            }
        ),
    }
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

        fn rm(&self, name: &str, force: bool) -> anyhow::Result<(&'static str, Vec<String>)> {
            remove(&RmArgs {
                name: name.to_string(),
                force,
                config: Some(self.config_path()),
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
            "current = \"tmp-devnet-1\"\n\n{}\n[networks.partner-net]\nmanifest = \"/m.json\"\n",
            entry("tmp-devnet-1", &root)
        ));

        let (_, next) = sandbox.rm("tmp-devnet-1", false).unwrap();

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

        let (_, next) = sandbox.rm("tmp-devnet-1", false).unwrap();
        assert!(next.is_empty());
    }

    #[test]
    fn registered_nodes_refuse_without_force_and_touch_nothing() {
        let sandbox = Sandbox::new();
        let root = sandbox.network_dir("tmp-devnet-1");
        sandbox.write_config(&format!(
            "{}\n[networks.tmp-devnet-1.nodes]\n{NODES}",
            entry("tmp-devnet-1", &root)
        ));

        let err = sandbox.rm("tmp-devnet-1", false).unwrap_err().to_string();
        assert!(err.contains("1 node(s) registered (alpha)"), "{err}");
        assert!(err.contains("pulumi destroy"), "{err}");
        assert!(err.contains("--force"), "{err}");
        assert!(root.exists());
        assert!(sandbox.config().networks.contains_key("tmp-devnet-1"));

        sandbox.rm("tmp-devnet-1", true).unwrap();
        assert!(!root.exists());
        assert!(sandbox.config().networks.is_empty());
    }

    #[test]
    fn a_directory_already_gone_still_removes_the_entry() {
        let sandbox = Sandbox::new();
        let root = sandbox.dir.path().join("gone");
        sandbox.write_config(&entry("gone", &root));

        sandbox.rm("gone", false).unwrap();
        assert!(sandbox.config().networks.is_empty());
    }

    #[test]
    fn an_entry_with_no_directory_loses_only_the_entry() {
        let sandbox = Sandbox::new();
        let manifest = sandbox.dir.path().join("network-manifest.json");
        std::fs::write(&manifest, "{}").unwrap();
        sandbox.write_config(&format!(
            "[networks.partner-net]\nmanifest = {:?}\n",
            manifest.to_str().unwrap()
        ));

        sandbox.rm("partner-net", false).unwrap();
        assert!(manifest.exists());
        assert!(sandbox.config().networks.is_empty());
    }

    #[test]
    fn a_file_the_layout_does_not_own_refuses_and_deletes_nothing() {
        let sandbox = Sandbox::new();
        let root = sandbox.network_dir("tmp-devnet-1");
        std::fs::write(root.join("notes.txt"), "mine").unwrap();
        sandbox.write_config(&entry("tmp-devnet-1", &root));

        let err = sandbox.rm("tmp-devnet-1", false).unwrap_err().to_string();
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

        let err = sandbox.rm("odd", false).unwrap_err().to_string();
        assert!(err.contains("is not a directory"), "{err}");
        assert!(file.exists());
    }

    #[test]
    fn an_unknown_name_lists_the_registered_ones() {
        let sandbox = Sandbox::new();
        sandbox.write_config("[networks.devnet-1]\nmanifest = \"/m.json\"\n");

        let err = sandbox.rm("devnet-2", false).unwrap_err().to_string();
        assert!(err.contains("no network `devnet-2`"), "{err}");
        assert!(err.contains("it holds devnet-1"), "{err}");
    }

    fn git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    }

    /// The monorepo's shape: `tee/networks/` ignores `tmp-*/`, and any other
    /// directory there is a network to commit.
    #[test]
    fn git_decides_a_directory_inside_a_work_tree() {
        let sandbox = Sandbox::new();
        let repo = sandbox.dir.path().join("repo");
        std::fs::create_dir_all(repo.join("networks")).unwrap();
        git(&repo, &["init", "--quiet"]);
        std::fs::write(repo.join("networks/.gitignore"), "tmp-*/\n").unwrap();
        let committed = sandbox.network_dir("repo/networks/devnet");
        let throwaway = sandbox.network_dir("repo/networks/tmp-devnet-1");
        sandbox.write_config(&format!(
            "{}\n{}",
            entry("devnet", &committed),
            entry("tmp-devnet-1", &throwaway)
        ));

        let err = sandbox.rm("devnet", false).unwrap_err().to_string();
        assert!(err.contains("git does not ignore it"), "{err}");
        assert!(err.contains("git rm -r"), "{err}");
        assert!(committed.exists());
        assert!(sandbox.config().networks.contains_key("devnet"));

        sandbox.rm("tmp-devnet-1", false).unwrap();
        assert!(!throwaway.exists());
        assert_eq!(
            sandbox.config().networks.keys().collect::<Vec<_>>(),
            ["devnet"]
        );
    }
}
