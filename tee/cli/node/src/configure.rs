//! `configure`: deliver provisioned nodes their config on first boot.
//!
//! ```text
//! seismic-tee node configure
//! ```
//!
//! One command, no mode flag: the network's live state decides what it does
//! ([`crate::plan`]). Every node in the node table is probed; a network whose
//! pinned box awaits its config still holding the pinned candidate is
//! unfounded, and the run founds it — the pinned box first and alone, then
//! the rest through it. A network with a key holder is live, and every node
//! named joins it through the key holders that pass their appraisal
//! ([`crate::bootnodes`]), or through the peers `--bootnode` names. A node
//! that took its config this boot is skipped; one not up yet is waited for.
//! Both runs are [`crate::cohort`]'s: one dashboard, each node appraised as
//! it comes up, and the founding's launch assertions over every node with a
//! harvest record. `--check` runs those assertions alone.
//!
//! There is no per-node `node.toml`: `[node]` (external_ip) comes from the
//! descriptor, and `[network]` from `--manifest`, `--reth-genesis`, `--summit-genesis` and the
//! bootnodes. Those network-wide artifacts stay standalone files, merged only
//! at POST time. The config is built as the [`InitConfig`] type tdx-init
//! deserializes it with (`deny_unknown_fields` at the far end), so this CLI
//! and the node image cannot disagree on the shape without failing to
//! compile. [`build_config`] and [`post_config_within`] are the primitives
//! every node goes through.
//!
//! The POST is the one irreversible step an operator takes (tdx-init accepts
//! one per boot), so the run is gated: a decoded preview of what it is about
//! to do is shown and the terminal is asked, once per run. The TOML itself is
//! three base64 blobs, so the preview says what they are: which manifest
//! (name, chain, network id), which genesis files (by hash), which bootnodes,
//! which policy will appraise the nodes. `--yes` sends unattended; with no
//! terminal and no `--yes` nothing is sent. Each node's rendered TOML is
//! written to `nodes/<name>.init-config.toml` beside `--manifest` before its
//! POST, as the record of what it booted with and a body `curl` can replay.

use std::cell::OnceCell;
use std::io::BufRead;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use base64::Engine as _;
use clap::Args;
use clap_complete::ArgValueCandidates;
use seismic_tee_common::{Artifact, Descriptors, Manifest, NetworkDir};
use seismic_tee_context::{Context, Selected, complete, echo, read_nodes};
use sha2::{Digest as _, Sha256};
use tdx_init_config::{Bootnode, InitConfig, NetworkConfig, NodeConfig};

use crate::args::CohortArgs;
use crate::cohort;
use crate::launch;
use crate::plan::{Remedy, Run, Through};
use crate::verify::{PolicySourceArgs, VerifierArgs, retry_flags};

/// How long to wait for tdx-init's HTTP listener to come up. tdx-init starts
/// after persistent-luks-setup, which can take ~20-40s on first boot (LUKS
/// format + mkfs + TPM enroll).
pub const TDX_INIT_LISTENER_TIMEOUT: Duration = Duration::from_secs(180);
pub const TDX_INIT_RETRY_INTERVAL: Duration = Duration::from_secs(5);

/// Resolve `--reth-genesis`, defaulting to the artifact-set convention:
/// `reth-genesis.json` beside the manifest, exactly where `assemble` writes
/// its byte-verbatim copy — so the file POSTed is the one the manifest's
/// `eth.genesis_hash` was computed from.
pub fn resolve_reth_genesis(flag: Option<&Path>, manifest_path: &Path) -> anyhow::Result<PathBuf> {
    resolve_beside_manifest(
        flag,
        NetworkDir::of_manifest(manifest_path).reth_genesis(),
        "reth genesis",
        "--reth-genesis",
    )
}

/// Resolve `--summit-genesis`, defaulting to the artifact-set convention:
/// `summit-genesis.toml` beside the manifest, exactly where `assemble` writes
/// its byte-verbatim copy — so the file POSTed is the one the manifest's
/// `summit.genesis_config_digest` was computed from.
pub fn resolve_summit_genesis(
    flag: Option<&Path>,
    manifest_path: &Path,
) -> anyhow::Result<PathBuf> {
    resolve_beside_manifest(
        flag,
        NetworkDir::of_manifest(manifest_path).summit_genesis(),
        "summit genesis",
        "--summit-genesis",
    )
}

fn resolve_beside_manifest(
    flag: Option<&Path>,
    default: PathBuf,
    what: &str,
    flag_name: &str,
) -> anyhow::Result<PathBuf> {
    let path = flag.map_or(default, Path::to_path_buf);
    if path.is_file() {
        return Ok(path);
    }
    let hint = if flag.is_some() {
        String::new()
    } else {
        format!(
            " (the default is {} beside --manifest; pass {flag_name} if it lives elsewhere)",
            path.file_name().unwrap_or_default().to_string_lossy(),
        )
    };
    bail!("{what} not found: {}{hint}", path.display())
}

/// Everything a node's config is built from.
///
/// The network-wide half — the manifest and the two genesis files it pins,
/// plus the bootnode set — is the same for every node configured at one
/// moment; the rest is this node only.
#[derive(Debug, Clone)]
pub struct ConfigInputs<'a> {
    pub manifest: &'a Manifest,
    /// The reth genesis, as the file it came from: named on a failed check.
    pub reth_genesis: &'a Artifact,
    /// The summit genesis, likewise. A founding delivers it with current
    /// validator IPs spliced in, which is why it is bytes-with-a-path rather
    /// than a path.
    pub summit_genesis: &'a Artifact,
    /// The enode set → `[network].bootnodes`. Empty is valid only on the
    /// pinned box of a founding.
    pub bootnodes: &'a [Bootnode],
    /// The node's own public IP → `[node].external_ip` (reth's `--nat extip`).
    pub external_ip: IpAddr,
    /// Only ever `true` for the pinned box of a founding. Not sent: tdx-init
    /// tells the pinned box from the manifest.
    pub pinned_box: bool,
}

/// Assemble the config POSTed to tdx-init, mutating no source.
///
/// Written fresh from its inputs, so there is no operator-supplied TOML that
/// could carry a conflicting `[node]`/`[network]` and fork the network. Each
/// field arrives parsed, so what is left are the cross-field checks tdx-init
/// would fail the POST on, run here first so a bad input fails while the fix
/// still costs nothing:
///
/// - A joiner needs at least one bootnode: tdx-init derives its root_key
///   fetch peers from them, so none means no way to bootstrap.
/// - The manifest's gates over the two genesis files, naming the file that
///   fails.
///
/// The artifacts travel base64-encoded so they stay opaque through the TOML
/// hop: tdx-init decodes and writes these exact bytes verbatim.
pub fn build_config(inputs: &ConfigInputs<'_>) -> anyhow::Result<InitConfig> {
    if !inputs.pinned_box && inputs.bootnodes.is_empty() {
        bail!(
            "build_config: a joining node needs at least one bootnode (tdx-init derives its \
             root_key fetch peers from them)"
        );
    }
    inputs
        .manifest
        .check_reth_genesis(inputs.reth_genesis.bytes())
        .with_context(|| format!("--reth-genesis {}", inputs.reth_genesis.path().display()))?;
    inputs
        .manifest
        .check_summit_genesis(inputs.summit_genesis.bytes())
        .with_context(|| {
            format!(
                "--summit-genesis {}",
                inputs.summit_genesis.path().display()
            )
        })?;

    let base64 = base64::engine::general_purpose::STANDARD;
    Ok(InitConfig {
        network: NetworkConfig {
            manifest_base64: base64.encode(inputs.manifest.bytes()),
            reth_genesis_base64: base64.encode(inputs.reth_genesis.bytes()),
            summit_genesis_base64: base64.encode(inputs.summit_genesis.bytes()),
            bootnodes: inputs.bootnodes.to_vec(),
        },
        node: NodeConfig {
            external_ip: inputs.external_ip,
        },
    })
}

/// The config as the TOML document tdx-init receives.
pub fn render_config(config: &InitConfig) -> anyhow::Result<String> {
    toml::to_string(config).context("rendering the tdx-init config as TOML")
}

/// Write the rendered config to `path`, creating its directory. Done before
/// the POST, so the record exists whatever comes of it — accepted, or
/// rejected by tdx-init.
pub fn write_record(path: &Path, rendered: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {} for the config record", parent.display()))?;
    }
    std::fs::write(path, rendered)
        .with_context(|| format!("writing the config record {}", path.display()))
}

/// What a `configure` run is about to do, decoded for a human.
pub struct Preview<'a> {
    pub table: &'a TableSource,
    pub manifest_path: &'a Path,
    pub manifest: &'a Manifest,
    pub reth_genesis: &'a Artifact,
    /// As delivered: spliced, in a founding.
    pub summit_genesis: &'a Artifact,
    /// The policy's provenance, or `None` under `--no-verify`.
    pub policy: Option<&'a str>,
    pub run: &'a Run,
    /// The nodes named that took their config this boot.
    pub skipped: &'a [String],
    /// What a joining run hands every node; empty in a founding, whose
    /// joiners get the pinned box's enode once it is up.
    pub bootnodes: &'a [Bootnode],
    /// Where the rendered TOML of a node `<name>` is written.
    pub record: &'a Path,
}

/// The preview as text. The rendered TOML carries the manifest and both
/// genesis files as base64, so the review names them by path and hash and
/// spells out what a wrong flag would change.
pub fn render_preview(preview: &Preview<'_>) -> String {
    let Preview {
        table,
        manifest_path,
        manifest,
        reth_genesis,
        summit_genesis,
        policy,
        run,
        skipped,
        bootnodes,
        record,
    } = preview;
    let sha256 = |bytes: &[u8]| hex::encode(Sha256::digest(bytes));
    let mut lines = vec![match run {
        Run::Found { pinned, .. } => format!(
            "This founds network {:?} (node table: {table}); pinned box {}.",
            manifest.name, pinned.name
        ),
        run => format!(
            "This joins {} node(s) to network {:?} (node table: {table}).",
            run.names().len(),
            manifest.name
        ),
    }];
    let mut field = |label: &str, value: String| lines.push(format!("  {label:<18}{value}"));
    field("Manifest:", manifest_path.display().to_string());
    field(
        "",
        format!(
            "chain_id {}, network_id {}",
            manifest.eth.chain_id,
            manifest.network_id()
        ),
    );
    field(
        "reth genesis:",
        format!(
            "{}  sha256 {}",
            reth_genesis.path().display(),
            sha256(reth_genesis.bytes())
        ),
    );
    field(
        "summit genesis:",
        format!(
            "{}  sha256 {}",
            summit_genesis.path().display(),
            sha256(summit_genesis.bytes())
        ),
    );
    field(
        "Appraise against:",
        policy.unwrap_or("nothing (--no-verify)").to_string(),
    );
    match run {
        Run::Found { pinned, rest } => {
            field(
                "Nodes:",
                format!(
                    "{}, first and alone: keeps its candidate root_key",
                    pinned.name
                ),
            );
            if !rest.is_empty() {
                let rest: Vec<&str> = rest.iter().map(|node| node.name.as_str()).collect();
                field(
                    "",
                    format!(
                        "then {}: fetch root_key from {}",
                        rest.join(", "),
                        pinned.name
                    ),
                );
            }
        }
        Run::Join { nodes, through } => {
            let names: Vec<&str> = nodes.iter().map(|node| node.name.as_str()).collect();
            field("Nodes:", names.join(", "));
            let source = match through {
                Through::NamedPeers => "named by --bootnode",
                Through::KeyHolders(_) => "key holder(s) that passed their appraisal",
            };
            field("Bootnodes:", format!("{}, {source}", bootnodes.len()));
            for bootnode in *bootnodes {
                field("", bootnode.to_string());
            }
        }
    }
    let not_up = run.not_up();
    if !not_up.is_empty() {
        field(
            "Not up yet:",
            format!(
                "{} (waited for, up to {}s)",
                not_up.join(", "),
                TDX_INIT_LISTENER_TIMEOUT.as_secs()
            ),
        );
    }
    if !skipped.is_empty() {
        field(
            "Skipped:",
            format!("{} (took its config this boot)", skipped.join(", ")),
        );
    }
    field("Records:", record.display().to_string());
    lines.join("\n") + "\n"
}

/// The gate before the one irreversible step. `yes` (`--yes`) sends without
/// asking. Otherwise a terminal is asked and anything but `y`/`yes` declines,
/// and no terminal is an error — a script that means it says so with the
/// flag, rather than a redirected stdin deciding silently.
pub fn confirm_run(yes: bool, interactive: bool, input: &mut impl BufRead) -> anyhow::Result<bool> {
    if yes {
        return Ok(true);
    }
    if !interactive {
        bail!(
            "stdin is not a terminal, so nobody is here to confirm; pass --yes to send unattended. \
             Nothing was sent."
        );
    }
    eprint!("Continue? [y/N] ");
    let mut answer = String::new();
    input
        .read_line(&mut answer)
        .context("reading the confirmation")?;
    let answer = answer.trim();
    Ok(answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes"))
}

/// Wait for tdx-init's HTTP listener at `url` to come up, then POST the
/// config, reporting progress through `report` — a line at a time, so the
/// dashboard can show it as the node's status. tdx-init validates the schema
/// server-side and 4xx-rejects a malformed payload, so any non-200 is an
/// error — 4xx = a config it rejects, 5xx = a tdx-init bug; neither benefits
/// from retry. `url` is the node's receiver
/// ([`seismic_tee_common::NodeDescriptor::tdx_init_url`]).
pub async fn post_config_within(
    client: &reqwest::Client,
    url: &str,
    config: &InitConfig,
    listener_timeout: Duration,
    retry_interval: Duration,
    mut report: impl FnMut(&str),
) -> anyhow::Result<()> {
    let body = render_config(config)?;

    report(&format!("Waiting for tdx-init listener at {url}..."));
    let deadline = Instant::now() + listener_timeout;
    let mut last_error = None;
    while Instant::now() < deadline {
        let response = match client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/toml")
            .body(body.clone())
            .send()
            .await
        {
            // The listener isn't up yet: the expected state for the first
            // minute or so of a boot. Covers a refused connection and a
            // hanging one alike — the shared client's connect timeout
            // (`http::CONNECT_TIMEOUT`) reports the latter as a connect
            // error too, so a node whose network is still coming up is
            // waited for rather than given up on. A timeout *after* the
            // connect is not retried: the POST may have landed, and tdx-init
            // takes one per boot.
            Err(error) if error.is_connect() => {
                last_error = Some(error);
                tokio::time::sleep(retry_interval).await;
                continue;
            }
            Err(error) => {
                return Err(error).with_context(|| format!("POSTing the config to {url}"));
            }
            Ok(response) => response,
        };
        let status = response.status();
        if status == reqwest::StatusCode::OK {
            report("tdx-init accepted the config.");
            return Ok(());
        }
        let text = response.text().await.unwrap_or_default();
        bail!("tdx-init rejected config: {} {text}", status.as_u16());
    }
    bail!(
        "tdx-init listener at {url} never came up after {}s (last error: {})",
        listener_timeout.as_secs(),
        last_error.map_or_else(|| "none".to_string(), |e| e.to_string()),
    )
}

#[derive(Debug, Args)]
#[command(after_help = "Examples:\n  \
    seismic-tee node configure                  every node in the selected network: found it, or join it\n  \
    seismic-tee node configure beta gamma       only these nodes\n  \
    seismic-tee node configure --yes            without asking, for a script or CI\n  \
    seismic-tee node configure --check          re-assert a founded cohort's launch\n  \
    seismic-tee node configure op-1 --bootnode \"$(scast rpc seismic_nodeInfo --rpc-url https://peer.example.com/rpc | jq -r .enode)\"\n  \
    seismic-tee node configure --node nodes.json --manifest network-manifest.json")]
pub struct ConfigureArgs {
    /// The nodes to configure, by name in the node table. Default: every
    /// node in it. A node that took its config this boot is skipped, and one
    /// not up yet is waited for.
    #[arg(value_name = "NAME", add = ArgValueCandidates::new(complete::nodes))]
    pub names: Vec<String>,

    #[command(flatten)]
    pub cohort: CohortArgs,

    /// Network manifest JSON (from `seismic-tee network assemble`). Merged
    /// into the POSTed config as [network].manifest_base64; shared across
    /// every node. Omit it to use the current context's network.
    #[arg(long, value_name = "FILE")]
    pub manifest: Option<PathBuf>,

    /// reth genesis JSON POSTed to each node as [network].reth_genesis_base64;
    /// tdx-init writes it to /run/seismic/conf/reth-genesis.json for reth's
    /// --chain. Must be the file the manifest's eth.genesis_hash was computed
    /// from. Default: reth-genesis.json beside --manifest (the artifact-set
    /// layout `assemble` produces).
    #[arg(long, value_name = "FILE")]
    pub reth_genesis: Option<PathBuf>,

    /// summit genesis TOML POSTed to each node as
    /// [network].summit_genesis_base64; tdx-init writes it to
    /// /run/seismic/conf/summit-genesis.toml for summit's --genesis-path.
    /// Must be the file the manifest's summit.genesis_config_digest was
    /// computed from; founding splices each validator's current IP into it.
    /// Default: summit-genesis.toml beside --manifest (the artifact-set layout
    /// `assemble` produces).
    #[arg(long, value_name = "FILE")]
    pub summit_genesis: Option<PathBuf>,

    /// Join through this peer's enode URL (enode://<pubkey>@<host>:<port>)
    /// → [network].bootnodes. reth dials it on startup, and tdx-init derives
    /// the root_key fetch peer from it (http://<host>:7878). Repeatable.
    /// Default: the enode of every node in the node table that holds root_key
    /// (serves :7878) and passes its deploy verification, under --no-verify
    /// too; with none, nothing is sent. Name a peer when the table holds no
    /// key holder — your own nodes joining someone else's network — from its
    /// seismic_nodeInfo RPC. Refused while the network is unfounded.
    #[arg(long, value_name = "ENODE")]
    pub bootnode: Vec<Bootnode>,

    /// Configure without deploy-verifying the nodes configured. By default
    /// each node is appraised once it is up — the same check as `seismic-tee
    /// node verify`, against the policy --manifest pins — and a node that
    /// fails counts as failed. What the run itself relies on is checked
    /// either way: the pinned box's quote before a founding, the key holders
    /// before a join.
    #[arg(long)]
    pub no_verify: bool,

    /// Send without asking. Without it, configure shows what it is about to
    /// do and waits for a `y` on the terminal; with no terminal (a script,
    /// CI) it refuses to send unless this is passed.
    #[arg(long, short = 'y')]
    pub yes: bool,

    #[command(flatten)]
    pub policy_source: PolicySourceArgs,

    #[command(flatten)]
    pub verifier: VerifierArgs,

    /// Run a founded cohort's launch assertions and nothing else: every
    /// founding node's reth must serve the manifest's eth.genesis_hash as
    /// block 0, and every founding box's holder its harvested keys.
    /// Configures nothing, writes nothing; for after a launch whose holders
    /// had not settled, a reboot, a re-image, or whenever the cohort may have
    /// drifted from its pins. Takes only --manifest and --node (or the
    /// context): the nodes, genesis files and the appraisal are delivery's,
    /// so their arguments are refused.
    #[arg(
        long,
        conflicts_with_all = [
            "names",
            "bootnode",
            "yes",
            "reth_genesis",
            "summit_genesis",
            "no_verify",
            "policy",
            "measurements",
            "pccs_url",
        ]
    )]
    pub check: bool,
}

pub async fn run(args: ConfigureArgs, config: Option<&Path>) -> anyhow::Result<ExitCode> {
    if args.check {
        return cohort::check(&args, config, launch::CHECK_TIMEOUT).await;
    }
    cohort::configure(&args, config).await
}

/// What a refusal suggests instead, with this run's arguments.
pub(crate) fn remedy(args: &ConfigureArgs) -> Remedy {
    let manifest = args.manifest.as_deref();
    Remedy {
        table: args.cohort.as_flags(),
        names: args.names.iter().map(|name| format!(" {name}")).collect(),
        manifest: manifest
            .map(|path| format!(" --manifest {}", path.display()))
            .unwrap_or_default(),
        dir: manifest
            .map(NetworkDir::of_manifest)
            .filter(|dir| !dir.root().as_os_str().is_empty())
            .map(|dir| format!(" {}", dir.root().display()))
            .unwrap_or_default(),
        appraisal: retry_flags(&args.policy_source, &args.verifier),
    }
}

/// Where a `configure` run's manifest and node table come from: each its own
/// flag, else the selected context network.
///
/// The context file is read the first time an input falls back to it, and
/// never again: with both `--node` and `--manifest` given it is never read,
/// so the command line is a complete record of what the run acted on. Each
/// input is resolved where the run first needs it, so a missing manifest
/// still fails before the node table is read.
pub(crate) struct Inputs<'a> {
    args: &'a ConfigureArgs,
    config: Option<&'a Path>,
    context: OnceCell<Context>,
}

impl<'a> Inputs<'a> {
    pub fn new(args: &'a ConfigureArgs, config: Option<&'a Path>) -> Self {
        Self {
            args,
            config,
            context: OnceCell::new(),
        }
    }

    /// `--manifest`, else the selected network's.
    pub fn manifest(&self) -> anyhow::Result<PathBuf> {
        match &self.args.manifest {
            Some(path) => Ok(path.clone()),
            None => self.selected("--manifest")?.manifest(),
        }
    }

    /// The context network whose directory holds the manifest; `None` under
    /// `--manifest`, which may name a directory no network is registered
    /// for.
    pub fn network(&self) -> anyhow::Result<Option<String>> {
        if self.args.manifest.is_some() {
            return Ok(None);
        }
        Ok(Some(self.selected("--manifest")?.selection.network))
    }

    /// `--node FILE`, else the selected network's node table, echoed.
    pub fn table(&self) -> anyhow::Result<(Descriptors, TableSource)> {
        if let Some(path) = &self.args.cohort.node {
            return Ok((read_nodes(path, "--node")?, TableSource::File(path.clone())));
        }
        let selected = self.selected("--node")?;
        let nodes = selected.nodes()?;
        echo(&selected.selection, &format!("{} node(s)", nodes.len()));
        let source = TableSource::Context(selected.selection.network.clone());
        Ok((nodes.clone(), source))
    }

    /// The selected network, standing in for `flag`, which was not given.
    fn selected(&self, flag: &str) -> anyhow::Result<Selected<'_>> {
        self.context()?.select_for(&self.args.cohort.context, flag)
    }

    fn context(&self) -> anyhow::Result<&Context> {
        if let Some(context) = self.context.get() {
            return Ok(context);
        }
        let context = Context::load(self.config)?;
        Ok(self.context.get_or_init(|| context))
    }
}

/// Where the node table came from, named in the preview beside the network
/// so a stale context shows.
#[derive(Debug)]
pub enum TableSource {
    File(PathBuf),
    Context(String),
}

impl std::fmt::Display for TableSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(path) => write!(f, "--node {}", path.display()),
            Self::Context(network) => write!(f, "context {network}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use seismic_tee_common::http;

    use seismic_tee_common::NodeDescriptor;
    use seismic_tee_common::founding::FoundingRecord;

    use super::*;
    use crate::bootnodes::KeyHolder;
    use crate::plan::{Pinned, Planned, Up};
    use crate::test_support::{FIXTURE_MANIFEST, FakeServer, refused_url, write_file};
    use crate::verify;

    const EXTERNAL_IP: &str = "203.0.113.7";
    /// chainId matches the fixture manifest's eth.chain_id.
    const RETH_GENESIS: &[u8] = br#"{"config": {"chainId": 5124}, "alloc": {}}"#;
    /// namespace matches the fixture manifest's summit.namespace.
    const SUMMIT_GENESIS: &[u8] = b"namespace = \"seismic-devnet-3\"\nvalidators = []\n";

    fn bootnode() -> Bootnode {
        format!("enode://{}@198.51.100.1:30303", "ab".repeat(64))
            .parse()
            .unwrap()
    }

    fn manifest() -> Manifest {
        Manifest::from_json_bytes(FIXTURE_MANIFEST).unwrap()
    }

    fn build(
        manifest: &Manifest,
        reth_genesis: &[u8],
        summit_genesis: &[u8],
        pinned_box: bool,
        bootnodes: &[Bootnode],
    ) -> anyhow::Result<InitConfig> {
        build_config(&ConfigInputs {
            manifest,
            reth_genesis: &Artifact::new("/nets/devnet/reth-genesis.json", reth_genesis),
            summit_genesis: &Artifact::new("/nets/devnet/summit-genesis.toml", summit_genesis),
            bootnodes,
            external_ip: EXTERNAL_IP.parse().unwrap(),
            pinned_box,
        })
    }

    fn decode(b64: &str) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap()
    }

    /// The pinned box keeps its candidate root_key, so no bootnodes — the key
    /// is present and empty. No role flag is sent: tdx-init tells the pinned
    /// box from the manifest. The rendered document is exactly `[node]` +
    /// `[network]`, and the artifacts round-trip byte-exact through the
    /// base64 hop.
    #[test]
    fn the_pinned_box_renders_exactly_the_two_sections() {
        let manifest = manifest();
        let config = build(&manifest, RETH_GENESIS, SUMMIT_GENESIS, true, &[]).unwrap();
        let rendered = render_config(&config).unwrap();

        let table: toml::Table = toml::from_str(&rendered).unwrap();
        let mut sections: Vec<_> = table.keys().collect();
        sections.sort();
        assert_eq!(sections, ["network", "node"]);
        assert_eq!(table["node"].get("genesis_node"), None);
        assert_eq!(table["node"]["external_ip"].as_str(), Some(EXTERNAL_IP));
        assert_eq!(table["node"].get("domain"), None);
        assert_eq!(
            decode(table["network"]["manifest_base64"].as_str().unwrap()),
            manifest.bytes()
        );
        assert_eq!(
            decode(table["network"]["reth_genesis_base64"].as_str().unwrap()),
            RETH_GENESIS
        );
        assert_eq!(
            decode(table["network"]["summit_genesis_base64"].as_str().unwrap()),
            SUMMIT_GENESIS
        );
        assert_eq!(table["network"]["bootnodes"].as_array().unwrap().len(), 0);

        // And it is the document tdx-init deserializes.
        let reparsed: InitConfig = toml::from_str(&rendered).unwrap();
        assert_eq!(reparsed.node.external_ip.to_string(), EXTERNAL_IP);
    }

    /// join: the bootnode set (root_key fetch peers are derived from it by
    /// tdx-init) survives verbatim — a node's own enode included, which
    /// tdx-init drops when deriving peers.
    #[test]
    fn join_mode_carries_the_bootnodes_verbatim() {
        let bootnodes = [
            bootnode(),
            format!("enode://{}@203.0.113.7:30303", "cd".repeat(64))
                .parse()
                .unwrap(),
        ];
        let config = build(&manifest(), RETH_GENESIS, SUMMIT_GENESIS, false, &bootnodes).unwrap();

        assert_eq!(config.node.external_ip.to_string(), EXTERNAL_IP);
        assert_eq!(config.network.bootnodes, bootnodes);

        let reparsed: InitConfig = toml::from_str(&render_config(&config).unwrap()).unwrap();
        assert_eq!(reparsed.network.bootnodes, bootnodes);
    }

    /// tdx-init derives a joiner's root_key fetch peers from the bootnodes, so
    /// an empty set is a far-end 400 — fail fast instead.
    #[test]
    fn a_joiner_without_bootnodes_is_rejected() {
        let error = build(&manifest(), RETH_GENESIS, SUMMIT_GENESIS, false, &[])
            .unwrap_err()
            .to_string();
        assert!(error.contains("at least one bootnode"), "{error}");
    }

    /// A genesis file other than the one the manifest was assembled from must
    /// fail the POST build, not boot a forked node — naming the file.
    #[test]
    fn a_genesis_that_is_not_the_manifests_is_rejected_by_file() {
        let manifest = manifest();

        let error = format!(
            "{:?}",
            build(
                &manifest,
                br#"{"config": {"chainId": 9999}}"#,
                SUMMIT_GENESIS,
                true,
                &[]
            )
            .unwrap_err()
        );
        assert!(
            error.contains("--reth-genesis /nets/devnet/reth-genesis.json"),
            "{error}"
        );
        assert!(error.contains("chainId 9999"), "{error}");

        let error = format!(
            "{:?}",
            build(
                &manifest,
                RETH_GENESIS,
                b"namespace = \"other-net\"\n",
                true,
                &[]
            )
            .unwrap_err()
        );
        assert!(
            error.contains("--summit-genesis /nets/devnet/summit-genesis.toml"),
            "{error}"
        );
        assert!(error.contains("other-net"), "{error}");
    }

    /// The artifact-set layout `assemble` writes is the default; an explicit
    /// path wins; a missing default says where it looked and how to override.
    #[test]
    fn the_genesis_files_default_to_the_manifests_siblings() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = dir
            .path()
            .join(seismic_tee_common::network_dir::MANIFEST_FILENAME);

        let custom = write_file(&dir, "custom.json", b"{}");
        assert_eq!(
            resolve_reth_genesis(Some(&custom), &manifest_path).unwrap(),
            custom
        );

        let error = resolve_reth_genesis(None, &manifest_path)
            .unwrap_err()
            .to_string();
        assert!(error.contains("reth genesis not found"), "{error}");
        assert!(error.contains("pass --reth-genesis"), "{error}");
        let error = resolve_summit_genesis(None, &manifest_path)
            .unwrap_err()
            .to_string();
        assert!(error.contains("summit genesis not found"), "{error}");
        assert!(error.contains("pass --summit-genesis"), "{error}");

        let reth = write_file(&dir, "reth-genesis.json", b"{}");
        let summit = write_file(&dir, "summit-genesis.toml", b"");
        assert_eq!(resolve_reth_genesis(None, &manifest_path).unwrap(), reth);
        assert_eq!(
            resolve_summit_genesis(None, &manifest_path).unwrap(),
            summit
        );

        // An explicit path that is missing gets no default-location hint: the
        // operator named it.
        let error = resolve_reth_genesis(Some(Path::new("/absent/g.json")), &manifest_path)
            .unwrap_err()
            .to_string();
        assert!(error.contains("/absent/g.json"), "{error}");
        assert!(!error.contains("the default is"), "{error}");
    }

    #[tokio::test]
    async fn the_post_is_the_rendered_toml_and_a_200_is_acceptance() {
        let manifest = manifest();
        let config = build(
            &manifest,
            RETH_GENESIS,
            SUMMIT_GENESIS,
            false,
            &[bootnode()],
        )
        .unwrap();
        let server = FakeServer::serve(vec![(200, "ok".to_string())]);
        let client = http::client().unwrap();

        post_config_within(
            &client,
            &format!("{}/", server.url),
            &config,
            Duration::from_secs(10),
            Duration::from_millis(20),
            |_| {},
        )
        .await
        .unwrap();

        let requests = server.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "POST");
        assert_eq!(requests[0].path, "/");
        assert_eq!(
            requests[0].content_type.as_deref(),
            Some("application/toml")
        );
        let received: InitConfig =
            toml::from_str(std::str::from_utf8(&requests[0].body).unwrap()).unwrap();
        assert_eq!(received.network.bootnodes, [bootnode()]);
        assert_eq!(received.node.external_ip.to_string(), EXTERNAL_IP);
    }

    /// A rejection is propagated at once, status and body: neither a 4xx
    /// (tdx-init refusing the schema) nor a 5xx benefits from retry.
    #[tokio::test]
    async fn a_rejection_is_not_retried() {
        let config = build(&manifest(), RETH_GENESIS, SUMMIT_GENESIS, true, &[]).unwrap();
        let server = FakeServer::serve(vec![
            (400, "unknown field `bogus`".to_string()),
            (200, "never reached".to_string()),
        ]);
        let client = http::client().unwrap();

        let error = post_config_within(
            &client,
            &format!("{}/", server.url),
            &config,
            Duration::from_secs(10),
            Duration::from_millis(20),
            |_| {},
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("tdx-init rejected config: 400 unknown field `bogus`"),
            "{error}"
        );
        assert_eq!(server.requests().len(), 1);
    }

    /// Connection refused is retried until the listener appears; a listener
    /// that never appears is the timeout, naming the last error.
    #[tokio::test]
    async fn a_listener_that_is_not_up_yet_is_waited_for() {
        let config = build(&manifest(), RETH_GENESIS, SUMMIT_GENESIS, true, &[]).unwrap();
        let client = http::client().unwrap();

        let server =
            FakeServer::serve_after(Duration::from_millis(300), vec![(200, "ok".to_string())]);
        post_config_within(
            &client,
            &format!("{}/", server.url),
            &config,
            Duration::from_secs(10),
            Duration::from_millis(20),
            |_| {},
        )
        .await
        .unwrap();
        assert_eq!(server.requests().len(), 1);

        let error = post_config_within(
            &client,
            &format!("{}/", refused_url()),
            &config,
            Duration::from_millis(100),
            Duration::from_millis(20),
            |_| {},
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(error.contains("never came up after 0s"), "{error}");
        assert!(error.contains("last error"), "{error}");
    }

    #[derive(Parser)]
    struct Probe {
        #[command(flatten)]
        args: ConfigureArgs,
    }

    fn parse(argv: &[&str]) -> Result<ConfigureArgs, clap::Error> {
        Probe::try_parse_from(std::iter::once(&"configure").chain(argv)).map(|probe| probe.args)
    }

    /// A bare `configure` is the whole node table, asking first; names
    /// narrow it.
    #[test]
    fn bare_configure_is_the_whole_table_with_the_documented_defaults() {
        let args = parse(&[]).unwrap();
        assert!(args.names.is_empty());
        assert!(args.bootnode.is_empty());
        assert!(!args.no_verify);
        assert!(!args.yes, "the gate is on unless asked off");
        assert!(!args.check);
        assert_eq!(
            args.policy_source.attestation_type,
            verify::DEFAULT_ATTESTATION_TYPE
        );

        let args = parse(&["beta", "gamma", "--yes"]).unwrap();
        assert_eq!(args.names, ["beta", "gamma"]);
        assert!(args.yes);
    }

    /// `--bootnode` is parsed as the type tdx-init takes, so a value it would
    /// refuse is refused before anything is probed or sent.
    #[test]
    fn a_flag_value_tdx_init_would_refuse_does_not_parse() {
        assert!(parse(&["--bootnode", "enode://ab@1.2.3.4:30303"]).is_err());
        let args = parse(&["--bootnode", bootnode().as_ref()]).unwrap();
        assert_eq!(args.bootnode, [bootnode()]);
    }

    /// `--email` went with the node's certificate: the image takes none.
    #[test]
    fn the_removed_email_flag_does_not_parse() {
        assert!(parse(&["--email", "ops@example.com"]).is_err());
    }

    /// The mode flags are gone: the network's state decides.
    #[test]
    fn the_retired_flags_do_not_parse() {
        for argv in [
            &["--genesis-node", "a"][..],
            &["--join", "b"],
            &["--name", "a"],
            &["--dump-config", "c.toml"],
        ] {
            assert!(parse(argv).is_err(), "{argv:?}");
        }
    }

    /// `--check` delivers nothing, so every argument that only delivery reads
    /// is refused rather than silently ignored.
    #[test]
    fn check_refuses_the_delivery_arguments() {
        assert!(parse(&["--check"]).unwrap().check);
        for excluded in [
            &["alpha"][..],
            &["--bootnode", bootnode().as_ref()],
            &["--yes"],
            &["--reth-genesis", "r.json"],
            &["--summit-genesis", "s.toml"],
            &["--no-verify"],
            &["--policy", "p.json"],
            &["--measurements", "m.json"],
            &["--pccs-url", "https://pccs"],
        ] {
            let argv: Vec<&str> = std::iter::once("--check")
                .chain(excluded.iter().copied())
                .collect();
            assert!(parse(&argv).is_err(), "{excluded:?}");
        }
    }

    /// A refusal's remedy repeats the run's table, names and manifest.
    #[test]
    fn the_remedy_repeats_the_runs_arguments() {
        let args = parse(&[
            "beta",
            "--node",
            "nodes.json",
            "--manifest",
            "net/network-manifest.json",
        ])
        .unwrap();
        let remedy = remedy(&args);
        assert_eq!(
            remedy.bootnode(),
            "seismic-tee node configure --node nodes.json beta --bootnode <ENODE> --manifest \
             net/network-manifest.json"
        );
        assert!(
            remedy
                .re_found()
                .contains("seismic-tee node harvest net --node nodes.json --force"),
            "{}",
            remedy.re_found()
        );
    }

    fn descriptor(name: &str) -> NodeDescriptor {
        NodeDescriptor {
            public_ip: "203.0.113.9".parse().unwrap(),
            fqdn: format!("{name}.example.com").parse().unwrap(),
        }
    }

    fn planned(name: &str, up: Up) -> Planned {
        Planned {
            name: name.to_string(),
            descriptor: descriptor(name),
            up,
            harvested_node_key: None,
            record: None,
        }
    }

    fn preview_inputs() -> (Manifest, Artifact, Artifact) {
        (
            manifest(),
            Artifact::new("/nets/devnet/reth-genesis.json", RETH_GENESIS),
            Artifact::new("/nets/devnet/summit-genesis.toml", SUMMIT_GENESIS),
        )
    }

    /// A founding's preview names the network and its pinned box, which goes
    /// first and alone; what the base64 hides is decoded.
    #[test]
    fn a_foundings_preview_names_the_pinned_box_first() {
        let (manifest, reth_genesis, summit_genesis) = preview_inputs();
        let run = Run::Found {
            pinned: Pinned {
                name: "alpha".to_string(),
                descriptor: descriptor("alpha"),
                record: FoundingRecord {
                    node_public_key: "aa".repeat(32),
                    consensus_public_key: "cc".repeat(48),
                    candidate_tx_io_public_key: "02".repeat(33),
                    document: serde_json::json!({}),
                },
            },
            rest: vec![planned("beta", Up::Awaiting), planned("gamma", Up::NotUp)],
        };
        let text = render_preview(&Preview {
            table: &TableSource::Context("devnet-1".to_string()),
            manifest_path: Path::new("/nets/devnet/network-manifest.json"),
            manifest: &manifest,
            reth_genesis: &reth_genesis,
            summit_genesis: &summit_genesis,
            policy: Some("/nets/devnet/measurement-policy-bootstrap.json (pinned by --manifest)"),
            run: &run,
            skipped: &[],
            bootnodes: &[],
            record: Path::new("/nets/devnet/nodes/<name>.init-config.toml"),
        });
        for expected in [
            &format!(
                "This founds network {:?} (node table: context devnet-1); pinned box alpha.",
                manifest.name
            ),
            "/nets/devnet/network-manifest.json",
            "chain_id 5124",
            &format!("network_id {}", manifest.network_id()),
            &format!(
                "reth-genesis.json  sha256 {}",
                hex::encode(Sha256::digest(RETH_GENESIS))
            ),
            &format!(
                "summit-genesis.toml  sha256 {}",
                hex::encode(Sha256::digest(SUMMIT_GENESIS))
            ),
            "measurement-policy-bootstrap.json (pinned by",
            "Nodes:            alpha, first and alone: keeps its candidate root_key",
            "then beta, gamma: fetch root_key from alpha",
            "Not up yet:       gamma (waited for, up to 180s)",
            "/nets/devnet/nodes/<name>.init-config.toml",
        ] {
            assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
        }
        assert!(!text.contains("Bootnodes:"), "{text}");
        assert!(!text.contains("Skipped:"), "{text}");
    }

    /// A join's preview lists the nodes, the bootnodes they fetch from, and
    /// the nodes skipped.
    #[test]
    fn a_joins_preview_lists_the_bootnodes_and_the_skipped() {
        let (manifest, reth_genesis, summit_genesis) = preview_inputs();
        let run = Run::Join {
            nodes: vec![planned("beta", Up::Awaiting)],
            through: Through::KeyHolders(vec![KeyHolder::new("alpha", &descriptor("alpha"))]),
        };
        let skipped = ["alpha".to_string()];
        let bootnodes = [bootnode()];
        let text = render_preview(&Preview {
            table: &TableSource::Context("devnet-1".to_string()),
            manifest_path: Path::new("/nets/devnet/network-manifest.json"),
            manifest: &manifest,
            reth_genesis: &reth_genesis,
            summit_genesis: &summit_genesis,
            policy: None,
            run: &run,
            skipped: &skipped,
            bootnodes: &bootnodes,
            record: Path::new("/nets/devnet/nodes/<name>.init-config.toml"),
        });
        for expected in [
            &format!(
                "This joins 1 node(s) to network {:?} (node table: context devnet-1).",
                manifest.name
            ),
            "Nodes:            beta",
            "Bootnodes:        1, key holder(s) that passed their appraisal",
            bootnode().as_ref(),
            "Skipped:          alpha (took its config this boot)",
            "nothing (--no-verify)",
        ] {
            assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
        }
    }

    /// `--yes` sends without reading; a terminal is asked and only y/yes
    /// sends; no terminal and no `--yes` is a refusal naming the flag.
    #[test]
    fn confirm_run_gates_the_run() {
        let mut empty = std::io::Cursor::new(b"".to_vec());
        assert!(confirm_run(true, false, &mut empty).unwrap());

        let error = confirm_run(false, false, &mut empty)
            .unwrap_err()
            .to_string();
        assert!(error.contains("pass --yes"), "{error}");
        assert!(error.contains("not a terminal"), "{error}");

        for (answer, sends) in [
            ("y\n", true),
            ("Y\n", true),
            ("yes\n", true),
            ("YES\n", true),
            ("n\n", false),
            ("\n", false),
            ("", false),
            ("yeah\n", false),
        ] {
            let mut input = std::io::Cursor::new(answer.as_bytes().to_vec());
            assert_eq!(
                confirm_run(false, true, &mut input).unwrap(),
                sends,
                "{answer:?}"
            );
        }
    }

    /// The record lands in the network directory's `nodes/` tier, and
    /// writing creates the tier.
    #[test]
    fn the_record_is_written_under_nodes_beside_the_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let record = NetworkDir::new(dir.path()).init_config("dev-2");
        assert_eq!(
            record,
            dir.path().join("nodes").join("dev-2.init-config.toml")
        );

        let config = build(
            &manifest(),
            RETH_GENESIS,
            SUMMIT_GENESIS,
            false,
            &[bootnode()],
        )
        .unwrap();
        let rendered = render_config(&config).unwrap();
        write_record(&record, &rendered).unwrap();
        assert_eq!(std::fs::read_to_string(&record).unwrap(), rendered);
        // And it is the document tdx-init would take.
        let reparsed: InitConfig = toml::from_str(&rendered).unwrap();
        assert_eq!(reparsed.network.bootnodes, [bootnode()]);
    }

    const CONTEXT: &str = r#"
current = "devnet-1"

[networks.devnet-1]
dir = "/nets/devnet-1"

[networks.devnet-1.nodes]
alpha = { public_ip = "203.0.113.7", fqdn = "alpha.example.com" }
"#;

    /// `Inputs` falls back to `SEISMIC_CONTEXT` too; nextest gives each test
    /// its own process, so clearing it here races no other test.
    fn clear_context_env() {
        // SAFETY: this process runs this one test.
        unsafe { std::env::remove_var("SEISMIC_CONTEXT") };
    }

    /// Neither flag: the context supplies the table, the manifest, and the
    /// network the closing step selects a node under.
    #[test]
    fn inputs_without_flags_come_from_the_context() {
        clear_context_env();
        let dir = tempfile::tempdir().unwrap();
        let config = write_file(&dir, "config.toml", CONTEXT.as_bytes());
        let args = parse(&[]).unwrap();
        let inputs = Inputs::new(&args, Some(&config));

        let (table, source) = inputs.table().unwrap();
        assert_eq!(table.keys().collect::<Vec<_>>(), ["alpha"]);
        assert_eq!(source.to_string(), "context devnet-1");
        assert_eq!(
            inputs.manifest().unwrap(),
            Path::new("/nets/devnet-1/network-manifest.json")
        );
        assert_eq!(inputs.network().unwrap().as_deref(), Some("devnet-1"));
    }

    /// The context file is read once: rewritten mid-run, the run still sees
    /// the first read.
    #[test]
    fn inputs_read_the_context_once() {
        clear_context_env();
        let dir = tempfile::tempdir().unwrap();
        let config = write_file(&dir, "config.toml", CONTEXT.as_bytes());
        let args = parse(&[]).unwrap();
        let inputs = Inputs::new(&args, Some(&config));

        inputs.manifest().unwrap();
        std::fs::write(&config, "not toml [").unwrap();
        assert_eq!(inputs.table().unwrap().1.to_string(), "context devnet-1");
        assert_eq!(inputs.network().unwrap().as_deref(), Some("devnet-1"));
    }

    /// Both flags: the context is never read, so even a malformed one is no
    /// obstacle.
    #[test]
    fn inputs_with_both_flags_never_read_the_context() {
        clear_context_env();
        let dir = tempfile::tempdir().unwrap();
        let config = write_file(&dir, "config.toml", b"not toml [");
        let nodes = write_file(
            &dir,
            "nodes.json",
            br#"{"solo": {"public_ip": "203.0.113.9", "fqdn": "solo.example.com"}}"#,
        );
        let args = parse(&[
            "--node",
            nodes.to_str().unwrap(),
            "--manifest",
            "net/network-manifest.json",
        ])
        .unwrap();
        let inputs = Inputs::new(&args, Some(&config));

        let (table, source) = inputs.table().unwrap();
        assert_eq!(table.keys().collect::<Vec<_>>(), ["solo"]);
        assert_eq!(source.to_string(), format!("--node {}", nodes.display()));
        assert_eq!(
            inputs.manifest().unwrap(),
            Path::new("net/network-manifest.json")
        );
        assert_eq!(inputs.network().unwrap(), None);
    }

    /// No context selected: each input's refusal names its own flag.
    #[test]
    fn inputs_with_no_context_name_the_missing_flag() {
        clear_context_env();
        let dir = tempfile::tempdir().unwrap();
        // Names no file: an empty config, with no `current`.
        let config = dir.path().join("config.toml");
        let args = parse(&[]).unwrap();
        let inputs = Inputs::new(&args, Some(&config));

        let err = inputs.table().unwrap_err().to_string();
        assert!(err.contains("no --node given"), "{err}");
        assert!(err.contains("ctx use"), "{err}");
        let err = inputs.manifest().unwrap_err().to_string();
        assert!(err.contains("no --manifest given"), "{err}");
        assert!(err.contains("ctx use"), "{err}");
    }
}
