//! The configure run: found a network, or join nodes to a live one, in one
//! command; and `configure --check`, re-asserting the launch of one founded.
//!
//! ```text
//! seismic-tee node configure --manifest tee/networks/devnet-3/network-manifest.json
//! ```
//!
//! [`configure`] probes the node table, lets [`crate::plan`] decide the run,
//! checks what the run relies on, shows one preview and asks once, then
//! configures every node it names on one dashboard.
//!
//! Every founding box minted a candidate `root_key` at boot and the manifest
//! pins one box's `tx_io_pk@0` (`founding_tx_io_pk`): that box keeps its
//! candidate, and every other discards its own and fetches `root_key` from
//! it via `getWrappedRootKey`. So a founding is the pinned box first, its
//! harvest-style quote DCAP-verified again before anything is sent. Bootnode
//! bootstrap is inherently two-stage on a greenfield cohort: a node's reth
//! enode isn't known until reth is up, so there is nothing to hand the rest
//! as `[network].bootnodes` up front. So:
//!
//! - Stage 1 — configure only the pinned box (empty `bootnodes`), then poll
//!   its `seismic_nodeInfo` until reth reports an enode.
//! - Stage 2 — configure the rest in parallel with `bootnodes = [pinned
//!   enode]`.
//!
//! A pinned box that fails stops the run: each box takes one POST per boot,
//! so the founding is redone rather than retried around it. A run against a
//! live network — the rest of a partly failed founding, a founding box
//! rebooted, a node added later — is one stage, through the key holders that
//! pass their appraisal ([`crate::bootnodes`]).
//!
//! Root-key bootstrap rides the same list: tdx-init derives each node's
//! root-key fetch peers from its POSTed bootnodes (`http://<host>:7878`, the
//! node's own entry dropped), so there is no second peer list that could skew
//! from the bootnode set.
//!
//! Every node is deploy-verified as soon as it reaches a ready state — the
//! `seismic-tee node verify` check, run as its own task so a fast box is
//! appraised while a slow one still wipes its disk. The founder is a relying
//! party the moment stage 2 hands the pinned box's enode to the rest, so its
//! gate lands before that. The verdict is the node's result: a node that
//! fails its appraisal counts as failed, and the launch assertions never run.
//!
//! A founding delivers the summit genesis with current IPs spliced in: the
//! assembled `summit-genesis.toml` is a founding-era snapshot whose
//! `[[validators]].ip_address` entries are network topology, not identity —
//! excluded from the manifest's config digest — so a founding replaces them
//! with the live IPs from the node table without re-serializing any other
//! field. After every node accepts its config, the launch assertions (see
//! [`crate::launch`]) verify each node with a harvest record against what
//! the manifest pins.
//!
//! `--check` runs the launch assertions and nothing else: no config is built
//! or POSTed, nothing is written, and every founding node — every box with a
//! harvest record — is held to the manifest's pins as it stands now. It is
//! `assemble --check`'s sibling one step on: that holds the artifact set on
//! disk to its inputs, this holds the live cohort to the artifact set. A
//! founder runs it after a launch some boxes had not settled by, after a
//! reboot or a re-image, or whenever the cohort may have drifted from what
//! was pinned. It waits [`launch::CHECK_TIMEOUT`], not the founding's
//! readiness window: the cohort is supposed to be up, so a node that does not
//! answer is reported as such and the check is re-run once it is.

use std::collections::{BTreeMap, BTreeSet};
use std::io::IsTerminal as _;
use std::net::SocketAddr;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use seismic_tee_common::founding::{
    FoundingRecord, FoundingRecords, SUMMIT_CONSENSUS_PORT, box_holding_the_pin,
    load_harvest_records,
};
use seismic_tee_common::http::{HARVEST_PORT, TDX_INIT_PORT};
use seismic_tee_common::{
    Artifact, Descriptors, Manifest, NetworkDir, NodeDescriptor, hex_0x, http, next_step, rpc,
};
use seismic_verify_quote::SeismicMeasurementPolicy;
use tdx_init_config::Bootnode;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::bootnodes;
use crate::configure::{
    ConfigInputs, ConfigureArgs, Inputs, Preview, TDX_INIT_LISTENER_TIMEOUT,
    TDX_INIT_RETRY_INTERVAL, build_config, confirm_run, post_config_within, remedy, render_config,
    render_preview, resolve_reth_genesis, resolve_summit_genesis, write_record,
};
use crate::dashboard::CohortDashboard;
use crate::harvest::{FetchError, HarvestTarget, build_record, fetch_quote, verify_quote};
use crate::launch::{self, LaunchTarget};
use crate::load_manifest;
use crate::plan::{self, Pinned, Plan, Planned, Remedy, Run, Through, Up};
use crate::probe::{self, Founding, probe_all};
use crate::status::{POLL_INTERVAL, ProvisioningWatch, poll_provisioning};
use crate::verify::{check_policy_source_files, resolve_policy};

/// A whole cohort's wipes tend to finish together, so every challenge hits the
/// PCCS at once — and a DCAP collateral fetch is the one transient way a
/// challenge fails. Retry a few times before the verdict is terminal: a
/// founding that a blip fails is expensive to recover (config delivery is once
/// per boot), while a real measurement mismatch just fails every attempt.
pub const APPRAISAL_ATTEMPTS: u32 = 3;
pub const APPRAISAL_RETRY: Duration = Duration::from_secs(15);

/// How far one node's configure got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Ready, and passed its appraisal (no appraisal under `--no-verify`:
    /// ready is the whole bar).
    Ready,
    /// Failed before its config was sent: a re-run configures it.
    NotSent,
    /// Failed once its config was sent, or may have been: tdx-init takes one
    /// POST per boot, so a re-run skips it.
    Failed,
}

/// One node of the run, as the plan parsed it.
///
/// `bootnodes` (→ `[network].bootnodes`: reth p2p + tdx-init's derived
/// root-key fetch peers) is assigned by the run: empty for the pinned box,
/// `[pinned enode]` for the rest of a founding, the vetted key holders'
/// enodes for a join.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The node's key in the node table.
    pub name: String,
    pub descriptor: NodeDescriptor,
    /// The pinned box of a founding.
    pub pinned: bool,
    pub bootnodes: Vec<Bootnode>,
    pub up: Up,
    /// In a founding, the node key harvested from it, checked again right
    /// before its POST.
    pub harvested_node_key: Option<String>,
    /// Its harvest record, when it is a founding box: what the launch
    /// assertions hold it to.
    pub record: Option<FoundingRecord>,
}

impl Node {
    /// The pinned box: awaiting its config, and given no bootnodes.
    fn pinned_box(pinned: Pinned) -> Self {
        Self {
            name: pinned.name,
            descriptor: pinned.descriptor,
            pinned: true,
            bootnodes: Vec::new(),
            up: Up::Awaiting,
            harvested_node_key: Some(pinned.record.node_public_key.clone()),
            record: Some(pinned.record),
        }
    }

    /// Any other node, fetching root_key through `bootnodes`.
    fn through(planned: Planned, bootnodes: Vec<Bootnode>) -> Self {
        Self {
            name: planned.name,
            descriptor: planned.descriptor,
            pinned: false,
            bootnodes,
            up: planned.up,
            harvested_node_key: planned.harvested_node_key,
            record: planned.record,
        }
    }

    fn role(&self) -> &'static str {
        if self.pinned { "pinned box" } else { "join" }
    }
}

/// The current-IP splice map: each harvested box's pinned node pubkey →
/// `<ip>:<consensus port>`, its IP from `descriptors`, the cohort's live node
/// table. Every harvest record must still have a node.
pub fn current_ips(
    records: &FoundingRecords,
    descriptors: &Descriptors,
) -> anyhow::Result<BTreeMap<String, String>> {
    let mut ip_by_node_pubkey = BTreeMap::new();
    for (name, record) in records {
        let Some(descriptor) = descriptors.get(name) else {
            bail!(
                "the cohort has no node {name:?} — the cohort's node table supplies each \
                 founding validator's current IP. A harvested box that is gone from the cohort \
                 means it changed under the founding: re-found rather than configuring",
            );
        };
        ip_by_node_pubkey.insert(
            record.node_public_key.clone(),
            SocketAddr::new(descriptor.public_ip, SUMMIT_CONSENSUS_PORT).to_string(),
        );
    }
    Ok(ip_by_node_pubkey)
}

/// One `ip_address = "…"` line, exactly as summit's emitter renders a
/// validator's entry: the splice rewrites these lines and nothing else.
fn is_ip_address_line(line: &str) -> bool {
    line.strip_prefix("ip_address = \"")
        .and_then(|rest| rest.strip_suffix('"'))
        .is_some_and(|inner| !inner.contains('"'))
}

/// Replace each `[[validators]].ip_address` with the box's current IP.
///
/// The assembled summit genesis is a founding-era snapshot: its validator IPs
/// were the descriptors' at assemble time, and only the IPs are free to change
/// — they are excluded from the config digest the manifest pins, and peers are
/// authenticated by the pinned ed25519 keys, so a stale IP is a liveness
/// problem only. Everything else in the file is identity: the digest hashes
/// the key/credential fields as the exact strings summit emitted, so this must
/// never re-serialize the document. The splice is therefore textual — the i-th
/// `ip_address` line is rewritten for the i-th validator — and self-checked by
/// re-parsing: the spliced file must parse identically to the input everywhere
/// but `ip_address`.
///
/// `ip_by_node_pubkey` must cover the validator set exactly: a pinned
/// validator without a current IP (or an IP for a key the genesis doesn't pin)
/// means the cohort changed under the founding — re-found rather than
/// delivering a genesis that strands a pinned peer.
pub fn splice_validator_ips(
    genesis: &[u8],
    ip_by_node_pubkey: &BTreeMap<String, String>,
) -> anyhow::Result<Vec<u8>> {
    let text = std::str::from_utf8(genesis).context("summit genesis is not valid TOML")?;
    let parsed: toml::Table = toml::from_str(text).context("summit genesis is not valid TOML")?;
    let validators = match parsed.get("validators").and_then(toml::Value::as_array) {
        Some(validators) if !validators.is_empty() => validators,
        _ => bail!(
            "summit genesis carries no [[validators]] — not an assembled artifact (assemble refuses \
             an empty founding set)"
        ),
    };
    let mut pinned = Vec::with_capacity(validators.len());
    for (i, entry) in validators.iter().enumerate() {
        match entry.get("node_public_key").and_then(toml::Value::as_str) {
            Some(key) => pinned.push(key.to_string()),
            None => bail!(
                "summit genesis validator #{} has no node_public_key string",
                i + 1
            ),
        }
    }
    let pinned_set: BTreeSet<&str> = pinned.iter().map(String::as_str).collect();
    let harvested_set: BTreeSet<&str> = ip_by_node_pubkey.keys().map(String::as_str).collect();
    if pinned_set != harvested_set {
        bail!(
            "the summit genesis's pinned validator set and the founding inputs (inputs/harvest/ + \
             the cohort's node table) disagree — the cohort changed under the founding; re-found \
             rather than delivering a genesis that strands a pinned peer:\n    pinned node keys:    \
             {}\n    harvested node keys: {}",
            pinned_set.iter().copied().collect::<Vec<_>>().join(", "),
            harvested_set.iter().copied().collect::<Vec<_>>().join(", ")
        );
    }

    let mut spliced = String::with_capacity(text.len());
    let mut lines_seen = 0usize;
    for line in text.split_inclusive('\n') {
        let body = line.strip_suffix('\n').unwrap_or(line);
        if is_ip_address_line(body) {
            if let Some(key) = pinned.get(lines_seen) {
                // A JSON string is a valid TOML basic string for "<ip>:<port>".
                spliced.push_str(&format!(
                    "ip_address = {}",
                    serde_json::to_string(&ip_by_node_pubkey[key]).expect("a string serializes")
                ));
                if line.ends_with('\n') {
                    spliced.push('\n');
                }
            }
            lines_seen += 1;
        } else {
            spliced.push_str(line);
        }
    }
    if lines_seen != validators.len() {
        bail!(
            "summit genesis has {} validator(s) but {lines_seen} ip_address line(s) — not the \
             layout summit's emitter renders; refusing to splice",
            validators.len()
        );
    }

    // Self-check: parse-identical to the input everywhere but ip_address.
    let mut expected = parsed.clone();
    expected.insert(
        "validators".to_string(),
        toml::Value::Array(
            validators
                .iter()
                .zip(&pinned)
                .map(|(entry, key)| {
                    let mut entry = entry.clone();
                    if let Some(table) = entry.as_table_mut() {
                        table.insert(
                            "ip_address".to_string(),
                            toml::Value::String(ip_by_node_pubkey[key].clone()),
                        );
                    }
                    entry
                })
                .collect(),
        ),
    );
    let reparsed: toml::Table =
        toml::from_str(&spliced).context("the spliced summit genesis does not parse")?;
    if reparsed != expected {
        bail!(
            "IP splice changed the summit genesis beyond ip_address — refusing to deliver it (the \
             config digest pins every other field)"
        );
    }
    Ok(spliced.into_bytes())
}

/// The deploy-verification inputs shared by the whole run: the measurement
/// policy resolved once up front, and what steers the verifier. Resolving the
/// policy is per-run work, not per-node, and it must fail before any node is
/// touched — config delivery is once per boot.
#[derive(Debug, Clone)]
pub struct Appraisal {
    pub policy: Vec<u8>,
    pub pccs_url: Option<String>,
}

/// Everything the run's nodes share.
pub struct Shared {
    pub manifest: Manifest,
    pub reth_genesis: Artifact,
    /// The delivered summit genesis: in a founding, the artifact-set copy
    /// with current IPs spliced in.
    pub summit_genesis: Artifact,
    /// `None` under `--no-verify`.
    pub appraisal: Option<Appraisal>,
    pub dir: NetworkDir,
    /// The plain-HTTP client: tdx-init's config receiver and the harvest ports.
    /// JSON-RPC endpoints are reached through [`rpc::Client`] instead.
    pub client: reqwest::Client,
}

/// Where a node's task reports its latest status line to the dashboard.
type StatusTx = mpsc::UnboundedSender<(String, String)>;

fn report_status(status: &StatusTx, node: &Node, line: impl Into<String>) {
    let _ = status.send((node.name.clone(), line.into()));
}

/// Deploy-verify one ready node, recording the verdict as its status line.
///
/// The verdict is the node's result, not a warning beside it: an unappraised
/// box must not be handed to the rest as a bootnode, or counted as a
/// configured node.
async fn appraise(node: &Node, shared: &Shared, appraisal: &Appraisal, status: &StatusTx) -> bool {
    let endpoint = node.descriptor.attestation_rpc_url();
    let mut attempt = 1;
    loop {
        report_status(status, node, "deploy-verifying…");
        match crate::verify::challenge_node(
            &endpoint,
            &shared.manifest,
            &appraisal.policy,
            appraisal.pccs_url.as_deref(),
        )
        .await
        {
            Ok(_) => {
                report_status(status, node, "ready, deploy-verified ✓");
                return true;
            }
            Err(error) if attempt == APPRAISAL_ATTEMPTS => {
                report_status(
                    status,
                    node,
                    format!("ERROR: deploy verification FAILED: {error:?}"),
                );
                return false;
            }
            Err(_) => {
                report_status(
                    status,
                    node,
                    format!(
                        "appraisal attempt {attempt}/{APPRAISAL_ATTEMPTS} failed, retrying in {}s…",
                        APPRAISAL_RETRY.as_secs()
                    ),
                );
                tokio::time::sleep(APPRAISAL_RETRY).await;
                attempt += 1;
            }
        }
    }
}

/// Right before a node's POST: wait for its `:7879` to answer, within
/// tdx-init's listener patience, then gate it as the plan gated the rest: a
/// node that took a config this boot is sent no other, and a founding box
/// serving a node key other than its harvest record's rebooted since the
/// harvest.
async fn await_config_window(
    client: &reqwest::Client,
    target: &HarvestTarget,
    harvested_node_key: Option<&str>,
    timeout: Duration,
    interval: Duration,
) -> anyhow::Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        match fetch_quote(client, target).await {
            Ok(quote) => match harvested_node_key {
                Some(harvested) if quote.node_public_key != harvested => bail!(
                    "serves node key {}, harvested {harvested}: it rebooted since the harvest, so \
                     its consensus slot is dead. Nothing was sent to it; re-found.",
                    quote.node_public_key
                ),
                _ => return Ok(()),
            },
            Err(FetchError::WindowClosed(_)) => {
                bail!("took a config POST this boot, from another run; nothing was sent to it")
            }
            Err(error @ (FetchError::Rejected(..) | FetchError::Malformed(_))) => bail!(
                "{error}: the image and this CLI disagree on the harvest API; nothing was sent to it"
            ),
            Err(error) if Instant::now() >= deadline => bail!(
                ":{HARVEST_PORT} did not answer within {}s ({error}); nothing was sent to it",
                timeout.as_secs()
            ),
            Err(_) => tokio::time::sleep(interval).await,
        }
    }
}

/// Build + POST one node's config, poll its LUKS wipe, then deploy-verify it,
/// reporting the latest status line for the dashboard. Returns how far the
/// node got. Never fails — a failure is reported as the node's status and
/// reflected in the return, so one bad node doesn't abort the rest of the
/// run.
///
/// A node not up at the probe, and every founding box, is gated first
/// ([`await_config_window`]): the rest of a founding wait out the pinned
/// box's wipe, long enough to reboot and mint fresh keys since the probe.
async fn configure_node(node: Node, shared: Arc<Shared>, status: StatusTx) -> (String, Outcome) {
    // Set once the POST is attempted: a failure before it sent nothing.
    let mut sent = false;
    let outcome = async {
        if node.up == Up::NotUp || node.harvested_node_key.is_some() {
            report_status(
                &status,
                &node,
                match node.up {
                    Up::NotUp => format!("not up yet: waiting for :{HARVEST_PORT} to answer…"),
                    Up::Awaiting => "checking it still serves its harvested node key…".to_string(),
                },
            );
            await_config_window(
                &shared.client,
                &HarvestTarget::new(&node.name, &node.descriptor),
                node.harvested_node_key.as_deref(),
                TDX_INIT_LISTENER_TIMEOUT,
                TDX_INIT_RETRY_INTERVAL,
            )
            .await?;
        }
        report_status(&status, &node, "building config…");
        let config = build_config(&ConfigInputs {
            manifest: &shared.manifest,
            reth_genesis: &shared.reth_genesis,
            summit_genesis: &shared.summit_genesis,
            bootnodes: &node.bootnodes,
            external_ip: node.descriptor.public_ip,
            pinned_box: node.pinned,
        })?;
        // The record first: what is about to be sent exists on disk before
        // anything is sent, as the record of what the node booted with.
        write_record(
            &shared.dir.init_config(&node.name),
            &render_config(&config)?,
        )?;
        let descriptor = &node.descriptor;
        let rpc_client = rpc::Client::new(&descriptor.attestation_rpc_url())?;

        report_status(
            &status,
            &node,
            format!("POSTing config to tdx-init :{TDX_INIT_PORT}…"),
        );
        sent = true;
        post_config_within(
            &shared.client,
            &descriptor.tdx_init_url(),
            &config,
            TDX_INIT_LISTENER_TIMEOUT,
            TDX_INIT_RETRY_INTERVAL,
            |line| report_status(&status, &node, line),
        )
        .await?;

        let ready = poll_provisioning(
            &rpc_client,
            ProvisioningWatch::default(),
            POLL_INTERVAL,
            |update| report_status(&status, &node, update.line.clone()),
        )
        .await;
        if !ready {
            return anyhow::Ok(false);
        }
        match &shared.appraisal {
            None => Ok(true),
            Some(appraisal) => Ok(appraise(&node, &shared, appraisal, &status).await),
        }
    }
    .await;
    let outcome = match outcome {
        Ok(true) => Outcome::Ready,
        Ok(false) => Outcome::Failed,
        Err(error) => {
            report_status(&status, &node, format!("ERROR: {error:#}"));
            if sent {
                Outcome::Failed
            } else {
                Outcome::NotSent
            }
        }
    };
    (node.name, outcome)
}

/// Configure and appraise every node concurrently, refreshing the dashboard
/// until all tasks finish. Returns `{node name: outcome}`, plus each failed
/// node's full status printed after the dashboard (which folds and truncates
/// a status to one row, and a failure's reason — often a verifier's
/// multi-line chain — is the one thing the run must not swallow).
///
/// Dropping the future aborts every task at its next await (the `JoinSet`
/// goes with it): that is how [`configure`]'s Ctrl-C stops the run.
pub async fn run_cohort(
    nodes: &[Node],
    shared: &Arc<Shared>,
) -> anyhow::Result<BTreeMap<String, Outcome>> {
    let mut states: BTreeMap<String, String> = nodes
        .iter()
        .map(|n| (n.name.clone(), "queued…".to_string()))
        .collect();
    let mut dashboard = CohortDashboard::new(
        nodes
            .iter()
            .map(|n| {
                (
                    n.name.clone(),
                    if n.pinned {
                        format!("{} (pinned box)", n.name)
                    } else {
                        n.name.clone()
                    },
                )
            })
            .collect(),
    );
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut tasks = JoinSet::new();
    for node in nodes {
        tasks.spawn(configure_node(node.clone(), Arc::clone(shared), tx.clone()));
    }
    drop(tx);

    let mut results = BTreeMap::new();
    let mut tick = tokio::time::interval(Duration::from_secs(1));
    while results.len() < nodes.len() {
        tokio::select! {
            Some((name, line)) = rx.recv() => {
                states.insert(name, line);
            }
            Some(joined) = tasks.join_next() => {
                match joined {
                    Ok((name, outcome)) => {
                        results.insert(name, outcome);
                    }
                    Err(error) => {
                        // A panic in a node's task is that node's failure, not
                        // the run's.
                        let name = error.to_string();
                        eprintln!("a node task ended abnormally: {name}");
                    }
                }
                if tasks.is_empty() {
                    // Drain what the finished tasks reported.
                    while let Ok((name, line)) = rx.try_recv() {
                        states.insert(name, line);
                    }
                    break;
                }
            }
            _ = tick.tick() => {
                dashboard.render(&states);
            }
        }
    }
    dashboard.render(&states); // final paint of terminal states
    for node in nodes {
        // A task that ended abnormally has no result: it failed, and may have
        // sent its config.
        let outcome = *results.entry(node.name.clone()).or_insert(Outcome::Failed);
        if outcome != Outcome::Ready {
            println!("\n{}: {}", node.name, states[&node.name]);
        }
    }
    Ok(results)
}

/// Two-stage bootstrap: the pinned box first (so its enode exists), then the
/// rest pointed at it. Returns `{node name: outcome}` across both stages.
///
/// Stage 1 ends with the pinned box ready *and* appraised, which is the gate
/// the stage boundary exists for: its enode is what every other node dials.
/// If the pinned box fails either half, the rest aren't configured (they'd
/// have neither a root_key source nor a bootnode); the missing results read
/// as failures in the report. If it serves no enode, the rest are sent
/// nothing either, and read as such: a re-run joins them through it once it
/// serves one.
pub async fn bootstrap_founding(
    pinned: &Node,
    rest: &mut [Node],
    shared: &Arc<Shared>,
) -> anyhow::Result<BTreeMap<String, Outcome>> {
    println!(
        "Stage 1/2: configuring the pinned box {} (no bootnodes yet)...",
        pinned.name
    );
    let mut results = run_cohort(std::slice::from_ref(pinned), shared).await?;
    if results[&pinned.name] != Outcome::Ready {
        println!(
            "The pinned box failed in stage 1 — not configuring the rest: it holds the only copy \
             of the network's root_key, and its enode is what every other node would dial."
        );
        return Ok(results);
    }
    if rest.is_empty() {
        return Ok(results);
    }

    println!("Stage 2/2: fetching the pinned box's enode via seismic_nodeInfo...");
    let enodes = match bootnodes::collect_enodes(
        &[(pinned.name.clone(), pinned.descriptor.eth_rpc_url())],
        bootnodes::ENODE_TIMEOUT,
        bootnodes::POLL_INTERVAL,
    )
    .await
    {
        Ok(enodes) => enodes,
        Err(error) => {
            println!(
                "The pinned box serves no enode — not configuring the rest, which would have no \
                 bootnode: {error:#}"
            );
            results.extend(
                rest.iter()
                    .map(|node| (node.name.clone(), Outcome::NotSent)),
            );
            return Ok(results);
        }
    };
    let pinned_enode = enodes[&pinned.name].clone();
    bootnodes::warn_on_ip_mismatch(&pinned_enode, pinned.descriptor.public_ip, &pinned.name);

    println!(
        "Stage 2/2: configuring {} node(s) off the pinned box's enode...",
        rest.len()
    );
    for node in rest.iter_mut() {
        node.bootnodes = vec![pinned_enode.clone()];
    }
    results.extend(run_cohort(rest, shared).await?);
    Ok(results)
}

/// The run's report, and the failure it ends in if any node did not make it.
///
/// What to run next depends on how far each failed node got: one sent
/// nothing and is configured by a re-run, one that took its config is
/// re-appraised with `verify`, and a pinned box that failed once sent is a
/// re-founding whatever the rest did.
fn report(
    nodes: &[&Node],
    skipped: &[String],
    results: &BTreeMap<String, Outcome>,
    args: &ConfigureArgs,
) -> anyhow::Result<()> {
    let remedy = remedy(args);
    // A node with no result was never reached: the pinned box failed first.
    let outcome = |node: &Node| results.get(&node.name).copied().unwrap_or(Outcome::NotSent);
    let rule = "=".repeat(80);
    println!("\n{rule}\nNODES CONFIGURED\n{rule}");
    for node in nodes {
        println!(
            "  {} {} ({}) — {}",
            if outcome(node) == Outcome::Ready {
                "✓"
            } else {
                "✗"
            },
            node.name,
            node.role(),
            node.descriptor.eth_rpc_url()
        );
    }
    for name in skipped {
        println!("  - {name} — skipped: took its config this boot");
    }
    println!("{rule}\n");

    let ended = |wanted: Outcome| -> Vec<&str> {
        nodes
            .iter()
            .filter(|node| outcome(node) == wanted)
            .map(|node| node.name.as_str())
            .collect()
    };
    let (not_sent, sent) = (ended(Outcome::NotSent), ended(Outcome::Failed));
    if not_sent.is_empty() && sent.is_empty() {
        return Ok(());
    }
    if nodes
        .iter()
        .any(|node| node.pinned && outcome(node) == Outcome::Failed)
    {
        println!(
            "The pinned box failed. It holds the only copy of the root_key the manifest pins, and \
             a box that failed once sent its config, or its appraisal, cannot be trusted with it: \
             {}, rather than retrying around it.",
            remedy.re_found()
        );
    } else {
        if !not_sent.is_empty() {
            println!(
                "A node that was sent nothing is configured by a re-run, once the reason printed \
                 above is dealt with:\n    {}\n",
                rerun(args, &remedy, &not_sent)
            );
        }
        if !sent.is_empty() && !args.no_verify {
            // tdx-init takes one config POST per boot, so a node that took
            // its config and then failed its appraisal is re-appraised, not
            // re-configured. Its full verifier reason was printed above.
            println!(
                "A node that took its config but did not pass the appraisal is retried with \
                 `verify`, not with a second `configure` (tdx-init takes one config POST per \
                 boot):\n    {}\n",
                remedy.verify("<node>"),
            );
        }
    }
    let failed: Vec<&str> = not_sent.into_iter().chain(sent).collect();
    bail!(
        "{}/{} node(s) failed: {}",
        failed.len(),
        nodes.len(),
        failed.join(", ")
    );
}

/// This run's `configure` over `names`: the table, manifest, named peers,
/// and appraisal it ran with.
fn rerun(args: &ConfigureArgs, remedy: &Remedy, names: &[&str]) -> String {
    let bootnodes: String = args
        .bootnode
        .iter()
        .map(|enode| format!(" --bootnode {enode}"))
        .collect();
    format!(
        "{}{bootnodes}{}{}",
        remedy.configure(names),
        if args.no_verify { " --no-verify" } else { "" },
        remedy.appraisal,
    )
}

/// What the run checks its own decisions against: the appraisal policy, or
/// under `--no-verify` — which skips appraising the nodes configured only —
/// the network's own, beside --manifest (`--no-verify` refuses --policy and
/// --measurements). A join can name its peer instead; a founding cannot.
fn checks_policy(
    args: &ConfigureArgs,
    appraisal: Option<&Appraisal>,
    manifest_path: &Path,
    manifest: &Manifest,
    founding: bool,
    remedy: &Remedy,
) -> anyhow::Result<Appraisal> {
    if let Some(appraisal) = appraisal {
        return Ok(appraisal.clone());
    }
    let path = NetworkDir::of_manifest(manifest_path).policy();
    if !path.is_file() {
        let or_name_a_peer = if founding {
            String::new()
        } else {
            format!(", or name a peer yourself:\n    {}", remedy.bootnode())
        };
        bail!(
            "measurement policy not found: {}\nThe pinned box's quote before a founding, and the \
             key holders before a join, are checked against the network's policy beside \
             --manifest; --no-verify skips appraising the nodes configured only. Nothing was \
             sent. Instead, fetch the network directory (`seismic-tee ctx \
             set-network`){or_name_a_peer}",
            path.display(),
        );
    }
    let policy = resolve_policy(&args.policy_source, manifest_path, manifest, false)?;
    Ok(Appraisal {
        policy: policy.bytes,
        pccs_url: args.verifier.pccs_url.clone(),
    })
}

/// The network directory's harvest records, when it has a harvest. Read
/// strictly, unlike `node status`'s informational reading: a founding stands
/// on them, so an unreadable record, or a manifest whose pin no harvested box
/// holds, is refused before any node is probed.
fn founding_records(
    dir: &NetworkDir,
    manifest: &Manifest,
) -> anyhow::Result<Option<FoundingRecords>> {
    if !dir.harvest().is_dir() {
        return Ok(None);
    }
    let records = load_harvest_records(dir)?;
    if box_holding_the_pin(&records, &manifest.founding_tx_io_pk).is_none() {
        bail!(
            "no harvested box's candidate is the manifest's founding_tx_io_pk ({}): the manifest \
             was not assembled from the harvest in {}. Nothing was sent. Re-run `seismic-tee \
             network assemble` over this harvest, or re-found.",
            hex_0x(&manifest.founding_tx_io_pk),
            dir.harvest().display()
        );
    }
    Ok(Some(records))
}

/// What to run after Ctrl-C stopped a run that had sent configs: watch the
/// nodes settle, appraise each (a re-run skips the nodes that took their
/// config, so it never appraises them), re-assert a founding's launch, and
/// re-run for the nodes it had not reached.
fn interrupted_steps(args: &ConfigureArgs, remedy: &Remedy, founding: bool) -> Vec<String> {
    let mut steps = vec![remedy.watch(), remedy.verify("<node>")];
    if founding {
        steps.push(format!(
            "seismic-tee node configure --check{}{}",
            remedy.table, remedy.manifest
        ));
    }
    let names: Vec<&str> = args.names.iter().map(String::as_str).collect();
    steps.push(rerun(args, remedy, &names));
    steps
}

/// Before a founding sends anything: the pinned box, asked afresh, still
/// serves the candidate the manifest pins and the node key harvested from it,
/// and its quote over them DCAP-verifies with harvest's own check. The probe
/// that called the network unfounded read the same endpoint unverified.
async fn verify_pinned_box(
    client: &reqwest::Client,
    target: &HarvestTarget,
    founding: &Founding,
    harvested: &FoundingRecord,
    appraisal: &Appraisal,
    remedy: &Remedy,
) -> anyhow::Result<()> {
    let name = &target.name;
    eprintln!("The network is unfounded: DCAP-verifying the pinned box {name}'s quote...");
    let quote = fetch_quote(client, target).await.map_err(|error| {
        anyhow::anyhow!(
            "{name}: the pinned box did not serve its quote: {error}\nNothing was sent; re-run \
             once it answers."
        )
    })?;
    if !founding.pins(&quote.candidate_tx_io_public_key)
        || quote.node_public_key != harvested.node_public_key
    {
        bail!(
            "{name} no longer serves the candidate and node key harvested from it: it rebooted, \
             and the founding root_key is gone. Nothing was sent. Instead, {}.",
            remedy.re_found()
        );
    }
    let policy = SeismicMeasurementPolicy::from_json_bytes(&appraisal.policy)
        .context("loading measurement policy")?;
    verify_quote(
        &build_record(target, &quote),
        &policy,
        appraisal.pccs_url.as_deref(),
    )
    .await
    .map_err(|error| {
        anyhow::anyhow!(
            "{name}: the pinned box's quote does not verify:\n{error:?}\nNothing was sent. A \
             collateral fetch that failed passes on a re-run; a quote that keeps failing is not \
             the measured box harvested: {}.",
            remedy.re_found()
        )
    })?;
    eprintln!("  ✓ {name}: quote DCAP-verified; its candidate is the pin");
    Ok(())
}

/// `configure`: probe the node table, decide, check, preview, ask, run,
/// report, and assert the launch of every founding node configured.
pub async fn configure(args: &ConfigureArgs, config: Option<&Path>) -> anyhow::Result<ExitCode> {
    check_policy_source_files(&args.policy_source, args.no_verify)?;
    // Validate the shared network artifacts once, so a bad one fails fast
    // here rather than as N identical per-node errors mid-dashboard.
    let inputs = Inputs::new(args, config);
    let manifest_path = inputs.manifest()?;
    let manifest = load_manifest(&manifest_path)?;
    let reth_genesis = Artifact::read(&resolve_reth_genesis(
        args.reth_genesis.as_deref(),
        &manifest_path,
    )?)?;
    manifest
        .check_reth_genesis(reth_genesis.bytes())
        .with_context(|| format!("--reth-genesis {}", reth_genesis.path().display()))?;
    let assembled = Artifact::read(&resolve_summit_genesis(
        args.summit_genesis.as_deref(),
        &manifest_path,
    )?)?;
    manifest
        .check_summit_genesis(assembled.bytes())
        .with_context(|| format!("--summit-genesis {}", assembled.path().display()))?;

    // Resolve the policy once for the whole run, before any node is touched:
    // a policy the manifest doesn't commit to, or a rejected measurements
    // file, must fail while the fix still costs nothing — config delivery is
    // once per boot.
    let appraisal = if args.no_verify {
        eprintln!(
            "warning: --no-verify: the nodes configured will not be appraised. Run `seismic-tee \
             node verify` before relying on an unappraised node."
        );
        None
    } else {
        let policy = resolve_policy(&args.policy_source, &manifest_path, &manifest, true)?;
        eprintln!("Appraising against {}", policy.source);
        Some((
            policy.source,
            Appraisal {
                policy: policy.bytes,
                pccs_url: args.verifier.pccs_url.clone(),
            },
        ))
    };
    let (policy_source, appraisal) = appraisal.unzip();

    // The founding inputs live beside the manifest (the network-directory
    // layout): the harvest supplies each box's pinned keys; the node table
    // (the context's, or --node) supplies its current IP.
    let dir = NetworkDir::of_manifest(&manifest_path);
    let (table, table_source) = inputs.table()?;
    let records = founding_records(&dir, &manifest)?;
    let founding = Founding::new(&manifest, records.as_ref());
    let remedy = remedy(args);
    // Collateral fetches go over TLS; see `node verify` for why the provider
    // is chosen here.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let client = http::client()?;

    eprintln!("Reading the {} node(s) in the node table...", table.len());
    let states = probe_all(&client, &probe::targets(&table)).await;
    let Plan { run, skipped } = plan::decide(
        &table,
        &args.names,
        &states,
        &founding,
        records.as_ref(),
        !args.bootnode.is_empty(),
        &remedy,
    )?;
    let Some(run) = run else {
        println!(
            "Nothing to configure: every node named took its config this boot ({}).",
            skipped.join(", ")
        );
        // Only a founding has pins to re-assert.
        let (lead, command) = if records.is_some() {
            (
                "re-assert a founded cohort's launch:",
                "node configure --check",
            )
        } else {
            ("see how they are doing:", "node status")
        };
        next_step::print(
            lead,
            &[format!(
                "seismic-tee {command}{}{}",
                remedy.table, remedy.manifest
            )],
        );
        return Ok(ExitCode::SUCCESS);
    };

    let (summit_genesis, bootnodes) = match &run {
        Run::Found { pinned, .. } => {
            let checks = checks_policy(
                args,
                appraisal.as_ref(),
                &manifest_path,
                &manifest,
                true,
                &remedy,
            )?;
            verify_pinned_box(
                &client,
                &HarvestTarget::new(&pinned.name, &pinned.descriptor),
                &founding,
                &pinned.record,
                &checks,
                &remedy,
            )
            .await?;
            let records = records
                .as_ref()
                .expect("a founding is planned from the harvest records");
            let spliced = splice_validator_ips(assembled.bytes(), &current_ips(records, &table)?)?;
            if spliced != assembled.bytes() {
                eprintln!(
                    "Spliced current node-table IPs into the delivered summit genesis (the \
                     assembled {} is a founding-era snapshot; validator IPs are topology, not \
                     identity)",
                    assembled
                        .path()
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                );
            }
            (Artifact::new(assembled.path(), spliced), Vec::new())
        }
        Run::Join {
            through: Through::KeyHolders(key_holders),
            ..
        } => {
            let checks = checks_policy(
                args,
                appraisal.as_ref(),
                &manifest_path,
                &manifest,
                false,
                &remedy,
            )?;
            let enodes =
                bootnodes::from_key_holders(key_holders, &manifest, &checks, &remedy).await?;
            (assembled, enodes)
        }
        Run::Join {
            through: Through::NamedPeers,
            ..
        } => (assembled, args.bootnode.clone()),
    };

    eprint!(
        "{}",
        render_preview(&Preview {
            table: &table_source,
            manifest_path: &manifest_path,
            manifest: &manifest,
            reth_genesis: &reth_genesis,
            summit_genesis: &summit_genesis,
            policy: policy_source.as_deref(),
            run: &run,
            skipped: &skipped,
            bootnodes: &bootnodes,
            record: &dir.init_config("<name>"),
        })
    );
    // Blocking read on the runtime's main task: nothing else is in flight
    // until the operator answers.
    let stdin = std::io::stdin();
    if !confirm_run(args.yes, stdin.is_terminal(), &mut stdin.lock())? {
        eprintln!("Nothing was sent.");
        return Ok(ExitCode::FAILURE);
    }

    let (pinned, mut rest): (Option<Node>, Vec<Node>) = match run {
        // The rest of a founding gets the pinned box's enode once it is up.
        Run::Found { pinned, rest } => (
            Some(Node::pinned_box(pinned)),
            rest.into_iter()
                .map(|node| Node::through(node, Vec::new()))
                .collect(),
        ),
        Run::Join { nodes, .. } => (
            None,
            nodes
                .into_iter()
                .map(|node| Node::through(node, bootnodes.clone()))
                .collect(),
        ),
    };
    let shared = Arc::new(Shared {
        manifest,
        reth_genesis,
        summit_genesis,
        appraisal,
        dir: dir.clone(),
        client,
    });
    // Only founding boxes have pins for `--check` to re-assert.
    let has_founding_boxes = pinned.iter().chain(&rest).any(|node| node.record.is_some());
    let delivery = async {
        let results = match &pinned {
            Some(pinned) => bootstrap_founding(pinned, &mut rest, &shared).await?,
            None => run_cohort(&rest, &shared).await?,
        };
        let nodes: Vec<&Node> = pinned.iter().chain(&rest).collect();
        report(&nodes, &skipped, &results, args)?;

        // Every node accepted its config — now assert the launch of the
        // founding boxes among them against what the manifest pins (see
        // `launch` for why both are load-bearing).
        let targets: Vec<LaunchTarget> = nodes
            .iter()
            .filter_map(|node| {
                let record = node.record.as_ref()?;
                Some(LaunchTarget::new(&node.name, &node.descriptor, record))
            })
            .collect();
        if !targets.is_empty() {
            assert_launch(
                &shared.client,
                shared.manifest.eth.genesis_hash,
                &targets,
                launch::READY_TIMEOUT,
            )
            .await?;
        }
        anyhow::Ok(())
    };
    // The run's one Ctrl-C handler, around everything after the
    // confirmation: tokio keeps a SIGINT handler for the life of the process,
    // so one installed per stage would leave the waits between stages deaf
    // to it.
    tokio::select! {
        delivered = delivery => delivered?,
        _ = tokio::signal::ctrl_c() => {
            println!("\nStopped watching: the nodes that took their config keep provisioning.");
            next_step::print(
                "watch them settle and appraise each; a re-run configures the nodes not yet sent \
                 theirs, and skips the rest:",
                &interrupted_steps(args, &remedy, has_founding_boxes),
            );
            return Ok(ExitCode::from(130));
        }
    }
    if args.no_verify {
        // The nodes are up but unappraised, and the appraisal has its own
        // command.
        next_step::print(
            "appraise them before relying on them:",
            &pinned
                .iter()
                .chain(&rest)
                .map(|node| remedy.verify(&node.name))
                .collect::<Vec<_>>(),
        );
    } else if let Some(pinned) = &pinned
        && let Some(network) = inputs.network()?
    {
        // The founding is done; what follows is using the network, which
        // has a name to select a node under only when the context supplied
        // the manifest.
        next_step::print(
            "select a node, and point a tool at it:",
            &[
                format!("seismic-tee ctx use {network}/{}", pinned.name),
                "seismic-tee ctx exec -- scast block-number".to_string(),
            ],
        );
    }
    Ok(ExitCode::SUCCESS)
}

/// Both launch assertions, in order, each announced as it starts: every
/// target's reth serves `genesis_hash` as block 0, then every target's
/// harvest endpoint serves its pinned founding keys. `timeout` is how long a target
/// that does not answer is waited for — the founding's readiness window from
/// `configure`, a short one under `--check`.
pub(crate) async fn assert_launch(
    client: &reqwest::Client,
    genesis_hash: [u8; 32],
    targets: &[LaunchTarget],
    timeout: Duration,
) -> anyhow::Result<()> {
    println!(
        "Launch assertion 1/2: every node's reth serves block 0 {}...",
        hex_0x(&genesis_hash)
    );
    launch::assert_cohort_genesis_hash(targets, genesis_hash, timeout, launch::POLL_INTERVAL)
        .await?;
    println!("Launch assertion 2/2: every box serves its pinned summit keys...");
    launch::assert_cohort_summit_keys(client, targets, timeout, launch::POLL_INTERVAL).await?;
    println!("Launch assertions green: the cohort that launched is the cohort pinned.");
    Ok(())
}

/// `--check`: the launch assertions alone, over the founding cohort as it
/// stands now — nothing is built, sent or written.
///
/// The founding cohort is the harvest: every record must still have a node
/// to reach (`current_ips` refuses otherwise, as delivery does), and
/// a node the harvest does not know is not a founding box — it joined later
/// and `node verify` appraises it — so it is named and skipped rather than
/// held to a pin it never had.
pub async fn check(
    args: &ConfigureArgs,
    config: Option<&Path>,
    timeout: Duration,
) -> anyhow::Result<ExitCode> {
    let inputs = Inputs::new(args, config);
    let manifest_path = inputs.manifest()?;
    let manifest = load_manifest(&manifest_path)?;
    let dir = NetworkDir::of_manifest(&manifest_path);
    let (descriptors, _) = inputs.table()?;
    let harvest_records = load_harvest_records(&dir)?;
    current_ips(&harvest_records, &descriptors)?;
    let later: Vec<&str> = descriptors
        .keys()
        .filter(|name| !harvest_records.contains_key(*name))
        .map(String::as_str)
        .collect();
    if !later.is_empty() {
        println!(
            "Skipping {} node(s) with no founding harvest record — not founding boxes, so \
             nothing is pinned for them (`seismic-tee node verify` appraises a later joiner): {}",
            later.len(),
            later.join(", ")
        );
    }
    let targets: Vec<LaunchTarget> = harvest_records
        .iter()
        .map(|(name, record)| LaunchTarget::new(name, &descriptors[name], record))
        .collect();
    println!(
        "Checking {} founding node(s) against the pins of {}",
        targets.len(),
        manifest_path.display()
    );
    // The eth RPC is plain HTTP, but the client still builds a rustls config,
    // which needs a provider; see `node verify` for why this one.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    assert_launch(
        &http::client()?,
        manifest.eth.genesis_hash,
        &targets,
        timeout,
    )
    .await?;
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use seismic_tee_common::network_dir::MANIFEST_FILENAME;
    use seismic_tee_common::test_support::{
        FIXTURE_FOUNDING_TX_IO_PK, FIXTURE_MANIFEST, FakeServer, azure_evidence,
        candidate_tx_io_public_key, consensus_key, refused_url, write_harvest,
    };
    use serde_json::json;

    use super::*;

    const NODE_KEY_1: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const NODE_KEY_2: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[derive(Parser)]
    struct Probe {
        #[command(flatten)]
        args: ConfigureArgs,
    }

    fn parse(argv: &[&str]) -> ConfigureArgs {
        Probe::try_parse_from(std::iter::once(&"configure").chain(argv))
            .unwrap()
            .args
    }

    fn descriptors(nodes: &[(&str, &str, &str)]) -> Descriptors {
        nodes
            .iter()
            .map(|(name, ip, fqdn)| {
                (
                    name.to_string(),
                    NodeDescriptor {
                        public_ip: ip.parse().unwrap(),
                        fqdn: fqdn.parse().unwrap(),
                    },
                )
            })
            .collect()
    }

    fn three() -> Descriptors {
        descriptors(&[
            ("node-1", "1.1.1.1", "n1.example.com"),
            ("node-2", "2.2.2.2", "n2.example.com"),
            ("node-3", "3.3.3.3", "n3.example.com"),
        ])
    }

    fn node(name: &str, pinned: bool) -> Node {
        Node {
            name: name.into(),
            descriptor: NodeDescriptor {
                public_ip: "127.0.0.1".parse().unwrap(),
                fqdn: format!("{name}.example.com").parse().unwrap(),
            },
            pinned,
            bootnodes: Vec::new(),
            up: Up::Awaiting,
            harvested_node_key: None,
            record: None,
        }
    }

    /// The layout summit's emitter renders: scalar parameters first, then one
    /// [[validators]] block per entry with the fields in struct order.
    fn genesis_text() -> String {
        format!(
            "eth_genesis_hash = \"0x{}\"\nleader_timeout_ms = 2000\nnamespace = \"tmp-devnet-1\"\n\n\
             [[validators]]\nnode_public_key = \"{NODE_KEY_1}\"\nconsensus_public_key = \"{}\"\n\
             ip_address = \"192.0.2.1:18551\"\nwithdrawal_credentials = \"0x{}01\"\n\n\
             [[validators]]\nnode_public_key = \"{NODE_KEY_2}\"\nconsensus_public_key = \"{}\"\n\
             ip_address = \"192.0.2.2:18551\"\nwithdrawal_credentials = \"0x{}02\"\n",
            "ab".repeat(32),
            "cc".repeat(48),
            "00".repeat(19),
            "dd".repeat(48),
            "00".repeat(19),
        )
    }

    fn ips(ip1: &str, ip2: &str) -> BTreeMap<String, String> {
        [(NODE_KEY_1, ip1), (NODE_KEY_2, ip2)]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// The splice rewrites the ip_address lines and not one byte more: the
    /// manifest's config digest pins every other field as the exact string
    /// summit emitted.
    #[test]
    fn the_splice_replaces_only_the_ip_address_lines_by_pubkey() {
        let text = genesis_text();
        let spliced = splice_validator_ips(
            text.as_bytes(),
            &ips("198.51.100.7:18551", "192.0.2.2:18551"),
        )
        .unwrap();
        let expected = text.replacen(
            "ip_address = \"192.0.2.1:18551\"",
            "ip_address = \"198.51.100.7:18551\"",
            1,
        );
        assert_eq!(String::from_utf8(spliced).unwrap(), expected);

        // Unchanged IPs round-trip byte-identical.
        assert_eq!(
            splice_validator_ips(text.as_bytes(), &ips("192.0.2.1:18551", "192.0.2.2:18551"))
                .unwrap(),
            text.as_bytes()
        );

        // Assigned by node pubkey, not position: swap the map, the lines swap.
        let spliced =
            splice_validator_ips(text.as_bytes(), &ips("192.0.2.2:18551", "192.0.2.1:18551"))
                .unwrap();
        let spliced = String::from_utf8(spliced).unwrap();
        let first = spliced.find("ip_address = \"192.0.2.2:18551\"").unwrap();
        let second = spliced.find("ip_address = \"192.0.2.1:18551\"").unwrap();
        assert!(first < second);
        assert!(spliced.find(NODE_KEY_1).unwrap() < first);
    }

    #[test]
    fn the_splice_refuses_a_cohort_change_or_an_unknown_layout() {
        let text = genesis_text();
        let mut one = BTreeMap::new();
        one.insert(NODE_KEY_1.to_string(), "192.0.2.1:18551".to_string());
        let err = splice_validator_ips(text.as_bytes(), &one)
            .unwrap_err()
            .to_string();
        assert!(err.contains("cohort changed under the founding"), "{err}");
        assert!(
            err.contains(&format!("pinned node keys:    {NODE_KEY_1}, {NODE_KEY_2}")),
            "{err}"
        );

        let err = splice_validator_ips(b"namespace = \"n\"\nvalidators = []\n", &one)
            .unwrap_err()
            .to_string();
        assert!(err.contains("carries no [[validators]]"), "{err}");

        // Inline-table validators carry no ip_address *lines*.
        let inline = format!(
            "validators = [{{ node_public_key = \"{NODE_KEY_1}\", ip_address = \"192.0.2.1:18551\" }}]\n"
        );
        let err = splice_validator_ips(inline.as_bytes(), &one)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("1 validator(s) but 0 ip_address line(s)"),
            "{err}"
        );
    }

    #[test]
    fn current_ips_join_harvest_keys_with_descriptor_ips() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = NetworkDir::new(tmp.path());
        std::fs::create_dir_all(dir.harvest()).unwrap();
        for (name, key, byte) in [("node-1", NODE_KEY_1, "cc"), ("node-2", NODE_KEY_2, "dd")] {
            std::fs::write(
                dir.harvest_record(name),
                json!({
                    "harvest_nonce": "11".repeat(32),
                    "node_public_key": key,
                    "consensus_public_key": byte.repeat(48),
                    "candidate_tx_io_public_key": format!("02{}", byte.repeat(32)),
                    "evidence": {},
                })
                .to_string(),
            )
            .unwrap();
        }
        let records = load_harvest_records(&dir).unwrap();
        let ips = current_ips(&records, &three()).unwrap();
        assert_eq!(ips[NODE_KEY_1], "1.1.1.1:18551");
        assert_eq!(ips[NODE_KEY_2], "2.2.2.2:18551");
        assert_eq!(ips.len(), 2);

        let one = descriptors(&[("node-1", "1.1.1.1", "n1.example.com")]);
        let err = current_ips(&records, &one).unwrap_err().to_string();
        assert!(err.contains("has no node \"node-2\""), "{err}");
        assert!(err.contains("re-found rather than configuring"), "{err}");
    }

    fn shared(dir: &NetworkDir) -> Shared {
        Shared {
            manifest: Manifest::from_json_bytes(FIXTURE_MANIFEST).unwrap(),
            reth_genesis: Artifact::new(
                "reth-genesis.json",
                br#"{"config": {"chainId": 5124}}"#.to_vec(),
            ),
            summit_genesis: Artifact::new(
                "summit-genesis.toml",
                b"namespace = \"seismic-devnet-3\"\n".to_vec(),
            ),
            appraisal: None,
            dir: dir.clone(),
            client: http::client().unwrap(),
        }
    }

    /// A node that fails fails alone: the run reports it and keeps going.
    #[tokio::test]
    async fn run_cohort_reports_every_node_and_keeps_going_on_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = NetworkDir::new(tmp.path());
        let shared = Arc::new(shared(&dir));
        // A joiner with no bootnodes fails in build_config, before any POST
        // and before its record is written: a re-run configures it.
        let results = run_cohort(&[node("node-2", false)], &shared).await.unwrap();
        assert_eq!(results.get("node-2"), Some(&Outcome::NotSent));
        assert!(!dir.init_config("node-2").exists());
    }

    /// A pinned box that fails stage 1 stops the founding: the rest are never
    /// configured, and read as failed.
    #[tokio::test]
    async fn a_pinned_box_that_fails_stops_the_founding() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = NetworkDir::new(tmp.path());
        // The reth genesis is not the manifest's: the pinned box fails in
        // build_config, before any POST.
        let mut shared = shared(&dir);
        shared.reth_genesis = Artifact::new(
            "reth-genesis.json",
            br#"{"config": {"chainId": 1}}"#.to_vec(),
        );
        let shared = Arc::new(shared);
        let mut rest = vec![node("node-1", false)];
        let results = bootstrap_founding(&node("node-2", true), &mut rest, &shared)
            .await
            .unwrap();
        assert_eq!(
            results,
            BTreeMap::from([("node-2".to_string(), Outcome::NotSent)])
        );
    }

    /// `--check` reaches the launch assertions with nothing delivered: no
    /// policy or genesis file is needed, nothing is written under the
    /// network directory, and a node the harvest does not know is skipped
    /// rather than held to a pin it never had. The nodes here are
    /// unreachable — the descriptor fixes the ports the assertions dial, so
    /// a fake server cannot stand in, and the IPv6 loopback is where a local
    /// sanvil, on `127.0.0.1:8545`, does not answer — and the check says so,
    /// per node.
    #[tokio::test]
    async fn check_asserts_the_founding_cohort_and_writes_nothing() {
        use seismic_tee_common::test_support::{NODE_KEY_1, descriptors_of, network_dir, record};

        let (_tmp, dir) = network_dir();
        std::fs::write(dir.manifest(), FIXTURE_MANIFEST).unwrap();
        write_harvest(&dir, "node-1", &record(NODE_KEY_1, "cc"));
        let descriptors = descriptors_of(&[
            ("node-1", "::1", "node-1.invalid"),
            ("node-2", "::1", "node-2.invalid"),
        ]);
        let nodes = dir.root().join("nodes.json");
        std::fs::write(&nodes, serde_json::to_vec(&descriptors).unwrap()).unwrap();

        let args = parse(&[
            "--check",
            "--manifest",
            &dir.manifest().to_string_lossy(),
            "--node",
            &nodes.to_string_lossy(),
        ]);
        let err = check(&args, None, Duration::ZERO)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("Cohort disagrees"), "{err}");
        assert!(
            err.contains("✗ node-1: unreachable via http://[::1]:8545"),
            "{err}"
        );
        assert!(!err.contains("node-2"), "{err}");
        assert!(!dir.nodes().exists(), "--check wrote under nodes/");
    }

    /// A missing manifest fails before anything else is read.
    #[tokio::test]
    async fn a_missing_manifest_fails_first() {
        let args = parse(&["--manifest", "/absent/m.json"]);
        let err = crate::configure::run(args, None)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("--manifest file not found"), "{err}");
    }

    /// --no-verify skips appraising the nodes configured, not what the run
    /// relies on: without the network's policy, nothing is sent, and the
    /// refusal names the way on — a named peer only for a join.
    #[test]
    fn no_verify_still_checks_what_the_run_relies_on() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = dir.path().join(MANIFEST_FILENAME);
        let args = parse(&[
            "dev-2",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--no-verify",
        ]);
        let manifest = Manifest::from_json_bytes(FIXTURE_MANIFEST).unwrap();

        let refusal = |founding: bool| {
            format!(
                "{:#}",
                checks_policy(
                    &args,
                    None,
                    &manifest_path,
                    &manifest,
                    founding,
                    &remedy(&args)
                )
                .unwrap_err()
            )
        };
        let error = refusal(false);
        for expected in [
            "measurement policy not found",
            "--no-verify skips appraising the nodes configured only",
            "Nothing was sent",
            "seismic-tee node configure dev-2 --bootnode <ENODE>",
        ] {
            assert!(
                error.contains(expected),
                "missing {expected:?} in:\n{error}"
            );
        }
        let error = refusal(true);
        assert!(error.contains("`seismic-tee ctx set-network`"), "{error}");
        assert!(!error.contains("--bootnode"), "{error}");

        // An appraisal already resolved is the one used.
        let appraisal = Appraisal {
            policy: b"[]".to_vec(),
            pccs_url: None,
        };
        let checks = checks_policy(
            &args,
            Some(&appraisal),
            &manifest_path,
            &manifest,
            false,
            &remedy(&args),
        )
        .unwrap();
        assert_eq!(checks.policy, b"[]");
    }

    /// The appraisal retries a transient failure and then fails the node; the
    /// verdict is the node's status line.
    #[tokio::test]
    async fn a_failed_appraisal_fails_the_node_after_retries() {
        let shared = shared(&NetworkDir::new("/nets/x"));
        let node = node("node-1", true);
        // An unparseable policy fails every attempt without reaching the
        // node; with the retry delay this would take 30s, so the test pins
        // the shape through one attempt's worth of status lines instead.
        let (tx, mut rx) = mpsc::unbounded_channel();
        let appraisal = Appraisal {
            policy: b"{ not a policy".to_vec(),
            pccs_url: None,
        };
        let verdict = tokio::time::timeout(
            Duration::from_millis(500),
            appraise(&node, &shared, &appraisal, &tx),
        )
        .await;
        assert!(verdict.is_err(), "still retrying after the first failure");
        let (name, first) = rx.recv().await.unwrap();
        assert_eq!(name, "node-1");
        assert_eq!(first, "deploy-verifying…");
        let (_, second) = rx.recv().await.unwrap();
        assert!(
            second.contains("attempt 1/3 failed, retrying in 15s"),
            "{second}"
        );
    }

    fn quote_body(node_public_key: &str, candidate: &str) -> String {
        json!({
            "node_public_key": node_public_key,
            "consensus_public_key": consensus_key("cd"),
            "candidate_tx_io_public_key": candidate,
            "evidence": azure_evidence(),
        })
        .to_string()
    }

    fn harvest_target(harvest_url: &str) -> HarvestTarget {
        HarvestTarget {
            name: "node-1".to_string(),
            harvest_url: harvest_url.to_string(),
            nonce: [0x11; 32],
        }
    }

    async fn await_window(
        harvest_url: &str,
        harvested_node_key: Option<&str>,
    ) -> anyhow::Result<()> {
        await_config_window(
            &http::client().unwrap(),
            &harvest_target(harvest_url),
            harvested_node_key,
            Duration::from_millis(300),
            Duration::from_millis(20),
        )
        .await
    }

    /// A node not up at the probe is waited for, then gated as the plan gates
    /// the rest: a founding box serving a node key other than its harvest
    /// record's, a node that took another run's POST, and an endpoint this
    /// CLI cannot read all refuse its POST.
    #[tokio::test]
    async fn a_late_node_is_waited_for_then_gated_before_its_post() {
        let candidate = candidate_tx_io_public_key("ee");
        let server = FakeServer::serve_after(
            Duration::from_millis(100),
            vec![(200, quote_body(NODE_KEY_1, &candidate))],
        );
        await_window(&server.url, Some(NODE_KEY_1)).await.unwrap();
        let server = FakeServer::serve(vec![(200, quote_body(NODE_KEY_1, &candidate))]);
        await_window(&server.url, None).await.unwrap();

        let server = FakeServer::serve(vec![(200, quote_body(NODE_KEY_2, &candidate))]);
        let err = await_window(&server.url, Some(NODE_KEY_1))
            .await
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(&format!(
                "serves node key {NODE_KEY_2}, harvested {NODE_KEY_1}"
            )),
            "{err}"
        );
        assert!(err.contains("rebooted since the harvest"), "{err}");

        for (status, expected) in [
            (410, "took a config POST this boot"),
            (404, "disagree on the harvest API"),
        ] {
            let server = FakeServer::serve(vec![(status, String::new())]);
            let err = await_window(&server.url, None)
                .await
                .unwrap_err()
                .to_string();
            assert!(err.contains(expected), "{status}: {err}");
            assert!(err.contains("nothing was sent to it"), "{status}: {err}");
        }

        let err = await_window(&refused_url(), None)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("did not answer within 0s"), "{err}");
    }

    /// The pinned box is asked afresh before a founding: one that does not
    /// answer, or that no longer serves the pinned candidate and the node key
    /// harvested from it, refuses the run before its quote is even verified.
    #[tokio::test]
    async fn a_pinned_box_that_changed_since_the_harvest_refuses_the_founding() {
        let founding = Founding {
            pin: FIXTURE_FOUNDING_TX_IO_PK.to_string(),
            pinned_box: Some("node-1".to_string()),
            founders: ["node-1".to_string()].into(),
        };
        let harvested = FoundingRecord {
            node_public_key: NODE_KEY_1.to_string(),
            consensus_public_key: consensus_key("cd"),
            candidate_tx_io_public_key: FIXTURE_FOUNDING_TX_IO_PK.to_string(),
            document: json!({}),
        };
        // Never reached: every case refuses before the verifier runs.
        let appraisal = Appraisal {
            policy: b"{ not a policy".to_vec(),
            pccs_url: None,
        };
        let args = parse(&[
            "--node",
            "nodes.json",
            "--manifest",
            "net/network-manifest.json",
        ]);
        let verify = |harvest_url: String| {
            let (founding, harvested, appraisal, remedy) =
                (&founding, &harvested, &appraisal, remedy(&args));
            async move {
                verify_pinned_box(
                    &http::client().unwrap(),
                    &harvest_target(&harvest_url),
                    founding,
                    harvested,
                    appraisal,
                    &remedy,
                )
                .await
                .unwrap_err()
                .to_string()
            }
        };

        let err = verify(refused_url()).await;
        assert!(err.contains("did not serve its quote"), "{err}");
        assert!(err.contains("Nothing was sent"), "{err}");

        for body in [
            quote_body(NODE_KEY_1, &candidate_tx_io_public_key("ee")),
            quote_body(NODE_KEY_2, FIXTURE_FOUNDING_TX_IO_PK),
        ] {
            let server = FakeServer::serve(vec![(200, body)]);
            let err = verify(server.url.clone()).await;
            assert!(err.contains("the founding root_key is gone"), "{err}");
            assert!(
                err.contains("seismic-tee node harvest net --node nodes.json --force"),
                "{err}"
            );
        }
    }

    /// configure reads the harvest strictly: none is a join's table, but
    /// records that do not parse, or that do not hold the manifest's pin,
    /// refuse the run.
    #[test]
    fn founding_records_must_hold_the_manifests_pin() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = NetworkDir::new(tmp.path());
        let manifest = Manifest::from_json_bytes(FIXTURE_MANIFEST).unwrap();
        assert!(founding_records(&dir, &manifest).unwrap().is_none());

        let record = |candidate: &str| {
            json!({
                "harvest_nonce": "11".repeat(32),
                "node_public_key": NODE_KEY_1,
                "consensus_public_key": consensus_key("cd"),
                "candidate_tx_io_public_key": candidate,
                "evidence": {},
            })
        };
        write_harvest(&dir, "node-1", &record(&candidate_tx_io_public_key("ee")));
        let err = founding_records(&dir, &manifest).unwrap_err().to_string();
        assert!(err.contains("not assembled from the harvest"), "{err}");
        assert!(err.contains("network assemble"), "{err}");

        write_harvest(&dir, "node-1", &record(FIXTURE_FOUNDING_TX_IO_PK));
        let records = founding_records(&dir, &manifest).unwrap().unwrap();
        assert_eq!(records["node-1"].node_public_key, NODE_KEY_1);

        std::fs::write(dir.harvest_record("node-2"), "{ not a record").unwrap();
        assert!(founding_records(&dir, &manifest).is_err());
    }

    /// A node that was sent nothing is re-run with the flags that shaped this
    /// run: its named peers and its appraisal.
    #[test]
    fn a_rerun_repeats_the_runs_peers_and_appraisal() {
        let peer = format!("enode://{}@198.51.100.1:30303", "ab".repeat(64));
        let args = parse(&[
            "beta",
            "gamma",
            "--context",
            "devnet-1",
            "--bootnode",
            &peer,
            "--pccs-url",
            "http://pccs",
        ]);
        assert_eq!(
            rerun(&args, &remedy(&args), &["gamma"]),
            format!(
                "seismic-tee node configure --context devnet-1 gamma --bootnode {peer} \
                 --pccs-url http://pccs"
            )
        );
        let args = parse(&["--node", "nodes.json", "--no-verify"]);
        assert_eq!(
            rerun(&args, &remedy(&args), &["beta"]),
            "seismic-tee node configure --node nodes.json beta --no-verify"
        );
    }

    /// After Ctrl-C: watch, appraise, re-assert a founding's launch, re-run
    /// for the rest — with this run's flags.
    #[test]
    fn an_interrupted_run_names_what_to_run_next() {
        let args = parse(&["beta", "--context", "devnet-1", "--pccs-url", "http://pccs"]);
        assert_eq!(
            interrupted_steps(&args, &remedy(&args), true),
            [
                "seismic-tee node status --watch --context devnet-1",
                "seismic-tee node verify --context devnet-1 --name <node> --pccs-url http://pccs",
                "seismic-tee node configure --check --context devnet-1",
                "seismic-tee node configure --context devnet-1 beta --pccs-url http://pccs",
            ]
        );
        assert!(
            !interrupted_steps(&args, &remedy(&args), false)
                .iter()
                .any(|step| step.contains("--check"))
        );
    }
}
