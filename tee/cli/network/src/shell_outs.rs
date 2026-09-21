//! The foreign-repo binaries a founding shells out to, and the seam that lets
//! a test stand in for them.
//!
//! Every artifact deploy derives from a sibling binary is derived by *running*
//! that binary, never by mirroring its logic here: the hash a node computes at
//! boot is the hash the node's own code computes.
//!
//! ```text
//! seismic-reth genesis-hash                eth.genesis_hash
//! summit genesis set-validators / digest   the completed summit genesis and
//!                                          summit.genesis_config_digest
//! ```
//!
//! These stay shell-outs under any design (the sorting rule): `seismic-reth`'s
//! `genesis_hash()` sits inside reth's `[patch.crates-io]` region — linking it
//! would mean reproducing that patch set here and tracking every rev bump, and
//! a miss would compute the hash against a different alloy pin than the node,
//! silently — and summit has no genesis library to link. A prebuilt binary
//! carries its own resolved graph. The enclave crates, by contrast, are linked
//! (see `gates`), so admission compilation and manifest rendering have no
//! counterpart here.
//!
//! Each method owns one subcommand's contract — argv, what travels on
//! stdin/stdout, what a failure means — and every failure names the command
//! that produced it. [`Derivations`] is the trait the gates and `assemble` are
//! written against, so a test can hand them a stand-in and never need the
//! binaries; [`ShellOuts`] is the one real implementation.

use std::io::Write as _;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context as _, bail};
use clap::Args;

use crate::founding::Validator;

/// The execution client. Its `genesis-hash` subcommand parses a genesis file
/// down the same path `seismic-reth node --chain` takes, so the hash it prints
/// is the one a node booted from that file computes.
pub const DEFAULT_RETH_BIN: &str = "seismic-reth";

/// Summit's node binary. Its `genesis digest` subcommand computes
/// `summit.genesis_config_digest`: SHA-256 over summit's domain-prefixed SSZ
/// serialization of the complete genesis — summit's own definition of chain
/// identity (its P2P and signing domains derive from it). Its `genesis
/// set-validators` subcommand emits the completed genesis the digest is
/// computed over.
pub const DEFAULT_SUMMIT_BIN: &str = "summit";

/// How long any one shell-out may run. Both binaries answer in well under a
/// second; a hang is a broken install, not a slow one.
pub const SHELL_OUT_TIMEOUT: Duration = Duration::from_secs(120);

/// The derivations a founding needs from the sibling binaries.
///
/// Every method takes the artifact as bytes: `assemble` derives from copies
/// that exist only in memory until the artifact set is written (or, under
/// `--check`, compared and never written), and the drift suite reads a
/// committed set back from disk — one interface for both.
pub trait Derivations {
    /// `eth.genesis_hash` for this reth genesis.
    fn reth_genesis_hash(&self, genesis: &[u8]) -> impl Future<Output = anyhow::Result<[u8; 32]>>;

    /// `summit.genesis_config_digest` for this complete summit genesis. The
    /// file is loaded down the same parse path a starting validator takes, so
    /// a digest doubles as a verdict that the genesis is well formed.
    fn summit_config_digest(
        &self,
        genesis: &[u8],
    ) -> impl Future<Output = anyhow::Result<[u8; 32]>>;

    /// The completed summit genesis: `template` with its validator set
    /// replaced by `validators`, rendered canonically by summit's own emitter.
    fn summit_set_validators(
        &self,
        template: &[u8],
        validators: &[Validator],
    ) -> impl Future<Output = anyhow::Result<Vec<u8>>>;
}

/// The `--reth-bin` / `--summit-bin` flags every command that derives carries.
#[derive(Debug, Clone, Args)]
pub struct DerivationArgs {
    /// seismic-reth binary whose `genesis-hash` subcommand computes
    /// eth.genesis_hash.
    #[arg(long, value_name = "BIN", default_value = DEFAULT_RETH_BIN)]
    pub reth_bin: String,

    /// summit binary whose `genesis digest` subcommand computes
    /// summit.genesis_config_digest (and `genesis set-validators` emits the
    /// completed genesis).
    #[arg(long, value_name = "BIN", default_value = DEFAULT_SUMMIT_BIN)]
    pub summit_bin: String,
}

impl DerivationArgs {
    pub fn shell_outs(&self) -> ShellOuts {
        ShellOuts {
            reth_bin: self.reth_bin.clone(),
            summit_bin: self.summit_bin.clone(),
        }
    }
}

/// The real derivations: the two binaries, run as subprocesses.
#[derive(Debug, Clone)]
pub struct ShellOuts {
    pub reth_bin: String,
    pub summit_bin: String,
}

impl Default for ShellOuts {
    fn default() -> Self {
        Self {
            reth_bin: DEFAULT_RETH_BIN.to_string(),
            summit_bin: DEFAULT_SUMMIT_BIN.to_string(),
        }
    }
}

impl Derivations for ShellOuts {
    /// `seismic-reth genesis-hash --chain <file>`: one 0x-hex digest on stdout.
    async fn reth_genesis_hash(&self, genesis: &[u8]) -> anyhow::Result<[u8; 32]> {
        let file = temp_file(genesis, ".json")?;
        let argv = [
            self.reth_bin.as_str(),
            "genesis-hash",
            "--chain",
            path_str(file.path()),
        ];
        let stdout = run(
            &argv,
            "build seismic-reth (the genesis-hash subcommand) or pass --reth-bin",
        )
        .await?;
        parse_digest(&stdout, &argv)
    }

    /// `summit genesis digest <file>`: one 0x-hex digest on stdout.
    async fn summit_config_digest(&self, genesis: &[u8]) -> anyhow::Result<[u8; 32]> {
        let file = temp_file(genesis, ".toml")?;
        let argv = [
            self.summit_bin.as_str(),
            "genesis",
            "digest",
            path_str(file.path()),
        ];
        let stdout = run(
            &argv,
            "build summit (the `genesis digest` subcommand) or pass --summit-bin",
        )
        .await?;
        parse_digest(&stdout, &argv)
    }

    /// `summit genesis set-validators -i <template> -v <validators.json>`: the
    /// completed genesis on stdout.
    ///
    /// Emission belongs to summit: the subcommand parses the template into
    /// summit's own Genesis type, replaces its validator set, sorts them by
    /// node key (the order config_digest hashes), renders the whole file
    /// canonically — summit's hex spellings, no authored comments — and
    /// reloads what it emits, so a genesis no node could load fails here
    /// rather than at boot. The returned bytes are what the artifact set ships
    /// and the manifest's digest commits to.
    async fn summit_set_validators(
        &self,
        template: &[u8],
        validators: &[Validator],
    ) -> anyhow::Result<Vec<u8>> {
        let template_file = temp_file(template, ".toml")?;
        let mut validators_json =
            serde_json::to_vec_pretty(validators).context("rendering the validator set")?;
        validators_json.push(b'\n');
        let validators_file = temp_file(&validators_json, ".json")?;
        let argv = [
            self.summit_bin.as_str(),
            "genesis",
            "set-validators",
            "-i",
            path_str(template_file.path()),
            "-v",
            path_str(validators_file.path()),
        ];
        let stdout = run(
            &argv,
            "build summit (the `genesis set-validators` subcommand) or pass --summit-bin",
        )
        .await?;
        if stdout.is_empty() {
            bail!("`{}` emitted nothing on stdout", argv.join(" "));
        }
        Ok(stdout)
    }
}

/// Materialize bytes for a path-taking subcommand. Dropped with the guard.
fn temp_file(bytes: &[u8], suffix: &str) -> anyhow::Result<tempfile::NamedTempFile> {
    let mut file = tempfile::Builder::new()
        .suffix(suffix)
        .tempfile()
        .context("creating a temporary file for a shell-out")?;
    file.write_all(bytes)
        .and_then(|()| file.flush())
        .context("writing a temporary file for a shell-out")?;
    Ok(file)
}

fn path_str(path: &Path) -> &str {
    path.to_str().expect("a temporary file path is valid UTF-8")
}

/// Run `argv` to completion and return its stdout. A missing binary is
/// reported with `not_found_hint`; a nonzero exit with the command's stderr; a
/// hang past [`SHELL_OUT_TIMEOUT`] as a timeout.
async fn run(argv: &[&str], not_found_hint: &str) -> anyhow::Result<Vec<u8>> {
    let (bin, args) = argv.split_first().expect("argv has a binary");
    let command = argv.join(" ");
    let child = tokio::process::Command::new(bin)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn();
    let child = match child {
        Ok(child) => child,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bail!("{bin:?} not found; {not_found_hint}")
        }
        Err(error) => return Err(error).with_context(|| format!("running `{command}`")),
    };
    let output = match tokio::time::timeout(SHELL_OUT_TIMEOUT, child.wait_with_output()).await {
        Ok(output) => output.with_context(|| format!("running `{command}`"))?,
        Err(_) => bail!("`{command}` timed out"),
    };
    if !output.status.success() {
        let detail = if output.stderr.is_empty() {
            &output.stdout
        } else {
            &output.stderr
        };
        bail!(
            "`{command}` failed: {}",
            String::from_utf8_lossy(detail).trim()
        );
    }
    Ok(output.stdout)
}

/// A digest a sibling binary printed on stdout: 32-byte 0x-hex, either case.
fn parse_digest(stdout: &[u8], argv: &[&str]) -> anyhow::Result<[u8; 32]> {
    let text = String::from_utf8_lossy(stdout);
    let text = text.trim();
    let digits = text.strip_prefix("0x").unwrap_or("");
    let mut digest = [0u8; 32];
    if digits.len() != 64 || hex::decode_to_slice(digits, &mut digest).is_err() {
        bail!(
            "`{}` output: expected 32-byte hex string, got {text:?}",
            argv.join(" ")
        );
    }
    Ok(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digests_are_32_byte_hex_in_either_case() {
        let argv = ["summit", "genesis", "digest", "g.toml"];
        let upper = format!("0x{}\n", "AB".repeat(32));
        assert_eq!(parse_digest(upper.as_bytes(), &argv).unwrap(), [0xab; 32]);

        for bad in ["0xabcd", &"ab".repeat(32), "not a hash", ""] {
            let err = parse_digest(bad.as_bytes(), &argv).unwrap_err().to_string();
            assert!(err.contains("expected 32-byte hex string"), "{err}");
            assert!(err.contains("summit genesis digest"), "{err}");
        }
    }

    /// A binary that is not there is named with the flag that points at
    /// another, before anything else can go wrong.
    #[tokio::test]
    async fn a_missing_binary_is_named_with_its_flag() {
        let shell_outs = ShellOuts {
            reth_bin: "/absent/seismic-reth".into(),
            summit_bin: "/absent/summit".into(),
        };
        let err = shell_outs
            .reth_genesis_hash(b"{}")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("/absent/seismic-reth"), "{err}");
        assert!(err.contains("--reth-bin"), "{err}");

        let err = shell_outs
            .summit_config_digest(b"")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("--summit-bin"), "{err}");

        let err = shell_outs
            .summit_set_validators(b"", &[])
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("set-validators"), "{err}");
    }

    /// A nonzero exit is the command's stderr, named by the command.
    #[tokio::test]
    async fn a_failure_carries_the_commands_stderr() {
        let err = run(&["sh", "-c", "echo boom >&2; exit 3"], "unused")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("failed: boom"), "{err}");
        assert!(err.contains("`sh -c"), "{err}");

        let stdout = run(&["sh", "-c", "printf hello"], "unused").await.unwrap();
        assert_eq!(stdout, b"hello");
    }

    /// The validator set travels to summit as the JSON array its
    /// `set-validators` reads, one object per validator with the four fields.
    #[test]
    fn validators_serialize_as_summit_reads_them() {
        let validator = Validator {
            node_public_key: "ab".repeat(32),
            consensus_public_key: "cd".repeat(48),
            ip_address: "203.0.113.7:18551".into(),
            withdrawal_credentials: format!("0x{}", "f3".repeat(20)),
        };
        let json: serde_json::Value = serde_json::to_value(vec![validator]).unwrap();
        assert_eq!(json[0]["node_public_key"], "ab".repeat(32));
        assert_eq!(json[0]["ip_address"], "203.0.113.7:18551");
        assert_eq!(json[0].as_object().unwrap().len(), 4);
    }
}
