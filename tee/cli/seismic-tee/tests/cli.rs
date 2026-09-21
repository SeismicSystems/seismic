#![cfg(unix)]
//! Black-box tests of the targeting verbs, and of a `node` command's echo
//! when it resolves its target from the context.
//!
//! `env` and `exec` are defined by what crosses the process boundary — the
//! bytes on stdout, the variables a child sees, the exit code that comes
//! back — so they are tested through the real binary rather than in process.
//! Each test runs `seismic-tee` (`env!("CARGO_BIN_EXE_seismic-tee")`) with a
//! cleared environment, `HOME`/`XDG_CONFIG_HOME` pointed at a fresh tempdir
//! carrying its own `config.toml`, and a scratch `PATH` holding one
//! executable stand-in: a `scast` shell script that appends its argv and
//! `$ETH_RPC_URL` and `$SEISMIC_CONTEXT` to `$SEISMIC_TEST_LOG` and exits 7.
//!
//! The same reasoning makes `node status`'s echo a subprocess test too: the
//! echo goes to the real stderr and the status to the real stdout, and only
//! a spawned child separates the two streams the way `Command::output` does.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;
use std::process::Command;

use seismic_tee_common::http::ATTESTATION_RPC_PORT;
use seismic_tee_common::test_support::{FakeServer, rpc_result};
use serde_json::json;

/// `devnet-1/alpha` current, a two-node table.
const TWO_NODE_CONFIG: &str = r#"
current = "devnet-1/alpha"

[networks.devnet-1]
dir = "/x"

[networks.devnet-1.nodes]
alpha = { public_ip = "203.0.113.7", fqdn = "alpha.example.com" }
beta = { public_ip = "203.0.113.8", fqdn = "beta.example.com" }
"#;

/// The same cohort, with the selection stopping at the network — no default
/// node — so a single-node command needs `--name` to say which.
const NETWORK_ONLY_CONFIG: &str = r#"
current = "devnet-1"

[networks.devnet-1]
dir = "/x"

[networks.devnet-1.nodes]
alpha = { public_ip = "203.0.113.7", fqdn = "alpha.example.com" }
beta = { public_ip = "203.0.113.8", fqdn = "beta.example.com" }
"#;

/// A network-only selection over a two-node cohort: `--name` supplies the
/// key the selection lacks, so the echo has a node to name that `current`
/// alone does not give it.
const NETWORK_ONLY_LOOPBACK_CONFIG: &str = r#"
current = "devnet-1"

[networks.devnet-1]
dir = "/x"

[networks.devnet-1.nodes]
alpha = { public_ip = "127.0.0.1", fqdn = "alpha.example.com" }
beta = { public_ip = "203.0.113.8", fqdn = "beta.example.com" }
"#;

/// `devnet-1/alpha` current, one node whose `public_ip` is loopback — so a
/// command that reaches it (rather than one that only prints its URL, like
/// `ctx env`) has something real on the other end.
const LOOPBACK_NODE_CONFIG: &str = r#"
current = "devnet-1/alpha"

[networks.devnet-1]
dir = "/x"

[networks.devnet-1.nodes]
alpha = { public_ip = "127.0.0.1", fqdn = "alpha.example.com" }
"#;

/// A tempdir standing in for the operator's whole environment: its own
/// `$HOME`/`$XDG_CONFIG_HOME`, and a `PATH` holding only the `scast` stub —
/// nothing a test does reaches outside it.
struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    fn new(config: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();

        let config_dir = dir.path().join("config/seismic");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config.toml"), config).unwrap();

        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let scast = bin_dir.join("scast");
        fs::write(
            &scast,
            "#!/bin/sh\n{ echo \"$@\"; echo \"$ETH_RPC_URL\"; echo \"$SEISMIC_CONTEXT\"; } >> \
             \"$SEISMIC_TEST_LOG\"\nexit 7\n",
        )
        .unwrap();
        fs::set_permissions(&scast, fs::Permissions::from_mode(0o755)).unwrap();

        Self { dir }
    }

    /// The binary, with a clean environment carrying only what the sandbox
    /// sets up. A test adds `SEISMIC_CONTEXT` or `--context` on top of this.
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_seismic-tee"));
        command
            .env_clear()
            .env("HOME", self.dir.path().join("home"))
            .env("XDG_CONFIG_HOME", self.dir.path().join("config"))
            .env("PATH", self.dir.path().join("bin"))
            .env("SEISMIC_TEST_LOG", self.log_path());
        command
    }

    fn log_path(&self) -> PathBuf {
        self.dir.path().join("scast.log")
    }

    fn log(&self) -> String {
        fs::read_to_string(self.log_path()).unwrap_or_default()
    }
}

/// `-v`, `-V` and `--version` are one flag: the name, the crate version and
/// the commit the binary was built from, on one line of stdout and nothing
/// on stderr. The commit is nine hex digits (with `-dirty` from an edited
/// checkout) or `unknown` from a tree with no git to ask.
#[test]
fn version_names_the_crate_version_and_the_build_commit() {
    let sandbox = Sandbox::new(TWO_NODE_CONFIG);
    let mut lines = Vec::new();
    for flag in ["-v", "-V", "--version"] {
        let output = sandbox.command().arg(flag).output().unwrap();
        assert!(output.status.success(), "{flag}: {output:?}");
        assert!(output.stderr.is_empty(), "{flag}: {output:?}");
        lines.push(String::from_utf8(output.stdout).unwrap());
    }
    assert!(lines.iter().all(|line| *line == lines[0]), "{lines:?}");

    let line = lines[0].trim_end();
    let commit = line
        .strip_prefix(concat!("seismic-tee ", env!("CARGO_PKG_VERSION"), " ("))
        .and_then(|rest| rest.strip_suffix(')'))
        .unwrap_or_else(|| panic!("{line}"));
    let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
    assert!(
        hash == "unknown" || (hash.len() == 9 && hash.bytes().all(|b| b.is_ascii_hexdigit())),
        "{line}"
    );
}

#[test]
fn env_prints_only_export_lines_on_stdout() {
    let sandbox = Sandbox::new(TWO_NODE_CONFIG);

    let output = sandbox.command().args(["ctx", "env"]).output().unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "export ETH_RPC_URL='https://alpha.example.com/rpc'\n\
         export SEISMIC_CONTEXT='devnet-1/alpha'\n"
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("context devnet-1/alpha →"), "{stderr}");
}

#[test]
fn env_unset_is_the_reverse() {
    let sandbox = Sandbox::new(TWO_NODE_CONFIG);

    let output = sandbox
        .command()
        .args(["ctx", "env", "--unset"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "unset ETH_RPC_URL\nunset SEISMIC_CONTEXT\n"
    );
}

/// `view`'s stdout is the file, byte for byte, and its path goes to stderr:
/// a redirect captures a faithful copy.
#[test]
fn view_prints_the_file_on_stdout_and_its_path_on_stderr() {
    let sandbox = Sandbox::new(TWO_NODE_CONFIG);

    let output = sandbox.command().args(["ctx", "view"]).output().unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), TWO_NODE_CONFIG);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        stderr.trim_end(),
        sandbox
            .dir
            .path()
            .join("config/seismic/config.toml")
            .to_str()
            .unwrap()
    );
}

/// `--names` is defined by its stdout: the bare names, one per line, with no
/// marker and no narration, so `$(…)` word-splits to exactly the table's
/// keys. The scope comes from `current` here — a network-only selection is
/// the shape a founder holds right after `init` and `set-nodes`.
#[test]
fn list_names_prints_bare_node_names_and_nothing_else() {
    let sandbox = Sandbox::new(NETWORK_ONLY_CONFIG);

    let output = sandbox
        .command()
        .args(["ctx", "list", "--names"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "alpha\nbeta\n");
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
}

/// The loop the ticket is for, run by a shell: `ctx list --names` is the
/// header and `ctx exec --name` the body, so `scast` reaches each node in
/// turn with that node's URL and a context pinned to it — and one child's
/// exit code does not stop the loop.
#[test]
fn a_shell_loop_over_list_names_reaches_every_node_through_exec() {
    let sandbox = Sandbox::new(NETWORK_ONLY_CONFIG);

    let output = sandbox
        .command()
        .env("SEISMIC_TEE", env!("CARGO_BIN_EXE_seismic-tee"))
        .args([
            "ctx",
            "exec",
            "--name",
            "alpha",
            "--",
            "/bin/sh",
            "-c",
            // Under `exec`, SEISMIC_CONTEXT is pinned to alpha: the loop's
            // own `--name` must win over it for beta.
            r#"for n in $("$SEISMIC_TEE" ctx list --names); do
                   "$SEISMIC_TEE" ctx exec --name "$n" -- scast block-number
               done"#,
        ])
        .output()
        .unwrap();

    // The shell's status is its last command's: the stub's 7, from beta —
    // each body carries its own exit code, and alpha's did not end the loop.
    assert_eq!(output.status.code(), Some(7), "{output:?}");
    assert_eq!(
        sandbox.log(),
        "block-number\nhttps://alpha.example.com/rpc\ndevnet-1/alpha\n\
         block-number\nhttps://beta.example.com/rpc\ndevnet-1/beta\n"
    );
}

#[test]
fn exec_hands_the_child_the_selected_nodes_rpc_url_and_context() {
    let sandbox = Sandbox::new(TWO_NODE_CONFIG);

    let output = sandbox
        .command()
        .args(["ctx", "exec", "--", "scast", "block-number"])
        .output()
        .unwrap();

    assert!(output.status.success() || output.status.code() == Some(7));
    let log = sandbox.log();
    let mut lines = log.lines();
    assert_eq!(lines.next(), Some("block-number"));
    assert_eq!(lines.next(), Some("https://alpha.example.com/rpc"));
    assert_eq!(lines.next(), Some("devnet-1/alpha"));
}

#[test]
fn exec_propagates_the_childs_exit_code() {
    let sandbox = Sandbox::new(TWO_NODE_CONFIG);

    let output = sandbox
        .command()
        .args(["ctx", "exec", "--", "scast"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(7));
}

#[test]
fn exec_dry_run_runs_nothing() {
    let sandbox = Sandbox::new(TWO_NODE_CONFIG);

    let output = sandbox
        .command()
        .args(["ctx", "exec", "--dry-run", "--", "scast", "block-number"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(!sandbox.log_path().exists());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("scast block-number"), "{stdout}");
    assert!(stdout.contains("https://alpha.example.com/rpc"), "{stdout}");
    assert!(
        stdout.contains("SEISMIC_CONTEXT=devnet-1/alpha"),
        "{stdout}"
    );
}

#[test]
fn the_env_var_beats_the_persisted_selection() {
    let sandbox = Sandbox::new(TWO_NODE_CONFIG); // current = devnet-1/alpha

    let output = sandbox
        .command()
        .env("SEISMIC_CONTEXT", "devnet-1/beta")
        .args(["ctx", "env"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("devnet-1/beta"), "{stdout}");
    assert!(stdout.contains("beta.example.com"), "{stdout}");
}

#[test]
fn the_flag_beats_the_env_var() {
    let sandbox = Sandbox::new(TWO_NODE_CONFIG);

    let output = sandbox
        .command()
        .env("SEISMIC_CONTEXT", "devnet-1/beta")
        .args(["ctx", "env", "--context", "devnet-1/alpha"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("devnet-1/alpha"), "{stdout}");
    assert!(stdout.contains("alpha.example.com"), "{stdout}");
}

#[test]
fn a_network_only_context_needs_name() {
    let sandbox = Sandbox::new(NETWORK_ONLY_CONFIG);

    let output = sandbox.command().args(["ctx", "env"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("alpha"), "{stderr}");
    assert!(stderr.contains("beta"), "{stderr}");

    let output = sandbox
        .command()
        .args(["ctx", "env", "--name", "alpha"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("alpha.example.com"), "{stdout}");
    // The pin names the node it resolved to, not just the network, so the
    // next command in that shell needs no --name.
    assert!(
        stdout.contains("export SEISMIC_CONTEXT='devnet-1/alpha'"),
        "{stdout}"
    );
}

/// `node status`, resolved from the context with no `--node`: the target is
/// echoed to stderr before the node is reached, and the status the node
/// reports lands on stdout — the same separation `ctx env` depends on, now
/// checked for a `node` command.
///
/// Both selection shapes are exercised here rather than in a test apiece:
/// `NodeDescriptor::attestation_rpc_url` fixes the port at `:7878` — it is
/// not a parameter the way `FakeServer::serve`'s is — so a second test
/// reaching a node would race this one's server across nextest's parallel
/// processes. One server, one process, two queued responses is deterministic
/// instead.
#[test]
fn node_status_echoes_the_resolved_node_on_stderr_and_the_status_on_stdout() {
    let _server = FakeServer::serve_at(
        ATTESTATION_RPC_PORT,
        vec![
            (200, rpc_result(json!({"state": "idle"}))),
            (200, rpc_result(json!({"state": "idle"}))),
        ],
    );

    // A selection that already names its node.
    let sandbox = Sandbox::new(LOOPBACK_NODE_CONFIG);
    let output = sandbox
        .command()
        .args(["node", "status", "--once"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"state\":\"idle\""), "{stdout}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    // NodeArgs::load echoes the node's public RPC URL — the same value every
    // context-resolved `node` command echoes, regardless of which endpoint
    // that particular command goes on to call.
    assert!(
        stderr.contains("context devnet-1/alpha → https://alpha.example.com/rpc"),
        "{stderr}"
    );

    // A network-only selection, with --name supplying the node: the echo
    // names the node it resolved to, not merely the network the selection
    // spelled, so what the command acts on is what it says it acts on.
    let sandbox = Sandbox::new(NETWORK_ONLY_LOOPBACK_CONFIG);
    let output = sandbox
        .command()
        .args(["node", "status", "--once", "--name", "alpha"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("context devnet-1/alpha → https://alpha.example.com/rpc"),
        "{stderr}"
    );
}

/// One tab press, as the registration script makes it: the binary entered
/// through `COMPLETE`, handed the shell's words — the command name first,
/// like bash's `COMP_WORDS` — and the index of the one being completed, the
/// last. The candidates come back one per line.
fn complete(sandbox: &Sandbox, words: &[&str], extra_env: &[(&str, &str)]) -> Vec<String> {
    let mut command = sandbox.command();
    command
        .env("COMPLETE", "bash")
        .env("_CLAP_COMPLETE_INDEX", words.len().to_string())
        .arg("--")
        .arg("seismic-tee")
        .args(words);
    for (key, value) in extra_env {
        command.env(key, value);
    }
    let output = command.output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout.lines().map(str::to_string).collect()
}

/// The value candidates of a tab press: [`complete`] less the flags clap
/// offers alongside them on an empty word (`--config`, `--help`), which are
/// the static half's and not what these tests are about.
fn names(sandbox: &Sandbox, words: &[&str], extra_env: &[(&str, &str)]) -> Vec<String> {
    complete(sandbox, words, extra_env)
        .into_iter()
        .filter(|word| !word.starts_with("--"))
        .collect()
}

/// The dynamic half of the ticket: the names an operator struggles to type
/// come out of the context file — and nothing else in the sandbox exists for
/// the completer to have read.
#[test]
fn tab_completes_context_network_and_node_names_from_the_config_file() {
    let sandbox = Sandbox::new(TWO_NODE_CONFIG);

    // Static: a subcommand name, from the clap tree.
    assert_eq!(complete(&sandbox, &["no"], &[]), ["node"]);

    // `ctx use <TAB>`: every network and <network>/<node>. No `previous` in
    // the file, so no `-`.
    assert_eq!(
        names(&sandbox, &["ctx", "use", ""], &[]),
        ["devnet-1", "devnet-1/alpha", "devnet-1/beta"]
    );
    // The typed prefix narrows it.
    assert_eq!(
        names(&sandbox, &["ctx", "use", "devnet-1/b"], &[]),
        ["devnet-1/beta"]
    );
    // `--context <TAB>` is the same set.
    assert_eq!(
        names(&sandbox, &["node", "status", "--context", ""], &[]),
        ["devnet-1", "devnet-1/alpha", "devnet-1/beta"]
    );
    // `ctx set-nodes <TAB>`: the registered networks.
    assert_eq!(
        names(&sandbox, &["ctx", "set-nodes", ""], &[]),
        ["devnet-1"]
    );
    // `ctx list <TAB>`: the same, the network to narrow to.
    assert_eq!(names(&sandbox, &["ctx", "list", ""], &[]), ["devnet-1"]);
    // `--name <TAB>`: the selected network's nodes.
    assert_eq!(
        names(&sandbox, &["node", "status", "--name", ""], &[]),
        ["alpha", "beta"]
    );
    assert_eq!(
        names(
            &sandbox,
            &["network", "configure", "--genesis-node", ""],
            &[]
        ),
        ["alpha", "beta"]
    );
    // The shell's pinned selection scopes `--name` the way it scopes the
    // command; one that names no registered network falls back to every
    // node the file holds.
    assert_eq!(
        names(
            &sandbox,
            &["node", "verify", "--name", ""],
            &[("SEISMIC_CONTEXT", "devnet-1/beta")]
        ),
        ["alpha", "beta"]
    );
    assert_eq!(
        names(
            &sandbox,
            &["node", "verify", "--name", ""],
            &[("SEISMIC_CONTEXT", "elsewhere")]
        ),
        ["alpha", "beta"]
    );
}

/// An unreadable context file contributes no names rather than an error in
/// the prompt; the static half — flags and subcommands — still completes.
#[test]
fn a_broken_config_file_completes_to_nothing_quietly() {
    let sandbox = Sandbox::new("current = 3\nthis is not toml");
    let offered = complete(&sandbox, &["ctx", "use", ""], &[]);
    assert!(
        !offered.is_empty() && offered.iter().all(|word| word.starts_with('-')),
        "{offered:?}"
    );
    assert_eq!(complete(&sandbox, &["ctx", "us"], &[]), ["use"]);
}

/// `--completions <SHELL>` is the operator-facing spelling of the engine's
/// `COMPLETE=<shell> seismic-tee`: byte-identical output, registered under
/// the installed name and calling back to this binary by absolute path.
#[test]
fn completions_prints_the_engines_registration_script() {
    let sandbox = Sandbox::new(TWO_NODE_CONFIG);

    let verb = sandbox
        .command()
        .args(["--completions", "bash"])
        .output()
        .unwrap();
    assert!(verb.status.success(), "{verb:?}");
    let script = String::from_utf8(verb.stdout).unwrap();
    assert!(
        script.contains("-F _clap_complete_seismic_tee seismic-tee"),
        "{script}"
    );
    assert!(
        script.contains(env!("CARGO_BIN_EXE_seismic-tee")),
        "{script}"
    );

    let engine = sandbox.command().env("COMPLETE", "bash").output().unwrap();
    assert!(engine.status.success(), "{engine:?}");
    assert_eq!(String::from_utf8(engine.stdout).unwrap(), script);

    // The shell comes from $SHELL when not named, and is an error when it
    // cannot: the sandbox clears the environment.
    let zsh = sandbox
        .command()
        .env("SHELL", "/bin/zsh")
        .arg("--completions")
        .output()
        .unwrap();
    assert!(zsh.status.success(), "{zsh:?}");
    assert!(
        String::from_utf8(zsh.stdout).unwrap().contains("compdef"),
        "not a zsh script"
    );
    let unknown = sandbox.command().arg("--completions").output().unwrap();
    assert!(!unknown.status.success(), "{unknown:?}");
    let stderr = String::from_utf8(unknown.stderr).unwrap();
    assert!(stderr.contains("$SHELL"), "{stderr}");
}
