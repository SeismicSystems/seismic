//! `configure`: configure a provisioned node to JOIN a network.
//!
//! Assemble a node's tdx-init config from flags + a descriptor + the network
//! manifest, and POST it to the node's tdx-init HTTP receiver:
//!
//! ```text
//! seismic-tee node configure --node n2.json \
//!     --bootnode enode://<pubkey>@<ip>:30303 --manifest m.json
//! ```
//!
//! The `node` group only ever *joins* an existing network (`genesis_node =
//! false`): the node fetches `root_key` via `getWrappedRootKey` from a peer
//! tdx-init derives from `--bootnode` (`http://<host>:7878` per bootnode).
//! Founding a network — designating the one genesis node that mints `root_key`
//! locally — is owned by `seismic-tee network configure`, not exposed here.
//! [`build_config`] and [`post_config`] are the shared primitives both groups
//! call; `genesis_node` is a library-only knob with no flag on this command.
//!
//! The node is deploy-verified once it reaches a ready state — the `verify`
//! step, run inline here so a node is appraised in the same breath it is
//! configured. The configured summary is printed only after it passes, so it
//! never reads as success over a failed verification. Delivery is once per
//! boot (tdx-init accepts one config POST) while the check is re-runnable, so
//! anything that interrupts the check points at `verify` rather than at
//! another configure run. Verification is the default because the artifact
//! set already carries everything it needs: the policy sits beside
//! `--manifest`, pinned by it. An operator who wants delivery alone asks for
//! it with `--no-verify`.
//!
//! There is no per-node `node.toml`: `[node]` (external_ip, genesis_node)
//! comes from the descriptor and the role, `[node.domain]` from the descriptor
//! fqdn and `--email`, and `[network]` from `--manifest`, `--reth-genesis`,
//! `--summit-genesis` and `--bootnode`. Those network-wide artifacts stay
//! standalone files, merged only at POST time. The config is built as the
//! [`InitConfig`] type tdx-init deserializes it with (`deny_unknown_fields`
//! at the far end), so this CLI and the node image cannot disagree on the
//! shape without failing to compile.
//!
//! The POST is the one irreversible step an operator takes (tdx-init accepts
//! one per boot), so it is gated: the rendered TOML is written to disk first
//! — `nodes/<name>.init-config.toml` beside `--manifest`, the network
//! directory's per-deploy tier, or `--dump-config` — then a decoded preview
//! of it is shown and the terminal is asked. The TOML itself is three base64
//! blobs, so the preview says what they are: which manifest (name, chain,
//! network id), which genesis files (by hash), which bootnodes, which policy
//! will appraise the node. `--yes` sends unattended; with no terminal and no
//! `--yes` nothing is sent. The record stays as what the node booted with,
//! and as a body `curl` can replay.

use std::io::{BufRead, IsTerminal as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use base64::Engine as _;
use clap::Args;
use seismic_tee_common::http::ATTESTATION_RPC_PORT;
use seismic_tee_common::{Artifact, Manifest, NetworkDir, NodeDescriptor, http, next_step, rpc};
use sha2::{Digest as _, Sha256};
use tdx_init_config::{DomainConfig, InitConfig, NetworkConfig, NodeConfig};

use crate::args::NodeArgs;
use crate::status;
use crate::verify::{self, PolicySourceArgs, VerifierArgs};

/// How long to wait for tdx-init's HTTP listener to come up. tdx-init starts
/// after persistent-luks-setup, which can take ~20-40s on first boot (LUKS
/// format + mkfs + TPM enroll).
pub const TDX_INIT_LISTENER_TIMEOUT: Duration = Duration::from_secs(180);
pub const TDX_INIT_RETRY_INTERVAL: Duration = Duration::from_secs(5);

/// Where the node's Let's Encrypt registration goes, unless told otherwise.
pub const DEFAULT_EMAIL: &str = "ops@seismic.systems";

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
    /// The summit genesis, likewise. A founder delivers it with current
    /// validator IPs spliced in, which is why it is bytes-with-a-path rather
    /// than a path.
    pub summit_genesis: &'a Artifact,
    /// The enode set → `[network].bootnodes`. Empty is valid only on the
    /// greenfield genesis node.
    pub bootnodes: &'a [String],
    /// The node's own public IP → `[node].external_ip` (reth's `--nat extip`).
    pub external_ip: &'a str,
    /// The node's DNS name → `[node.domain].name`, the cert domain.
    pub fqdn: &'a str,
    /// → `[node.domain].email`, the Let's Encrypt registration.
    pub email: &'a str,
    /// Only ever `true` from the founding command; the operator `configure`
    /// always joins.
    pub genesis_node: bool,
}

/// Assemble the config POSTed to tdx-init, mutating no source.
///
/// Written fresh from its inputs, so there is no operator-supplied TOML that
/// could carry a conflicting `[node]`/`[network]` and fork the network. The
/// two checks tdx-init would fail the POST on are run here first, so a bad
/// input fails while the fix still costs nothing:
///
/// - `external_ip` must be non-empty: tdx-init parses it as an `IpAddr`, and
///   every caller sources it from the descriptor's required `public_ip`, so
///   this should be unreachable.
/// - A joiner needs at least one bootnode: tdx-init derives its root_key
///   fetch peers from them, so none means no way to bootstrap.
///
/// Then the manifest's gates over the two genesis files, naming the file that
/// fails. The artifacts travel base64-encoded so they stay opaque through the
/// TOML hop: tdx-init decodes and writes these exact bytes verbatim.
pub fn build_config(inputs: &ConfigInputs<'_>) -> anyhow::Result<InitConfig> {
    if inputs.external_ip.is_empty() {
        bail!("build_config: external_ip is required and must be non-empty");
    }
    if !inputs.genesis_node && inputs.bootnodes.is_empty() {
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
            external_ip: inputs.external_ip.to_string(),
            genesis_node: inputs.genesis_node,
            domain: DomainConfig {
                email: inputs.email.to_string(),
                name: inputs.fqdn.to_string(),
            },
        },
    })
}

/// The config as the TOML document tdx-init receives.
pub fn render_config(config: &InitConfig) -> anyhow::Result<String> {
    toml::to_string(config).context("rendering the tdx-init config as TOML")
}

/// Where the rendered config is recorded: `--dump-config`, or the network
/// directory's `nodes/<name>.init-config.toml` beside `--manifest` — the tier
/// that holds the bootnode set, per-deploy output like this.
pub fn resolve_record_path(flag: Option<&Path>, manifest_path: &Path, name: &str) -> PathBuf {
    flag.map_or_else(
        || NetworkDir::of_manifest(manifest_path).init_config(name),
        Path::to_path_buf,
    )
}

/// Write the rendered config to `path`, creating its directory. Done before
/// the POST, so the record exists whatever comes of it — confirmed, declined,
/// or rejected by tdx-init.
pub fn write_record(path: &Path, rendered: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {} for the config record", parent.display()))?;
    }
    std::fs::write(path, rendered)
        .with_context(|| format!("writing the config record {}", path.display()))
}

/// What `configure` is about to do, decoded for a human.
pub struct Preview<'a> {
    /// The node's key in the descriptor map.
    pub name: &'a str,
    pub manifest_path: &'a Path,
    pub inputs: &'a ConfigInputs<'a>,
    /// The policy's provenance, or `None` under `--no-verify`.
    pub policy: Option<&'a str>,
    /// Where the rendered TOML was written.
    pub record: &'a Path,
    pub post_url: &'a str,
}

/// The preview as text. The rendered TOML carries the manifest and both
/// genesis files as base64, so the review names them by path and hash and
/// spells out the fields a wrong flag would change.
pub fn render_preview(preview: &Preview<'_>) -> String {
    let Preview {
        name,
        manifest_path,
        inputs,
        policy,
        record,
        post_url,
    } = preview;
    let manifest = inputs.manifest;
    let sha256 = |bytes: &[u8]| hex::encode(Sha256::digest(bytes));
    let mut out = String::new();
    out.push_str(&format!(
        "About to configure node {name}
"
    ));
    out.push_str(&format!(
        "  POST to:          {post_url}
"
    ));
    out.push_str(&format!(
        "  Node:             {} ({}), role: {}
",
        inputs.fqdn,
        inputs.external_ip,
        if inputs.genesis_node {
            "genesis"
        } else {
            "join"
        },
    ));
    out.push_str(&format!(
        "  Cert email:       {}
",
        inputs.email
    ));
    out.push_str(&format!(
        "  Manifest:         {}
",
        manifest_path.display()
    ));
    out.push_str(&format!(
        "                    name {:?}, chain_id {}, network_id {}
",
        manifest.name,
        manifest.eth.chain_id,
        manifest.network_id(),
    ));
    out.push_str(&format!(
        "  reth genesis:     {}  sha256 {}
",
        inputs.reth_genesis.path().display(),
        sha256(inputs.reth_genesis.bytes()),
    ));
    out.push_str(&format!(
        "  summit genesis:   {}  sha256 {}
",
        inputs.summit_genesis.path().display(),
        sha256(inputs.summit_genesis.bytes()),
    ));
    out.push_str(&format!(
        "  Bootnodes:        {}
",
        inputs.bootnodes.len()
    ));
    for bootnode in inputs.bootnodes {
        out.push_str(&format!(
            "                    {bootnode}
"
        ));
    }
    out.push_str(&format!(
        "  Appraise against: {}
",
        policy.unwrap_or("nothing (--no-verify)")
    ));
    out.push_str(&format!(
        "  Rendered config:  {}
",
        record.display()
    ));
    out
}

/// The gate before the one irreversible step. `yes` is `--yes <NAME>`, and
/// sends without asking only when `NAME` names the node this run resolved to
/// — a mismatch is refused rather than silently redirected at the wrong box,
/// which is what makes `--yes` safe to leave in a script alongside an
/// implicit (context-resolved) target. Otherwise a terminal is asked and
/// anything but `y`/`yes` declines, and no terminal is an error — a script
/// that means it says so with the flag, rather than a redirected stdin
/// deciding silently.
pub fn confirm_post(
    yes: Option<&str>,
    node: &str,
    interactive: bool,
    input: &mut impl BufRead,
) -> anyhow::Result<bool> {
    match yes {
        Some(named) if named == node => return Ok(true),
        Some(named) => bail!(
            "--yes {named} does not name the node this resolved to ({node}); nothing was sent"
        ),
        None => {}
    }
    if !interactive {
        bail!(
            "stdin is not a terminal, so nobody is here to confirm the POST; pass --yes <NAME> \
             naming the node to send it unattended"
        );
    }
    eprint!("POST this config? [y/N] ");
    let mut answer = String::new();
    input
        .read_line(&mut answer)
        .context("reading the confirmation")?;
    let answer = answer.trim();
    Ok(answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes"))
}

/// Wait for tdx-init's HTTP listener at `url` to come up, then POST the
/// config. tdx-init validates the schema server-side and 4xx-rejects a
/// malformed payload, so any non-200 is an error — 4xx = a config it rejects,
/// 5xx = a tdx-init bug; neither benefits from retry. `url` is the node's
/// receiver ([`NodeDescriptor::tdx_init_url`]).
pub async fn post_config(
    client: &reqwest::Client,
    url: &str,
    config: &InitConfig,
) -> anyhow::Result<()> {
    post_config_within(
        client,
        url,
        config,
        TDX_INIT_LISTENER_TIMEOUT,
        TDX_INIT_RETRY_INTERVAL,
        |line| eprintln!("{line}"),
    )
    .await
}

/// [`post_config`] with its own patience for the listener, reporting progress
/// through `report` — a line at a time, so a cohort dashboard can show it as
/// the node's status instead of it landing on stderr mid-repaint.
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

/// The success banner. Nothing prints it before the last gate. `record` is
/// where the POSTed config was written.
pub fn print_summary(fqdn: &str, public_ip: &str, record: &Path) {
    let rule = "=".repeat(80);
    println!("\n{rule}");
    println!("NODE CONFIGURED");
    println!("{rule}");
    println!("\nNode:       {fqdn}");
    println!("IP Address: {public_ip}");
    println!("Config:     {} (as POSTed)", record.display());
    println!("\nNginx + SSL set up automatically after initialization.");
    println!("Endpoints (once the node settles):");
    println!("  https://{fqdn}/rpc");
    println!("  https://{fqdn}/ws");
    println!("  https://{fqdn}/summit");
    println!("\n{rule}\n");
}

#[derive(Debug, Args)]
pub struct ConfigureArgs {
    #[command(flatten)]
    pub node: NodeArgs,

    /// Network manifest JSON (from `seismic-tee network assemble`). Merged
    /// into the POSTed config as [network].manifest_base64; shared across
    /// every node, so it lives outside the per-node flags. Omit it to use
    /// the current context's network.
    #[arg(long, value_name = "FILE")]
    pub manifest: Option<PathBuf>,

    /// reth genesis JSON POSTed to the node as [network].reth_genesis_base64;
    /// tdx-init writes it to /run/seismic/conf/reth-genesis.json for reth's
    /// --chain. Must be the file the manifest's eth.genesis_hash was computed
    /// from. Default: reth-genesis.json beside --manifest (the artifact-set
    /// layout `assemble` produces).
    #[arg(long, value_name = "FILE")]
    pub reth_genesis: Option<PathBuf>,

    /// summit genesis TOML POSTed to the node as
    /// [network].summit_genesis_base64; tdx-init writes it to
    /// /run/seismic/conf/summit-genesis.toml for summit's --genesis-path.
    /// Must be the file the manifest's summit.genesis_config_digest was
    /// computed from. Default: summit-genesis.toml beside --manifest (the
    /// artifact-set layout `assemble` produces).
    #[arg(long, value_name = "FILE")]
    pub summit_genesis: Option<PathBuf>,

    /// Bootnode enode URL (enode://<pubkey>@<host>:<port>) →
    /// [network].bootnodes. reth dials it on startup, and tdx-init derives the
    /// root_key fetch peer from it (http://<host>:7878). Repeatable; required
    /// — a joining node has no root_key of its own. Fetch a running node's
    /// enode from its seismic_nodeInfo RPC (the founding set a network writes
    /// to nodes/bootnodes.json).
    #[arg(long, value_name = "ENODE", required = true)]
    pub bootnode: Vec<String>,

    /// Contact email for the node's Let's Encrypt registration (certbot); goes
    /// into [node.domain].email of the POSTed config. Same across a cohort.
    #[arg(long, default_value = DEFAULT_EMAIL)]
    pub email: String,

    /// Configure the node without deploy-verifying it. By default the node is
    /// appraised once it is up — the same check as `seismic-tee node verify`,
    /// against the policy --manifest pins — and this command exits nonzero
    /// unless it passes.
    #[arg(long)]
    pub no_verify: bool,

    /// Send the config without asking. Takes the node's name: you name the
    /// box you meant, so a stale context fails loudly instead of configuring
    /// the wrong one. Without it, configure shows what it is about to POST
    /// and waits for a `y` on the terminal; with no terminal (a script, CI)
    /// it refuses to send unless this is passed.
    #[arg(long, short = 'y', value_name = "NAME")]
    pub yes: Option<String>,

    /// Where to write the rendered TOML, byte-exact as POSTed. Default:
    /// nodes/<name>.init-config.toml beside --manifest (the network
    /// directory's per-deploy tier). Written before the POST, so it records
    /// the attempt whatever comes of it, and is a body `curl` can replay.
    #[arg(long, value_name = "FILE")]
    pub dump_config: Option<PathBuf>,

    #[command(flatten)]
    pub policy_source: PolicySourceArgs,

    #[command(flatten)]
    pub verifier: VerifierArgs,
}

pub async fn run(args: ConfigureArgs) -> anyhow::Result<ExitCode> {
    let (name, descriptor) = args.node.load()?;
    verify::check_policy_source_files(&args.policy_source, args.no_verify)?;
    let manifest_path = crate::resolve_manifest(args.manifest.as_deref(), &args.node.context)?;
    let manifest = crate::load_manifest(&manifest_path)?;
    let NodeDescriptor { fqdn, public_ip } = &descriptor;

    // Resolve the policy before the node is touched: a policy the manifest
    // doesn't commit to, or a rejected measurements file, must fail while the
    // fix still costs nothing, not after the config POST landed.
    let policy = if args.no_verify {
        eprintln!(
            "warning: --no-verify: this node will not be appraised. Run `seismic-tee node \
             verify` before relying on an unappraised node."
        );
        None
    } else {
        Some(verify::resolve_policy(
            &args.policy_source,
            &manifest_path,
            &manifest,
            true,
        )?)
    };

    // Assemble the POST config before contacting the node, so bad local input
    // (a genesis that isn't the manifest's, a missing bootnode) fails fast.
    let reth_genesis = Artifact::read(&resolve_reth_genesis(
        args.reth_genesis.as_deref(),
        &manifest_path,
    )?)?;
    let summit_genesis = Artifact::read(&resolve_summit_genesis(
        args.summit_genesis.as_deref(),
        &manifest_path,
    )?)?;
    let inputs = ConfigInputs {
        manifest: &manifest,
        reth_genesis: &reth_genesis,
        summit_genesis: &summit_genesis,
        bootnodes: &args.bootnode,
        external_ip: public_ip,
        fqdn,
        email: &args.email,
        genesis_node: false,
    };
    let config = build_config(&inputs)?;

    // The record first: what is about to be sent exists on disk before
    // anything is sent, and the preview names it so it can be opened while
    // deciding.
    let record = resolve_record_path(args.dump_config.as_deref(), &manifest_path, &name);
    write_record(&record, &render_config(&config)?)?;

    let post_url = descriptor.tdx_init_url();
    eprint!(
        "{}",
        render_preview(&Preview {
            name: &name,
            manifest_path: &manifest_path,
            inputs: &inputs,
            policy: policy.as_ref().map(|p| p.source.as_str()),
            record: &record,
            post_url: &post_url,
        })
    );
    // Blocking read on the runtime's main task: nothing else is in flight
    // until the operator answers.
    let stdin = std::io::stdin();
    if !confirm_post(
        args.yes.as_deref(),
        &name,
        stdin.is_terminal(),
        &mut stdin.lock(),
    )? {
        eprintln!(
            "Nothing was sent. The rendered config is at {}.",
            record.display()
        );
        return Ok(ExitCode::FAILURE);
    }

    let client = http::client()?;
    // Built before the POST so an unusable endpoint fails while nothing has
    // been delivered.
    let rpc_url = descriptor.attestation_rpc_url();
    let rpc_client = rpc::Client::new(&rpc_url)?;
    eprintln!("Configuring node {fqdn} ({public_ip}) as join...");
    post_config(&client, &post_url, &config).await?;
    eprintln!("config delivered to tdx-init.");

    // Watch the first-boot LUKS wipe — the long, otherwise-opaque phase.
    // Purely local observability: the POST already landed, so ctrl-C here
    // only stops watching; the node keeps provisioning in the background.
    let ready = tokio::select! {
        ok = status::watch_luks_provisioning(&rpc_client, status::POLL_INTERVAL) => Some(ok),
        _ = tokio::signal::ctrl_c() => None,
    };

    let Some(ready) = ready else {
        println!("\nStopped watching — node still provisioning in the background.");
        let Some(_) = policy else {
            print_summary(fqdn, public_ip, &record);
            // No appraisal was asked for, so nothing is owed but the watch
            // the operator left; re-watch to see it settle.
            next_step::print(
                "",
                &[format!("seismic-tee node status{}", args.node.as_flags())],
            );
            return Ok(ExitCode::SUCCESS);
        };
        // The operator stopped watching before the attestation service came
        // up, so there is nothing to challenge yet. Exit nonzero — the
        // requested verification did not happen — and point at the standalone
        // command: the config this boot needs is already delivered.
        bail!(
            "deploy verification skipped: the node was not confirmed ready. Once it is up, \
             run:\n    seismic-tee node verify{} --manifest {}{}",
            args.node.as_flags(),
            manifest_path.display(),
            verify::retry_flags(&args.policy_source, &args.verifier),
        );
    };

    if !ready {
        // The POST succeeded, but the node never reached a ready state within
        // the watch window: the attestation service's :7878 didn't come up (it
        // only starts serving once it has root_key) or the LUKS wipe errored.
        // Don't print a success summary — that's the misleading case. It may
        // still be mid-bootstrap (e.g. fetching root_key from a slow peer), so
        // point at a re-watch rather than declaring the node dead; exit
        // non-zero either way.
        bail!(
            "config delivered to {fqdn} ({public_ip}), but the node did not reach a ready state \
             within the watch window (attestation service :{ATTESTATION_RPC_PORT} never came \
             up, or the LUKS wipe errored). It may still be bootstrapping, or stuck — check \
             attestation-service logs on the node, then re-watch with:\n    seismic-tee node \
             status{}",
            args.node.as_flags(),
        );
    }

    if let Some(policy) = &policy {
        // Interruptible like the watch: the config is delivered either way, and
        // the appraisal has its own command.
        tokio::select! {
            result = verify::verify_deployment(
                &descriptor,
                &rpc_url,
                &manifest,
                &policy.bytes,
                args.verifier.pccs_url.as_deref(),
            ) => result?,
            _ = tokio::signal::ctrl_c() => {
                println!(
                    "\nInterrupted during deploy verification — the config is delivered. \
                     Appraise the node with:\n    seismic-tee node verify{} --manifest {}{}",
                    args.node.as_flags(),
                    manifest_path.display(),
                    verify::retry_flags(&args.policy_source, &args.verifier),
                );
                return Ok(ExitCode::from(130));
            }
        }
    }
    print_summary(fqdn, public_ip, &record);
    if policy.is_none() {
        // --no-verify: the node is up but unappraised, and the appraisal has
        // its own command.
        next_step::print(
            "appraise it before relying on it:",
            &[format!(
                "seismic-tee node verify{} --manifest {}{}",
                args.node.as_flags(),
                manifest_path.display(),
                verify::retry_flags(&args.policy_source, &args.verifier),
            )],
        );
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use seismic_tee_common::network_dir::MANIFEST_FILENAME;

    use super::*;
    use crate::test_support::{FIXTURE_MANIFEST, FakeServer, refused_url, write_file};

    const FQDN: &str = "node1.example.com";
    const EMAIL: &str = "ops@example.com";
    const EXTERNAL_IP: &str = "203.0.113.7";
    /// chainId matches the fixture manifest's eth.chain_id.
    const RETH_GENESIS: &[u8] = br#"{"config": {"chainId": 5124}, "alloc": {}}"#;
    /// namespace matches the fixture manifest's summit.namespace.
    const SUMMIT_GENESIS: &[u8] = b"namespace = \"seismic-devnet-3\"\nvalidators = []\n";

    fn bootnode() -> String {
        format!("enode://{}@198.51.100.1:30303", "ab".repeat(64))
    }

    fn manifest() -> Manifest {
        Manifest::from_json_bytes(FIXTURE_MANIFEST).unwrap()
    }

    fn build(
        manifest: &Manifest,
        reth_genesis: &[u8],
        summit_genesis: &[u8],
        genesis_node: bool,
        bootnodes: &[String],
    ) -> anyhow::Result<InitConfig> {
        build_config(&ConfigInputs {
            manifest,
            reth_genesis: &Artifact::new("/nets/devnet/reth-genesis.json", reth_genesis),
            summit_genesis: &Artifact::new("/nets/devnet/summit-genesis.toml", summit_genesis),
            bootnodes,
            external_ip: EXTERNAL_IP,
            fqdn: FQDN,
            email: EMAIL,
            genesis_node,
        })
    }

    fn decode(b64: &str) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap()
    }

    /// genesis: mints root_key locally, so genesis_node=true and (in the
    /// greenfield case) no bootnodes — the key is present and empty. The
    /// rendered document is exactly `[node]` + `[network]`, and the artifacts
    /// round-trip byte-exact through the base64 hop.
    #[test]
    fn genesis_mode_renders_exactly_the_two_sections() {
        let manifest = manifest();
        let config = build(&manifest, RETH_GENESIS, SUMMIT_GENESIS, true, &[]).unwrap();
        let rendered = render_config(&config).unwrap();

        let table: toml::Table = toml::from_str(&rendered).unwrap();
        let mut sections: Vec<_> = table.keys().collect();
        sections.sort();
        assert_eq!(sections, ["network", "node"]);
        assert_eq!(table["node"]["genesis_node"].as_bool(), Some(true));
        assert_eq!(table["node"]["external_ip"].as_str(), Some(EXTERNAL_IP));
        assert_eq!(table["node"]["domain"]["name"].as_str(), Some(FQDN));
        assert_eq!(table["node"]["domain"]["email"].as_str(), Some(EMAIL));
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
        assert!(reparsed.node.genesis_node);
    }

    /// join: genesis_node=false and the bootnode set (root_key fetch peers are
    /// derived from it by tdx-init) survives verbatim — a node's own enode
    /// included, which tdx-init drops when deriving peers.
    #[test]
    fn join_mode_carries_the_bootnodes_verbatim() {
        let bootnodes = [
            bootnode(),
            format!("enode://{}@203.0.113.7:30303", "cd".repeat(64)),
        ];
        let config = build(&manifest(), RETH_GENESIS, SUMMIT_GENESIS, false, &bootnodes).unwrap();

        assert!(!config.node.genesis_node);
        assert_eq!(config.node.external_ip, EXTERNAL_IP);
        assert_eq!(config.network.bootnodes, bootnodes);

        let reparsed: InitConfig = toml::from_str(&render_config(&config).unwrap()).unwrap();
        assert_eq!(reparsed.network.bootnodes, bootnodes);
    }

    /// tdx-init requires [node].external_ip and parses it as an IpAddr, so an
    /// empty value is a far-end 400 — fail fast client-side instead.
    #[test]
    fn an_empty_external_ip_is_rejected() {
        let manifest = manifest();
        let error = build_config(&ConfigInputs {
            manifest: &manifest,
            reth_genesis: &Artifact::new("reth-genesis.json", RETH_GENESIS),
            summit_genesis: &Artifact::new("summit-genesis.toml", SUMMIT_GENESIS),
            bootnodes: &[],
            external_ip: "",
            fqdn: FQDN,
            email: EMAIL,
            genesis_node: true,
        })
        .unwrap_err()
        .to_string();
        assert!(error.contains("external_ip"), "{error}");
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
        let manifest_path = dir.path().join(MANIFEST_FILENAME);

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

        post_config(&client, &format!("{}/", server.url), &config)
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
        assert_eq!(received.node.domain.name, FQDN);
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

        let error = post_config(&client, &format!("{}/", server.url), &config)
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

    /// The runbook's invocation, and the flags it leaves at their defaults.
    #[test]
    fn the_runbooks_argv_parses_with_the_documented_defaults() {
        let probe = Probe::try_parse_from([
            "configure",
            "--node",
            "nodes/nodes.json",
            "--name",
            "tmp-devnet-1-2",
            "--bootnode",
            "enode://ab@1.2.3.4:30303",
            "--bootnode",
            "enode://cd@5.6.7.8:30303",
            "--manifest",
            "network-manifest.json",
        ])
        .unwrap()
        .args;

        assert_eq!(probe.bootnode.len(), 2);
        assert_eq!(probe.email, DEFAULT_EMAIL);
        assert!(!probe.no_verify);
        assert_eq!(probe.yes, None, "the gate is on unless asked off");
        assert_eq!(probe.dump_config, None);
        assert_eq!(probe.reth_genesis, None);
        assert_eq!(
            probe.policy_source.attestation_type,
            verify::DEFAULT_ATTESTATION_TYPE
        );

        // A joiner has no root_key of its own: the bootnode is required.
        assert!(
            Probe::try_parse_from(["configure", "--node", "n.json", "--manifest", "m.json"])
                .is_err()
        );
    }

    /// The preview names what the base64 hides: the manifest by identity,
    /// the genesis files by hash, the bootnodes, the policy, the record.
    #[test]
    fn the_preview_decodes_what_the_toml_hides() {
        let manifest = manifest();
        let bootnodes = [bootnode()];
        let inputs = ConfigInputs {
            manifest: &manifest,
            reth_genesis: &Artifact::new("/nets/devnet/reth-genesis.json", RETH_GENESIS),
            summit_genesis: &Artifact::new("/nets/devnet/summit-genesis.toml", SUMMIT_GENESIS),
            bootnodes: &bootnodes,
            external_ip: EXTERNAL_IP,
            fqdn: FQDN,
            email: EMAIL,
            genesis_node: false,
        };
        let preview = Preview {
            name: "dev-2",
            manifest_path: Path::new("/nets/devnet/network-manifest.json"),
            inputs: &inputs,
            policy: Some("/nets/devnet/measurement-policy-bootstrap.json (pinned by --manifest)"),
            record: Path::new("/nets/devnet/nodes/dev-2.init-config.toml"),
            post_url: "http://203.0.113.7:8080/",
        };

        let text = render_preview(&preview);
        for expected in [
            "node dev-2",
            "http://203.0.113.7:8080/",
            "node1.example.com (203.0.113.7), role: join",
            EMAIL,
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
            "Bootnodes:        1",
            &bootnode(),
            "measurement-policy-bootstrap.json (pinned by",
            "/nets/devnet/nodes/dev-2.init-config.toml",
        ] {
            assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
        }

        let unappraised = render_preview(&Preview {
            policy: None,
            ..preview
        });
        assert!(unappraised.contains("--no-verify"), "{unappraised}");
    }

    /// A `--yes` naming the resolved node sends without reading; one naming
    /// something else is refused, naming both; a terminal is asked and only
    /// y/yes sends; no terminal and no `--yes` is a refusal that names the
    /// flag's new shape.
    #[test]
    fn confirm_post_gates_the_post() {
        let mut empty = std::io::Cursor::new(b"".to_vec());
        assert!(confirm_post(Some("dev-2"), "dev-2", false, &mut empty).unwrap());
        assert!(confirm_post(Some("dev-2"), "dev-2", true, &mut empty).unwrap());

        let error = confirm_post(Some("wrong-name"), "dev-2", false, &mut empty)
            .unwrap_err()
            .to_string();
        assert!(error.contains("--yes wrong-name"), "{error}");
        assert!(error.contains("dev-2"), "{error}");

        let error = confirm_post(None, "dev-2", false, &mut empty)
            .unwrap_err()
            .to_string();
        assert!(error.contains("--yes <NAME>"), "{error}");
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
                confirm_post(None, "dev-2", true, &mut input).unwrap(),
                sends,
                "{answer:?}"
            );
        }
    }

    /// The record lands in the network directory's `nodes/` tier by default,
    /// `--dump-config` puts it anywhere, and writing creates the tier.
    #[test]
    fn the_record_defaults_to_nodes_beside_the_manifest_and_is_written() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = dir.path().join(MANIFEST_FILENAME);

        let default = resolve_record_path(None, &manifest_path, "dev-2");
        assert_eq!(
            default,
            dir.path().join("nodes").join("dev-2.init-config.toml")
        );
        let explicit = Path::new("/elsewhere/dev-2.toml");
        assert_eq!(
            resolve_record_path(Some(explicit), &manifest_path, "dev-2"),
            explicit
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
        write_record(&default, &rendered).unwrap();
        assert_eq!(std::fs::read_to_string(&default).unwrap(), rendered);
        // And it is the document tdx-init would take.
        let reparsed: InitConfig = toml::from_str(&rendered).unwrap();
        assert_eq!(reparsed.network.bootnodes, [bootnode()]);
    }
}
