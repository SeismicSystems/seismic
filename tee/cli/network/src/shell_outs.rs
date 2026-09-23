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
//! Which build of each binary matters as much as running it: the digest the
//! manifest pins must come from the code the nodes run. For a directory
//! `init --image` made, [`DerivationArgs::resolve`] runs the image's own
//! binaries, fetched from its seismic-images release and verified against
//! the release's `SHA256SUMS` ([`crate::image`]); `--reth-bin` /
//! `--summit-bin` point elsewhere for a host that cannot run them.
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
use seismic_tee_common::NetworkDir;

use crate::founding::Validator;
use crate::image::{self, ImageRecord};

/// The execution client. Its `genesis-hash` subcommand parses a genesis file
/// down the same path `seismic-reth node --chain` takes, so the hash it prints
/// is the one a node booted from that file computes. This is the PATH name
/// [`ShellOuts::default`] runs — the drift suite's binaries; a founding runs
/// the image's own instead ([`DerivationArgs::resolve`]).
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
///
/// Neither is needed for a directory `init --image` made: the derivations
/// then run the image's own binaries, fetched from its seismic-images
/// release and verified against the release's `SHA256SUMS`, so the hash a
/// node computes at boot is computed here by the very same bytes. The flags
/// are for a host that cannot run them (they are x86-64 Linux) and for a
/// directory scaffolded from loose files.
#[derive(Debug, Clone, Args)]
pub struct DerivationArgs {
    /// seismic-reth binary whose `genesis-hash` subcommand computes
    /// eth.genesis_hash. Default: the image's own, from the seismic-images
    /// release inputs/image.json names, verified against its SHA256SUMS,
    /// whose build provenance gh verifies first.
    #[arg(long, value_name = "BIN")]
    pub reth_bin: Option<String>,

    /// summit binary whose `genesis digest` subcommand computes
    /// summit.genesis_config_digest (and `genesis set-validators` emits the
    /// completed genesis). Default: the image's own, as for --reth-bin.
    #[arg(long, value_name = "BIN")]
    pub summit_bin: Option<String>,
}

impl DerivationArgs {
    /// The binaries this founding derives with: each flag as given, else the
    /// image's own from its release, cached under
    /// [`crate::image::default_cache_dir`] and re-verified on every use.
    pub async fn resolve(&self, dir: &NetworkDir) -> anyhow::Result<ShellOuts> {
        if let (Some(reth_bin), Some(summit_bin)) = (&self.reth_bin, &self.summit_bin) {
            return Ok(ShellOuts {
                reth_bin: reth_bin.clone(),
                summit_bin: summit_bin.clone(),
            });
        }
        let record = ImageRecord::read(dir)?;
        let release = record.release();
        if let Err(why) = image::host_runs_image_binaries() {
            bail!(
                "image {}'s own seismic-reth and summit cannot run here: {why}. The derivations \
                 must still be theirs — put both on PATH, built at the revs inputs/image.json \
                 pins under `sources`, and pass --reth-bin seismic-reth --summit-bin summit",
                release.tag()
            );
        }
        let cache = image::default_cache_dir()?;
        let client = image::download_client()?;
        let sums = image::fetch_sums(&client, &release).await?;
        let mut resolved = Vec::with_capacity(2);
        for (given, asset) in [
            (&self.reth_bin, image::RETH_BIN_ASSET),
            (&self.summit_bin, image::SUMMIT_BIN_ASSET),
        ] {
            resolved.push(match given {
                Some(bin) => bin.clone(),
                None => image::fetch_binary(&client, &release, &sums, asset, &cache)
                    .await?
                    .display()
                    .to_string(),
            });
        }
        let [reth_bin, summit_bin] = <[String; 2]>::try_from(resolved).expect("two binaries");
        eprintln!(
            "deriving with image {}'s own binaries, verified against its release's attested \
             SHA256SUMS: {reth_bin}, {summit_bin}",
            release.tag()
        );
        Ok(ShellOuts {
            reth_bin,
            summit_bin,
        })
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
pub(crate) fn temp_file(bytes: &[u8], suffix: &str) -> anyhow::Result<tempfile::NamedTempFile> {
    let mut file = tempfile::Builder::new()
        .suffix(suffix)
        .tempfile()
        .context("creating a temporary file for a shell-out")?;
    file.write_all(bytes)
        .and_then(|()| file.flush())
        .context("writing a temporary file for a shell-out")?;
    Ok(file)
}

pub(crate) fn path_str(path: &Path) -> &str {
    path.to_str().expect("a temporary file path is valid UTF-8")
}

/// Run `argv` to completion and return its stdout. A missing binary is
/// reported with `not_found_hint`; a nonzero exit with the command's stderr; a
/// hang past [`SHELL_OUT_TIMEOUT`] as a timeout.
pub(crate) async fn run(argv: &[&str], not_found_hint: &str) -> anyhow::Result<Vec<u8>> {
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

    /// Both flags given: no record, no release, no network — the flags are
    /// the answer. One or none given: the directory must carry the record
    /// `init --image` leaves, and the error says so with the way out.
    #[tokio::test]
    async fn the_flags_win_outright_and_a_recordless_directory_is_named() {
        let dir = tempfile::tempdir().unwrap();
        let network = NetworkDir::new(dir.path());
        let both = DerivationArgs {
            reth_bin: Some("/x/seismic-reth".into()),
            summit_bin: Some("/x/summit".into()),
        };
        let resolved = both.resolve(&network).await.unwrap();
        assert_eq!(resolved.reth_bin, "/x/seismic-reth");
        assert_eq!(resolved.summit_bin, "/x/summit");

        let one = DerivationArgs {
            reth_bin: Some("/x/seismic-reth".into()),
            summit_bin: None,
        };
        let err = one.resolve(&network).await.unwrap_err().to_string();
        assert!(err.contains("image.json not found"), "{err}");
        assert!(err.contains("init --image"), "{err}");
        assert!(err.contains("--summit-bin"), "{err}");
    }

    /// With the record in place, the missing binary comes from the release —
    /// on a host that can run it; elsewhere the error names the host and the
    /// flags. Either way the given flag is kept as given.
    #[tokio::test]
    async fn a_missing_binary_comes_from_the_images_release() {
        use crate::image::tests::{TAG, release_files, serve_release};

        let dir = tempfile::tempdir().unwrap();
        let network = NetworkDir::new(dir.path().join("net"));
        std::fs::create_dir_all(network.inputs()).unwrap();
        let files = release_files(TAG);
        std::fs::write(network.input_image(), &files[image::IMAGE_JSON_ASSET]).unwrap();
        let (_server, release) = serve_release(TAG, files);
        // The record names GitHub; the test's release is local. Resolve the
        // way `resolve` does, against the local one.
        let sums = image::fetch_sums(&image::download_client().unwrap(), &release)
            .await
            .unwrap();
        let cache = tempfile::tempdir().unwrap();
        let summit = image::fetch_binary(
            &image::download_client().unwrap(),
            &release,
            &sums,
            image::SUMMIT_BIN_ASSET,
            cache.path(),
        )
        .await
        .unwrap();
        // The fetched "binary" is a script printing a digest: the shell-out
        // contract holds end to end over a fetched file.
        let shell_outs = ShellOuts {
            reth_bin: "unused".into(),
            summit_bin: summit.display().to_string(),
        };
        assert_eq!(
            shell_outs.summit_config_digest(b"").await.unwrap(),
            [0x22; 32]
        );

        // And `resolve` itself against the real record: on a host that
        // cannot run x86-64 Linux binaries the refusal names the host; on one
        // that can, it would reach GitHub, which a unit test does not.
        if let Err(why) = image::host_runs_image_binaries() {
            let args = DerivationArgs {
                reth_bin: None,
                summit_bin: Some("/x/summit".into()),
            };
            let err = args.resolve(&network).await.unwrap_err().to_string();
            assert!(err.contains(&why), "{err}");
            assert!(err.contains(TAG), "{err}");
            assert!(err.contains("--reth-bin"), "{err}");
        }
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
