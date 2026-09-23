//! `seismic-tee`: the Seismic TEE deploy CLI — one binary over the three
//! library crates.
//!
//! The command groups follow the parties of
//! [the trust model](https://github.com/SeismicSystems/seismic/blob/main/docs/tee/trust-model.md#the-trust-anchor-per-action),
//! whose table names who takes each trust-sensitive action in a network's
//! life:
//!
//! - `ctx` — anyone: name a network and select which one — and which of its
//!   nodes — is current ([`seismic_tee_context::cmd`]).
//! - `network` — the genesis deployer: found a network. The party's anchor is
//!   its own verification at assemble, which is what these commands
//!   implement ([`seismic_tee_network`]).
//! - `node` — standing up and appraising a node. Named for its subject rather
//!   than a party: the validator's actions in the table are the enclave's,
//!   not a human's, and a non-staking full-node operator runs the same
//!   commands ([`seismic_tee_node`]).
//! - `admission` — governance: measurement admission from the human side,
//!   the pipeline from an image's measurements to the policy record a
//!   network accepts. Authoring and review exist today; changing a live
//!   network's accepted set on-chain lands in the same group
//!   ([`seismic_tee_admission`]).
//! - `verify-founding` — the auditor's, at the top level: the auditor takes
//!   no trust-sensitive action, and their subject is the network's record as
//!   a whole, not any one party's work
//!   ([`seismic_tee_network::verify_founding`]).
//!
//! The client and security council have no CLI work today, so no group
//! stands empty for them.
//!
//! Two things belong to no party, so neither is a command: the root's
//! `--completions <SHELL>` option prints the shell code that turns tab
//! completion on, and `--upgrade [VERSION]` replaces this binary with a fresh
//! download ([`run_upgrade`]). Both sit with `--help` and `--version`, leaving
//! the command listing a who-runs-what. `--version` (`-v`, or clap's usual
//! `-V`) names the crate version and the commit the binary was built from
//! ([`VERSION`]): the releases are cut per merge as well as per version, so
//! the version alone does not identify a build. (The `help` subcommand is
//! disabled for the same reason; `--help` is the one spelling.) Completion is
//! dynamic — each tab press re-enters this binary through
//! [`CompleteEnv`], which walks the clap tree for flags and subcommand names
//! and calls the candidate functions in [`seismic_tee_context::complete`] for
//! the values an operator actually struggles to type: contexts, networks and
//! nodes, read from nothing but the context file.
//!
//! This crate is the mount point and nothing else. The library crates' one-way
//! dependency rule — `common` and `admission` on neither side, `node` only on
//! `common`, only `network` on both — is what keeps each party's crate free
//! of the others' dependencies, and a binary that links them all changes
//! nothing about it.

use std::io::Write as _;
use std::process::{Command as Process, ExitCode, Stdio};

use anyhow::{Context as _, bail};
use clap::{CommandFactory as _, Parser, Subcommand};
use clap_complete::Shell;
use clap_complete::env::{CompleteEnv, Shells};
use seismic_tee_admission::AdmissionCommand;
use seismic_tee_context::cmd::CtxCommand;
use seismic_tee_network::NetworkCommand;
use seismic_tee_network::verify_founding::VerifyFoundingArgs;
use seismic_tee_node::NodeCommand;

/// The name the binary is installed and invoked as.
const BIN_NAME: &str = "seismic-tee";

/// The environment variable the completion engine is entered through:
/// `COMPLETE=bash seismic-tee` prints the registration script, and the script
/// sets it on every callback. `--completions` prints the same script under a
/// spelling an operator can find in `--help`.
const COMPLETE_VAR: &str = "COMPLETE";

/// The repository the releases are cut from, `<owner>/<repo>`. The same
/// default the installer carries; `--upgrade` fetches the installer from here.
const REPO: &str = "SeismicSystems/seismic";

/// The installer's path in [`REPO`], on the default branch.
const INSTALLER_PATH: &str = "tee/cli/install.sh";

/// What `--version` prints after the name: the crate version, then the
/// commit the binary was built from, stamped by `build.rs` — `-dirty` when
/// the checkout had uncommitted changes, `unknown` when there was no git to
/// ask. `seismic-tee 0.1.0 (1a2b3c4d5)` is what an operator quotes when
/// reporting a problem and what a founder records beside a founding.
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("SEISMIC_TEE_BUILD_COMMIT"),
    ")"
);

#[derive(Debug, Parser)]
#[command(
    name = BIN_NAME,
    version = VERSION,
    // The flag is declared below, so it answers to `-v` as well as `-V`.
    disable_version_flag = true,
    about = "Found, join, govern and audit a Seismic TEE network",
    long_about = "The Seismic TEE deploy CLI.\n\n\
                  One command group per party of the trust model — network (the genesis \
                  deployer: found a network), node (stand up and appraise a node), admission \
                  (governance: the policies that decide which images a network accepts) — and \
                  the auditor's verify-founding at the top level.\n\n\
                  Never provisions: every command starts at the node descriptors the Pulumi \
                  program produces, or at a network directory.",
    // clap 4 dropped the `<name> <version>` line clap 2 and 3 opened help
    // with; this is that line back, and the rest of the template is clap's
    // default. A binary that replaces itself with `--upgrade` is a binary
    // whose help should say which one it is — help pasted into a bug report
    // carries the build commit with it.
    help_template = "\
{name} {version}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}",
    // The listing is the parties; `--help` is the one way to ask for it.
    disable_help_subcommand = true,
    // `--completions` stands alone, and nothing at all prints this help.
    args_conflicts_with_subcommands = true,
    arg_required_else_help = true
)]
struct Cli {
    /// Print the shell code that turns on tab completion, of command and
    /// context names alike: source <(seismic-tee --completions bash)
    #[arg(
        long,
        value_name = "SHELL",
        value_enum,
        num_args = 0..=1,
        long_help = "Print the shell code that turns on tab completion for seismic-tee.\n\n\
                     Completes flags and command names, and the values an operator types \
                     most: every <network> and <network>/<node> in the context file for \
                     `ctx use` and `--context`, the registered networks for `ctx set-nodes` \
                     and `ctx set-network`, and the selected network's nodes for `--name`, \
                     `--genesis-node` and `--join`. Names come from the context file alone — no \
                     RPC, no Pulumi — so a tab press never waits on the network.\n\n\
                     Add the line for your shell to its rc file:\n  \
                     bash:   source <(seismic-tee --completions bash)\n  \
                     zsh:    source <(seismic-tee --completions zsh)\n  \
                     fish:   seismic-tee --completions fish | source\n\n\
                     The code calls back into this binary on every tab press, at the path it \
                     was printed from, so re-source it after moving or upgrading the binary — \
                     an rc line does that on every new shell. SHELL defaults to what $SHELL \
                     names."
    )]
    completions: Option<Option<Shell>>,

    /// Replace this binary with a fresh download from the repository's
    /// releases: seismic-tee --upgrade [VERSION]
    #[arg(
        long,
        value_name = "VERSION",
        num_args = 0..=1,
        conflicts_with = "completions",
        long_help = "Replace this binary with a fresh download from the repository's releases.\n\n\
                     VERSION takes the release spellings: X.Y.Z or vX.Y.Z for a release, \
                     main-<sha> for one prerelease build, main for the tip of main. With no \
                     VERSION, the newest release. Downgrading is naming an older one.\n\n\
                     The new binary lands beside this one, not at the installer's default, so \
                     a copy installed outside ~/.local/bin is the copy replaced. Re-source \
                     your completion line afterwards if it names a path rather than the \
                     command.\n\n\
                     Fetches and runs this repository's installer, which checks the download \
                     against the release's SHA256SUMS and, through `gh`, the binary's build \
                     provenance attestation, when `gh` is installed and logged in."
    )]
    upgrade: Option<Option<String>>,

    /// Print the version and the commit this binary was built from
    #[arg(short = 'v', short_alias = 'V', long, action = clap::ArgAction::Version)]
    version: (),

    #[command(subcommand)]
    command: Option<Command>,
}

/// The groups in the trust model's order of appearance in a network's life:
/// founded, joined, governed — then audited. Each one-liner leads with the
/// party it is for, so the listing doubles as a who-runs-what.
#[derive(Debug, Subcommand)]
enum Command {
    /// Anyone: select the network and node the other commands act on, and
    /// reach the selected node from a shell.
    Ctx {
        #[command(subcommand)]
        command: CtxCommand,
    },
    /// Network founder: scaffold a network's inputs, harvest a cohort's
    /// founding keys, assemble its identity, configure the cohort.
    #[command(
        long_about = "Found a Seismic network: scaffold its inputs, harvest a cohort's founding \
                      keys, assemble its identity, and configure the cohort.\n\n\
                      Never provisions: like every group it starts at the node descriptors, \
                      which the Pulumi program produces.\n\n\
                      Joining an existing network is `seismic-tee node configure`, not a \
                      command of this group.",
        after_help = "Commands are listed in the order they should be run: init → harvest → \
                      assemble → configure. Between init and harvest, provision the cohort with \
                      the seismic_node Pulumi program (tee/pulumi/seismic_node); pulumi destroy \
                      tears it down, and rm then deletes the network directory and its \
                      context entry.\n\n\
                      Checking an assembled set later — after a merge, or when it may have \
                      drifted from its inputs — is `assemble --check`: the same derivation, \
                      compared with what is on disk instead of written. Checking a launched \
                      cohort — after a reboot or a re-image, or when its holders had not \
                      settled at launch — is `configure --check`: the launch assertions \
                      again, with nothing configured.\n\n\
                      Auditing a founding afterwards is `seismic-tee verify-founding`: not a \
                      founding step, and not a command of this group."
    )]
    Network {
        #[command(subcommand)]
        command: NetworkCommand,
    },
    /// Node operator: configure your node on first boot, verify its
    /// attestation, watch its first-boot disk wipe.
    #[command(
        long_about = "Stand up and appraise a Seismic TEE node: configure it on first boot, \
                      verify its attestation, watch its first-boot disk wipe.\n\n\
                      Cloud-agnostic, and never provisions: each command consumes a descriptor \
                      of an already-running node and reaches the node over HTTP."
    )]
    Node {
        #[command(subcommand)]
        command: NodeCommand,
    },
    /// Governance: author and review the measurement policies that decide
    /// which images a network accepts.
    #[command(
        long_about = "Measurement admission from the human side: the pipeline from an \
                      image's `make measure` output to the policy record a network accepts. \
                      promote authors the record; compile reports what it admits — the \
                      admission IDs and the registry genesis storage seeding them — for \
                      review before it is pinned at founding or proposed to a live network.\n\n\
                      Runs the same compiler `network assemble` pins the founding policy \
                      with. Changing a live network's accepted set on-chain is not here yet.",
        after_help = "Commands are listed in pipeline order: promote → compile."
    )]
    Admission {
        #[command(subcommand)]
        command: AdmissionCommand,
    },
    /// Auditor: verify a founding — the genesis validator set's TEE
    /// provenance — from a committed network directory, offline.
    VerifyFounding(VerifyFoundingArgs),
}

/// The registration script for `shell` (`--completions [SHELL]`), the same
/// one `COMPLETE=<shell> seismic-tee` prints — the engine's own spelling,
/// which the script itself uses when it calls back. Registered under the
/// installed name (the word the operator types), calling back to the binary
/// that printed it.
fn run_completions(shell: Option<Shell>) -> anyhow::Result<ExitCode> {
    let shell = match shell.or_else(Shell::from_env) {
        Some(shell) => shell,
        None => bail!(
            "cannot tell the shell from $SHELL — name it: seismic-tee --completions \
             <bash|zsh|fish|elvish|powershell>"
        ),
    };
    let shells = Shells::builtins();
    let completer = shells
        .completer(&shell.to_string())
        .with_context(|| format!("no dynamic completer for {shell}"))?;
    let bin_path = std::env::current_exe().context("locating this binary")?;
    let mut script = Vec::new();
    completer.write_registration(
        COMPLETE_VAR,
        BIN_NAME,
        BIN_NAME,
        &bin_path.to_string_lossy(),
        &mut script,
    )?;
    std::io::stdout().write_all(&script)?;
    Ok(ExitCode::SUCCESS)
}

/// Replace this binary with a fresh download, by running the installer a
/// first install runs.
///
/// The installer is fetched and run rather than reimplemented here. It
/// already picks the release for `version`, checks the tarball against the
/// release's SHA256SUMS, verifies the binary's build provenance through `gh`,
/// and installs by renaming over the target — which is the one way to replace
/// an executable while it is running, since the file cannot be written to but
/// can be renamed over. A second copy of that in Rust would be a second thing
/// to keep correct, and the way it would drift is towards checking less than
/// the installer does.
///
/// `--to` is this binary's own directory rather than the installer's default,
/// so upgrading a copy that lives outside `~/.local/bin` replaces that copy
/// instead of leaving a second one for `PATH` to choose between.
fn run_upgrade(version: Option<String>) -> anyhow::Result<ExitCode> {
    let exe = std::env::current_exe().context("locating this binary")?;
    let dir = exe
        .parent()
        .with_context(|| format!("{} has no parent directory", exe.display()))?;
    let script = fetch_installer()?;

    // argv, not a shell string: VERSION reaches the installer as one word and
    // is never parsed as shell syntax.
    let mut installer = Process::new("sh");
    installer.arg("-s").arg("--").arg("--to").arg(dir);
    if let Some(version) = &version {
        installer.arg("--version").arg(version);
    }
    let mut child = installer
        .stdin(Stdio::piped())
        .spawn()
        .context("running the installer with sh")?;
    // The handle drops at the end of the statement, which is the EOF `sh -s`
    // waits for before it runs anything.
    child
        .stdin
        .take()
        .expect("stdin was piped")
        .write_all(&script)
        .context("writing the installer to sh")?;
    let status = child.wait().context("waiting for the installer")?;
    if !status.success() {
        bail!("the installer did not finish; this binary is as it was");
    }
    Ok(ExitCode::SUCCESS)
}

/// The installer's source, over curl from the default branch — the same
/// fetch its one-liner makes. The installer needs curl anyway, and the
/// binary's build provenance check inside it is the one step that wants `gh`.
fn fetch_installer() -> anyhow::Result<Vec<u8>> {
    let url = format!("https://raw.githubusercontent.com/{REPO}/main/{INSTALLER_PATH}");
    let out = Process::new("curl")
        .args(["-fsSL", &url])
        .output()
        .context("running curl to fetch the installer")?;
    if !out.status.success() {
        bail!(
            "could not fetch the installer from {url}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out.stdout)
}

fn main() -> ExitCode {
    // A completion callback never reaches the parser: it answers and exits
    // here, before anything below can open a file or a socket.
    CompleteEnv::with_factory(Cli::command)
        .var(COMPLETE_VAR)
        .complete();
    let cli = Cli::parse();
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("error: starting the async runtime: {error}");
            return ExitCode::FAILURE;
        }
    };
    let result = runtime.block_on(async {
        match (cli.completions, cli.upgrade, cli.command) {
            (Some(shell), _, _) => run_completions(shell),
            (_, Some(version), _) => run_upgrade(version),
            (None, None, Some(command)) => match command {
                Command::Ctx { command } => seismic_tee_context::cmd::run(command),
                Command::Network { command } => seismic_tee_network::run(command).await,
                Command::Node { command } => seismic_tee_node::run(command).await,
                Command::Admission { command } => seismic_tee_admission::run(command),
                Command::VerifyFounding(args) => {
                    seismic_tee_network::verify_founding::run(args).await
                }
            },
            // `arg_required_else_help`: clap has already printed the help.
            (None, None, None) => unreachable!("clap requires an argument or a subcommand"),
        }
    });
    match result {
        Ok(code) => code,
        Err(error) => {
            // The whole chain, one cause per line: a DCAP failure is several
            // layers deep and the last one alone rarely says what happened.
            eprintln!("error: {error:?}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn the_command_tree_is_well_formed() {
        Cli::command().debug_assert();
    }

    /// The released binary reports the crate's version and its build
    /// commit, which is what a founder records alongside a founding and an
    /// operator quotes when reporting a problem — under `-v`, `-V` and
    /// `--version` alike.
    #[test]
    fn the_binary_is_named_and_versioned() {
        let command = Cli::command();
        assert_eq!(command.get_name(), BIN_NAME);
        assert_eq!(command.get_version(), Some(VERSION));
        let commit = VERSION
            .strip_prefix(concat!(env!("CARGO_PKG_VERSION"), " ("))
            .and_then(|rest| rest.strip_suffix(')'))
            .unwrap_or_else(|| panic!("{VERSION}"));
        assert!(!commit.is_empty());
        for flag in ["-v", "-V", "--version"] {
            let err = Cli::try_parse_from([BIN_NAME, flag]).unwrap_err();
            assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion, "{flag}");
        }
    }

    fn subcommand_names(command: &clap::Command) -> Vec<String> {
        command
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect()
    }

    /// One group per party with CLI work, in the order a network meets them,
    /// and the auditor's command at the top level. Nothing else in the
    /// listing: no `help` verb, and completion is an option, not a command.
    #[test]
    fn the_groups_follow_the_trust_models_parties() {
        let cli = Cli::command();
        assert_eq!(
            subcommand_names(&cli),
            ["ctx", "network", "node", "admission", "verify-founding"]
        );
        assert!(cli.is_disable_help_subcommand_set());
        let completions = cli
            .get_arguments()
            .find(|a| a.get_id() == "completions")
            .unwrap();
        assert!(!completions.is_hide_set());
        assert_eq!(completions.get_long(), Some("completions"));
        let group = |name: &str| subcommand_names(cli.find_subcommand(name).unwrap());
        assert_eq!(
            group("ctx"),
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
        assert_eq!(
            group("network"),
            ["init", "harvest", "assemble", "configure", "rm"]
        );
        assert_eq!(group("node"), ["configure", "verify", "status"]);
        assert_eq!(group("admission"), ["promote", "compile"]);
        assert!(subcommand_names(cli.find_subcommand("verify-founding").unwrap()).is_empty());
    }

    /// Every shell `--completions <SHELL>` accepts has a dynamic completer to
    /// print for: the argument's enum is clap_complete's static one, and its
    /// spellings must be the names the dynamic engine answers to.
    #[test]
    fn every_offered_shell_has_a_dynamic_completer() {
        use clap::ValueEnum as _;
        let shells = Shells::builtins();
        for shell in Shell::value_variants() {
            assert!(shells.completer(&shell.to_string()).is_some(), "{shell}");
        }
    }

    /// The candidate functions are wired to the arguments an operator types
    /// a context, network or node name into, and to no path argument. The
    /// closure itself is opaque; what can be checked is that the derive
    /// attached the extension.
    #[test]
    fn the_name_arguments_carry_completion_candidates() {
        use clap_complete::ArgValueCandidates;
        let cli = Cli::command();
        let arg = |path: &[&str], id: &str| {
            let mut command = &cli;
            for name in path {
                command = command.find_subcommand(name).unwrap();
            }
            command
                .get_arguments()
                .find(|a| a.get_id() == id)
                .unwrap_or_else(|| panic!("{path:?} has no `{id}`"))
                .get::<ArgValueCandidates>()
                .is_some()
        };
        assert!(arg(&["ctx", "use"], "selection"));
        assert!(arg(&["ctx", "set-nodes"], "name"));
        assert!(arg(&["ctx", "set-network"], "name"));
        for verb in ["configure", "verify", "status"] {
            assert!(arg(&["node", verb], "name"), "{verb}");
            assert!(arg(&["node", verb], "context"), "{verb}");
        }
        assert!(arg(&["network", "configure"], "genesis_node"));
        assert!(arg(&["network", "configure"], "join"));
        assert!(arg(&["network", "harvest"], "context"));
        assert!(arg(&["network", "rm"], "name"));
        assert!(arg(&["verify-founding"], "context"));
        // A path is not a name: the file arguments complete as files.
        assert!(!arg(&["node", "configure"], "node"));
    }

    /// "Harvest" is the founder's internal step name: an auditor never types
    /// it, and the retired `tools` spellings are gone rather than aliased.
    #[test]
    fn retired_spellings_do_not_parse() {
        for argv in [
            vec!["network", "verify-harvest", "n"],
            vec!["verify-harvest", "n"],
            vec!["network", "verify-founding", "n"],
            // re-deriving and comparing is `assemble --check`
            vec!["network", "validate", "n"],
            vec!["network", "validate"],
            vec!["network", "assemble", "--check", "--force", "n"],
            // init names the image, or gives all three inputs itself
            vec!["network", "init", "n"],
            vec!["network", "init", "n", "--reth-genesis", "g.json"],
            vec![
                "network",
                "init",
                "n",
                "--reth-genesis",
                "g.json",
                "--summit-genesis",
                "s.toml",
            ],
            vec![
                "network",
                "assemble",
                "--check",
                "--nodes",
                "nodes.json",
                "n",
            ],
            vec!["tools", "admission", "compile", "p.json"],
            vec!["network", "tools", "admission", "compile", "p.json"],
            vec!["admission", "compile", "-"],
            vec!["configure", "--node", "n.json", "--manifest", "m.json"],
            // the genesis node is named as one: bare `--genesis` read as a
            // file beside --reth-genesis and --summit-genesis
            vec!["network", "configure", "--genesis", "a"],
            // completion is an option; `help` is `--help`
            vec!["completions", "bash"],
            vec!["--completions", "bash", "ctx", "list"],
            vec!["ctx", "list", "--completions", "bash"],
            // upgrading is an option too, and stands as alone as completion
            vec!["upgrade"],
            vec!["--upgrade", "ctx", "list"],
            vec!["--upgrade", "--completions", "bash"],
            vec!["help"],
            vec!["help", "ctx"],
        ] {
            let full: Vec<&str> = std::iter::once(BIN_NAME)
                .chain(argv.iter().copied())
                .collect();
            let parsed = Cli::try_parse_from(&full);
            // `-` is a path like any other now, so it parses; it just names
            // a file called `-`. Everything else is a usage error.
            if argv == ["admission", "compile", "-"] {
                assert!(parsed.is_ok(), "{argv:?}");
            } else {
                assert!(parsed.is_err(), "{argv:?}");
            }
        }
    }

    /// `--upgrade` takes the installer's version grammar, or nothing at all
    /// for the newest release. The value is passed through rather than parsed
    /// here — the installer owns the grammar — so what this pins is that a
    /// bare flag and a flag with a value both reach [`run_upgrade`].
    #[test]
    fn upgrade_takes_an_optional_version() {
        let bare = Cli::try_parse_from([BIN_NAME, "--upgrade"]).unwrap();
        assert_eq!(bare.upgrade, Some(None));
        assert!(bare.command.is_none());

        for version in ["1.2.3", "v1.2.3", "main", "main-1a2b3c4d5"] {
            let parsed = Cli::try_parse_from([BIN_NAME, "--upgrade", version]).unwrap();
            assert_eq!(parsed.upgrade, Some(Some(version.to_string())), "{version}");
        }
    }

    /// The argv the README, the runbook and the founding workflow spell,
    /// exactly — and every shape a command's next-step line prints (see
    /// `seismic_tee_common::next_step`), so a printed invocation cannot drift
    /// from the command it names.
    #[test]
    fn the_documented_invocations_parse() {
        for argv in [
            // ctx
            vec!["ctx", "list"],
            vec!["ctx", "list", "--names"],
            vec!["ctx", "list", "--names", "devnet-1"],
            vec!["ctx", "view"],
            vec!["ctx", "unset"],
            vec!["ctx", "set-nodes", "devnet-1"],
            vec!["ctx", "use", "devnet-1/alpha"],
            vec!["ctx", "use", "devnet-1"],
            vec!["ctx", "env"],
            vec!["ctx", "env", "--unset"],
            vec!["ctx", "exec", "--", "scast", "block-number"],
            vec![
                "ctx",
                "exec",
                "--name",
                "alpha",
                "--",
                "scast",
                "rpc",
                "seismic_getTeePublicKey",
            ],
            vec![
                "ctx",
                "set-network",
                "my-node",
                "--manifest",
                "./network-manifest.json",
            ],
            // network
            vec![
                "network",
                "init",
                "tee/networks/devnet-3",
                "--image",
                "seismic_2026-09-22.2ee71c",
                "--founders",
                "2",
            ],
            // one input overridden, the rest from the image's release
            vec![
                "network",
                "init",
                "n",
                "--image",
                "seismic_2026-09-22.2ee71c",
                "--summit-genesis",
                "reviewed.toml",
            ],
            // no release: all three given
            vec![
                "network",
                "init",
                "tee/networks/devnet-3",
                "--reth-genesis",
                "https://github.com/SeismicSystems/seismic-images/releases/download/seismic_2026-09-22.2ee71c/reth-genesis.json",
                "--summit-genesis",
                "https://github.com/SeismicSystems/seismic-images/releases/download/seismic_2026-09-22.2ee71c/summit-genesis-starter.toml",
                "--measurements",
                "https://github.com/SeismicSystems/seismic-images/releases/download/seismic_2026-09-22.2ee71c/measurements.azure-tdx.json",
                "--founders",
                "2",
            ],
            vec![
                "network",
                "init",
                "n",
                "--reth-genesis",
                "g.json",
                "--summit-genesis",
                "s.toml",
                "--measurements",
                "m.json",
                "--name",
                "x",
                "--force",
            ],
            // deploy's .github/workflows/found-devnet.yml
            vec![
                "network",
                "init",
                "tee/networks/tmp-ci-1-1",
                "--image",
                "seismic_2026-09-22.2ee71c",
                "--founders",
                "4",
            ],
            vec!["network", "harvest", "tee/networks/devnet-3"],
            // resolved from the context
            vec!["network", "harvest"],
            vec!["network", "harvest", "--nodes", "nodes.json"],
            vec![
                "network",
                "harvest",
                "n",
                "--attestation-type",
                "azure-tdx",
                "--pccs-url",
                "http://pccs",
                "--force",
            ],
            vec!["network", "assemble", "tee/networks/devnet-3"],
            // resolved from the context
            vec!["network", "assemble"],
            vec!["network", "assemble", "--force"],
            vec!["network", "assemble", "--context", "devnet-1"],
            vec![
                "network",
                "assemble",
                "n",
                "--registry",
                "0x1000000000000000000000000000000000000001",
                "--authority",
                "0x1000000000000000000000000000000000000002",
                "--force",
                "--reth-bin",
                "/x/seismic-reth",
                "--summit-bin",
                "/x/summit",
            ],
            vec!["network", "assemble", "--check", "tee/networks/devnet-3"],
            // resolved from the context
            vec!["network", "assemble", "--check"],
            vec![
                "network",
                "assemble",
                "n",
                "--check",
                "--reth-bin",
                "r",
                "--summit-bin",
                "s",
            ],
            vec![
                "network",
                "configure",
                "--genesis-node",
                "devnet-3-1",
                "--manifest",
                "tee/networks/devnet-3/network-manifest.json",
            ],
            vec![
                "network",
                "configure",
                "--genesis-node",
                "a",
                "--join",
                "b",
                "--manifest",
                "m.json",
                "--no-verify",
                "--email",
                "x@y",
            ],
            vec![
                "network",
                "configure",
                "--genesis-node",
                "a",
                "--manifest",
                "m.json",
                "--measurements",
                "m.json",
                "--pccs-url",
                "http://pccs",
            ],
            // resolved from the context
            vec!["network", "configure", "--genesis-node", "a"],
            vec![
                "network",
                "configure",
                "--genesis-node",
                "a",
                "--context",
                "devnet-1",
            ],
            vec!["network", "rm", "tmp-devnet-1"],
            vec!["network", "rm", "tmp-devnet-1", "--force"],
            vec!["network", "remove", "tmp-devnet-1"],
            // node
            vec![
                "node",
                "configure",
                "--node",
                "/tmp/nodes.json",
                "--bootnode",
                "enode://ab@1.2.3.4:30303",
                "--manifest",
                "./network-manifest.json",
            ],
            vec![
                "node",
                "configure",
                "--node",
                "nodes/nodes.json",
                "--name",
                "tmp-devnet-1-2",
                "--bootnode",
                "enode://ab@1.2.3.4:30303",
                "--manifest",
                "network-manifest.json",
                "--no-verify",
                "--yes",
                "tmp-devnet-1-2",
            ],
            vec![
                "node",
                "configure",
                "-y",
                "dev-2",
                "--node",
                "n.json",
                "--bootnode",
                "enode://ab@1.2.3.4:30303",
                "--manifest",
                "m.json",
                "--dump-config",
                "/tmp/n.init-config.toml",
            ],
            vec![
                "node",
                "verify",
                "--node",
                "/tmp/nodes.json",
                "--manifest",
                "./network-manifest.json",
            ],
            vec![
                "node",
                "verify",
                "--node",
                "n.json",
                "--manifest",
                "m.json",
                "--measurements",
                "measurements.json",
                "--attestation-type",
                "azure-tdx",
                "--pccs-url",
                "http://pccs",
            ],
            vec!["node", "status", "--node", "n.json", "--name", "dev-2"],
            vec!["node", "status", "--node", "n.json", "--once"],
            vec!["node", "status", "--node", "n.json", "--interval", "10"],
            // resolved from the context
            vec!["node", "status"],
            vec!["node", "status", "--name", "alpha"],
            vec!["node", "verify", "--context", "devnet-1/alpha"],
            vec!["node", "verify", "--name", "dev-2", "--manifest", "m.json"],
            vec!["node", "status", "--context", "devnet-1/alpha"],
            vec![
                "node",
                "configure",
                "--bootnode",
                "enode://ab@1.2.3.4:30303",
                "--yes",
                "alpha",
            ],
            vec![
                "node",
                "configure",
                "--bootnode",
                "enode://ab@1.2.3.4:30303",
            ],
            vec![
                "node",
                "configure",
                "--name",
                "dev-2",
                "--bootnode",
                "enode://ab@1.2.3.4:30303",
            ],
            // admission
            vec![
                "admission",
                "compile",
                "tee/networks/devnet-3/measurement-policy-bootstrap.json",
            ],
            vec![
                "admission",
                "promote",
                "../seismic-images/build/measurements.azure-tdx.json",
                "--attestation-type",
                "azure-tdx",
            ],
            vec![
                "admission",
                "promote",
                "m.json",
                "--measurement-id",
                "img.vhd",
            ],
            // the audit
            vec!["verify-founding", "tee/networks/devnet-3"],
            vec!["verify-founding", "n", "--record", "n-2"],
            // resolved from the context
            vec!["verify-founding"],
            vec!["verify-founding", "--record", "n-2"],
            // tab completion
            vec!["--completions", "bash"],
            vec!["--completions", "zsh"],
            vec!["--completions", "fish"],
            vec!["--completions"],
        ] {
            let full: Vec<&str> = std::iter::once(BIN_NAME)
                .chain(argv.iter().copied())
                .collect();
            assert!(Cli::try_parse_from(&full).is_ok(), "{argv:?}");
        }
    }
}
