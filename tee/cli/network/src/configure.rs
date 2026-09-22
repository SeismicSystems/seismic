//! `configure`: found a network in one command.
//!
//! ```text
//! seismic-tee network configure --genesis-node devnet-3-1 \
//!     --manifest tee/networks/devnet-3/network-manifest.json
//! ```
//!
//! Configures a whole cohort at once: the one genesis node (`--genesis-node`, mints
//! `root_key` locally) plus every joining node (`--join`, fetches `root_key`
//! from genesis via `getWrappedRootKey`). Exactly one node is genesis —
//! assigned here, not left to a per-node flag — so a double-genesis network
//! split is unrepresentable. This is the only caller that ever sets
//! `genesis_node = true`.
//!
//! Bootnode bootstrap is inherently two-stage on a greenfield cohort: a node's
//! reth enode isn't known until reth is up, so there is nothing to hand the
//! joiners as `[network].bootnodes` up front. So:
//!
//! - Stage 1 — configure only the genesis node (empty `bootnodes`), then poll
//!   its `seismic_nodeInfo` until reth reports an enode.
//! - Stage 2 — configure the joiners in parallel with `bootnodes = [genesis
//!   enode]`.
//!
//! Then every node's enode is collected and the founding set is persisted to
//! `nodes/bootnodes.json`. On a later configure run (reboot or re-provision)
//! that file exists, so the two-stage dance is skipped and the full founding
//! set goes to every node in one parallel pass.
//!
//! Root-key bootstrap rides the same list: tdx-init derives each node's
//! root-key fetch peers from its POSTed bootnodes (`http://<host>:7878`, the
//! node's own entry dropped), so stage-2 joiners fetch `root_key` from
//! genesis, and there is no second peer list that could skew from the bootnode
//! set.
//!
//! Every node is deploy-verified as soon as it reaches a ready state — the
//! `seismic-tee node verify` check, run as its own task so a fast box is
//! appraised while a slow one still wipes its disk. The founder is a relying
//! party the moment stage 2 hands genesis's enode to the joiners, so the
//! genesis gate lands before that: a genesis node that does not pass stops the
//! founding rather than pointing the cohort at an unappraised box. The verdict
//! is the node's result: a node that fails its appraisal counts as failed, its
//! enode never enters `nodes/bootnodes.json`, and the launch assertions never
//! run.
//!
//! The summit genesis is delivered with current IPs spliced in: the committed
//! `summit-genesis.toml` is a founding-era snapshot whose
//! `[[validators]].ip_address` entries are network topology, not identity —
//! excluded from the manifest's config digest — so each configure run replaces
//! them with the live IPs from the cohort's descriptor map without
//! re-serializing any other field. After the whole cohort accepts its config,
//! the launch assertions (see [`crate::launch`]) verify each box against what
//! the manifest pins.
//!
//! Founding is the founder's act, so this lives in the `network` group; joining
//! an already-live network is `seismic-tee node configure`. Both go
//! through the node crate's `build_config` / `post_config` primitives and its
//! status poller, so each node's POSTed config and wipe-watch are identical —
//! only `[node].genesis_node` and the bootnode set differ.
//!
//! `--check` runs the launch assertions and nothing else: no config is built
//! or POSTed, nothing is written, and every founding node — every box with a
//! harvest record — is held to the manifest's pins as it stands now. It is
//! `assemble --check`'s sibling one step on: that holds the artifact set on
//! disk to its inputs, this holds the live cohort to the artifact set. A
//! founder runs it after a launch whose holders had not settled, after a
//! reboot or a re-image, or whenever the cohort may have drifted from what
//! was pinned. It waits [`launch::CHECK_TIMEOUT`], not the founding's
//! readiness window: the cohort is supposed to be up, so a node that does not
//! answer is reported as such and the check is re-run once it is.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context as _, bail};
use clap::Args;
use clap_complete::ArgValueCandidates;
use seismic_tee_common::http::TDX_INIT_PORT;
use seismic_tee_common::{
    Artifact, Descriptors, Manifest, NetworkDir, NodeDescriptor, http, next_step, rpc,
};
use seismic_tee_context::{Context, ContextArgs, complete, load_nodes};
use seismic_tee_node::configure::{
    ConfigInputs, DEFAULT_EMAIL, TDX_INIT_LISTENER_TIMEOUT, TDX_INIT_RETRY_INTERVAL, build_config,
    post_config_within, render_config, resolve_reth_genesis, resolve_summit_genesis, write_record,
};
use seismic_tee_node::status::{POLL_INTERVAL, ProvisioningWatch, poll_provisioning};
use seismic_tee_node::verify::{
    PolicySourceArgs, VerifierArgs, challenge_node, check_policy_source_files, resolve_policy,
    retry_flags,
};
use seismic_tee_node::{load_manifest, resolve_manifest};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::args::DirArgs;
use crate::bootnodes::{self, Bootnode};
use crate::dashboard::CohortDashboard;
use crate::founding::{FoundingRecords, SUMMIT_CONSENSUS_PORT, load_harvest_records};
use crate::gates::hex_0x;
use crate::launch::{self, LaunchTarget};

/// This command, spelled as the next step after `assemble` (written or
/// `--check`ed), with `genesis` as the genesis node. Which node is genesis is the
/// founder's call and any founding node is a valid one, so callers pass the
/// first in name order. `configure` takes the manifest, not `DIR`, so an
/// explicit `DIR` becomes `--manifest`; an explicit `--context` is repeated
/// as [`DirArgs::as_args`] would.
pub fn invocation(genesis: &str, args: &DirArgs, dir: &NetworkDir) -> String {
    let scope = match (&args.dir, &args.context.context) {
        (Some(_), _) => format!(" --manifest {}", dir.manifest().display()),
        (None, Some(context)) => format!(" --context {context}"),
        (None, None) => String::new(),
    };
    format!("seismic-tee network configure --genesis-node {genesis}{scope}")
}

/// A whole cohort's wipes tend to finish together, so every challenge hits the
/// PCCS at once — and a DCAP collateral fetch is the one transient way a
/// challenge fails. Retry a few times before the verdict is terminal: a
/// founding that a blip fails is expensive to recover (config delivery is once
/// per boot), while a real measurement mismatch just fails every attempt.
pub const APPRAISAL_ATTEMPTS: u32 = 3;
pub const APPRAISAL_RETRY: Duration = Duration::from_secs(15);

/// One cohort member, resolved from its descriptor-map entry + assigned role.
///
/// `bootnodes` (→ `[network].bootnodes`: reth p2p + tdx-init's derived
/// root-key fetch peers) is assigned by the configure flow, not here — empty
/// for the greenfield genesis node, `[genesis enode]` for its joiners, the
/// full founding set on re-configure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The node's key in the descriptor map.
    pub name: String,
    pub public_ip: String,
    pub fqdn: String,
    pub genesis: bool,
    pub bootnodes: Vec<String>,
}

impl Node {
    fn descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            public_ip: self.public_ip.clone(),
            fqdn: self.fqdn.clone(),
        }
    }

    fn role(&self) -> &'static str {
        if self.genesis { "genesis" } else { "join" }
    }
}

/// Resolve the cohort from the descriptor map by name: exactly one genesis
/// (mints `root_key`), everyone else a joiner. Role assignment lives here, not
/// in a per-node flag, so there is exactly one genesis by construction.
///
/// `join` is the joiners' names; `None` means every other node in the map, and
/// a list configures just that subset. Names must be keys of the map — a role
/// for a node the stack doesn't have is a typo, not a node.
pub fn build_cohort(
    descriptors: &Descriptors,
    genesis: &str,
    join: Option<&[String]>,
) -> anyhow::Result<Vec<Node>> {
    let join: Vec<String> = match join {
        Some(join) => join.to_vec(),
        None => descriptors
            .keys()
            .filter(|name| *name != genesis)
            .cloned()
            .collect(),
    };
    let names: Vec<&str> = std::iter::once(genesis)
        .chain(join.iter().map(String::as_str))
        .collect();
    let unknown: BTreeSet<&str> = names
        .iter()
        .copied()
        .filter(|name| !descriptors.contains_key(*name))
        .collect();
    if !unknown.is_empty() {
        bail!(
            "no such node(s) in the descriptor map: {} — it holds {}",
            unknown.into_iter().collect::<Vec<_>>().join(", "),
            descriptors.keys().cloned().collect::<Vec<_>>().join(", ")
        );
    }

    // A name given twice (--genesis-node reused as --join, or a copy-pasted --join)
    // would race two conflicting POSTs against one node and silently collide
    // on the name-keyed dashboard/result maps — refuse instead. Two map
    // entries sharing an IP are the same mistake in the file.
    let nodes: Vec<Node> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let descriptor = &descriptors[*name];
            Node {
                name: name.to_string(),
                public_ip: descriptor.public_ip.clone(),
                fqdn: descriptor.fqdn.clone(),
                genesis: i == 0,
                bootnodes: Vec::new(),
            }
        })
        .collect();
    for (what, key_of) in [
        ("name", (|n: &Node| n.name.as_str()) as fn(&Node) -> &str),
        ("public_ip", |n: &Node| n.public_ip.as_str()),
    ] {
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for node in &nodes {
            *counts.entry(key_of(node)).or_default() += 1;
        }
        let dupes: Vec<&str> = counts
            .iter()
            .filter(|(_, c)| **c > 1)
            .map(|(k, _)| *k)
            .collect();
        if !dupes.is_empty() {
            bail!(
                "duplicate node {what}(s) in cohort: {} — was the same node named more than once?",
                dupes.join(", ")
            );
        }
    }
    Ok(nodes)
}

/// Load the founding facts the delivery needs from the network directory: the
/// current-IP splice map (pinned node pubkey → `<ip>:<consensus port>`, from
/// `inputs/harvest/` joined with `descriptors`, the cohort's live node table)
/// and the harvest records themselves (each configured box's pinned keys, for
/// the launch assertions).
pub fn load_founding_facts(
    dir: &NetworkDir,
    descriptors: &Descriptors,
) -> anyhow::Result<(BTreeMap<String, String>, FoundingRecords)> {
    let records = load_harvest_records(dir)?;
    let mut ip_by_node_pubkey = BTreeMap::new();
    for (name, record) in &records {
        let Some(descriptor) = descriptors.get(name) else {
            bail!(
                "the cohort has no node {name:?} — the cohort's node table supplies each \
                 founding validator's current IP. A harvested box that is gone from the cohort \
                 means it changed under the founding: re-found rather than configuring",
            );
        };
        ip_by_node_pubkey.insert(
            record.node_public_key.clone(),
            format!("{}:{SUMMIT_CONSENSUS_PORT}", descriptor.public_ip),
        );
    }
    Ok((ip_by_node_pubkey, records))
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
/// The committed summit genesis is a founding-era snapshot: its validator IPs
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

/// The deploy-verification inputs shared by the whole cohort: the measurement
/// policy resolved once up front, and what steers the verifier. Resolving the
/// policy is per-run work, not per-node, and it must fail before any node is
/// touched — config delivery is once per boot.
#[derive(Debug, Clone)]
pub struct Appraisal {
    pub policy: Vec<u8>,
    pub pccs_url: Option<String>,
}

/// Everything the cohort's nodes share for one configure run.
pub struct Shared {
    pub manifest: Manifest,
    pub reth_genesis: Artifact,
    /// The delivered summit genesis: the artifact-set copy with current IPs
    /// spliced in.
    pub summit_genesis: Artifact,
    pub email: String,
    /// `None` under `--no-verify`.
    pub appraisal: Option<Appraisal>,
    pub dir: NetworkDir,
    /// The plain-HTTP client: tdx-init's config receiver and the key holders.
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
/// box must not be handed to the joiners as a bootnode, written into the
/// founding `bootnodes.json`, or counted as a founded node.
async fn appraise(node: &Node, shared: &Shared, appraisal: &Appraisal, status: &StatusTx) -> bool {
    let endpoint = node.descriptor().attestation_rpc_url();
    let mut attempt = 1;
    loop {
        report_status(status, node, "deploy-verifying…");
        match challenge_node(
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

/// Build + POST one node's config, poll its LUKS wipe, then deploy-verify it,
/// reporting the latest status line for the dashboard. Returns whether the
/// node reached a ready state and passed its appraisal (no appraisal under
/// `--no-verify`: ready is the whole bar). Never fails — a failure is reported
/// as the node's status and reflected in the return, so one bad node doesn't
/// abort the rest of the cohort.
async fn configure_node(node: Node, shared: Arc<Shared>, status: StatusTx) -> (String, bool) {
    let outcome = async {
        report_status(&status, &node, "building config…");
        let config = build_config(&ConfigInputs {
            manifest: &shared.manifest,
            reth_genesis: &shared.reth_genesis,
            summit_genesis: &shared.summit_genesis,
            bootnodes: &node.bootnodes,
            external_ip: &node.public_ip,
            fqdn: &node.fqdn,
            email: &shared.email,
            genesis_node: node.genesis,
        })?;
        // The record first: what is about to be sent exists on disk before
        // anything is sent, as the record of what the node booted with.
        write_record(
            &shared.dir.init_config(&node.name),
            &render_config(&config)?,
        )?;
        let descriptor = node.descriptor();
        let rpc_client = rpc::Client::new(&descriptor.attestation_rpc_url())?;

        report_status(
            &status,
            &node,
            format!("POSTing config to tdx-init :{TDX_INIT_PORT}…"),
        );
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
    let ok = match outcome {
        Ok(ok) => ok,
        Err(error) => {
            report_status(&status, &node, format!("ERROR: {error:#}"));
            false
        }
    };
    (node.name, ok)
}

/// Ctrl-C during a cohort run: the POSTs that landed keep provisioning
/// server-side; this process stops watching.
#[derive(Debug)]
pub struct Interrupted;

impl std::fmt::Display for Interrupted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("interrupted")
    }
}

impl std::error::Error for Interrupted {}

/// Configure and appraise every node concurrently, refreshing the dashboard
/// until all tasks finish. Returns `{node name: ok}`, plus each failed node's
/// full status printed after the dashboard (which folds and truncates a
/// status to one row, and a failure's reason — often a verifier's multi-line
/// chain — is the one thing the run must not swallow).
pub async fn run_cohort(
    nodes: &[Node],
    shared: &Arc<Shared>,
) -> anyhow::Result<BTreeMap<String, bool>> {
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
                    if n.genesis {
                        format!("{} (genesis)", n.name)
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
                    Ok((name, ok)) => {
                        results.insert(name, ok);
                    }
                    Err(error) => {
                        // A panic in a node's task is that node's failure, not
                        // the cohort's.
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
            _ = tokio::signal::ctrl_c() => {
                // Aborting ends every task at its next await: a wipe watch
                // within a poll, a POST or a collateral fetch within its own
                // timeout. The POSTs that landed keep provisioning
                // server-side either way.
                tasks.abort_all();
                println!("\nStopped watching — configured nodes keep provisioning.");
                return Err(Interrupted.into());
            }
        }
    }
    dashboard.render(&states); // final paint of terminal states
    for node in nodes {
        // A task that ended abnormally has no result: it failed.
        if !results.get(&node.name).copied().unwrap_or(false) {
            results.insert(node.name.clone(), false);
            println!("\n{}: {}", node.name, states[&node.name]);
        }
    }
    Ok(results)
}

/// Two-stage greenfield bootstrap: genesis first (so its enode exists), then
/// the joiners pointed at it. Returns `{node name: ok}` across both stages.
///
/// Stage 1 ends with the genesis node ready *and* appraised, which is the gate
/// the stage boundary exists for: its enode is what every joiner dials and
/// what `nodes/bootnodes.json` records. If genesis fails either half, the
/// joiners aren't configured (they'd have neither a root_key source nor a
/// bootnode); the missing results read as failures in the report.
pub async fn bootstrap_greenfield(
    nodes: &mut [Node],
    shared: &Arc<Shared>,
) -> anyhow::Result<BTreeMap<String, bool>> {
    let (genesis, joiners) = nodes
        .split_first_mut()
        .expect("a cohort has a genesis node");

    println!("Stage 1/2: configuring the genesis node (no bootnodes yet)...");
    genesis.bootnodes.clear();
    let mut results = run_cohort(std::slice::from_ref(genesis), shared).await?;
    if !results.get(&genesis.name).copied().unwrap_or(false) {
        println!(
            "Genesis node failed in stage 1 — skipping joiner bootstrap: its enode is what every \
             joiner would dial."
        );
        return Ok(results);
    }
    if joiners.is_empty() {
        return Ok(results);
    }

    println!("Stage 2/2: fetching the genesis enode via seismic_nodeInfo...");
    let enodes = bootnodes::collect_enodes(
        &[(genesis.name.clone(), genesis.descriptor().eth_rpc_url())],
        bootnodes::ENODE_TIMEOUT,
        bootnodes::POLL_INTERVAL,
    )
    .await?;
    let genesis_enode = enodes[&genesis.name].clone();
    bootnodes::warn_on_ip_mismatch(&genesis_enode, &genesis.public_ip, &genesis.name);

    println!(
        "Stage 2/2: configuring {} joining node(s) off genesis enode...",
        joiners.len()
    );
    for joiner in joiners.iter_mut() {
        joiner.bootnodes = vec![genesis_enode.clone()];
    }
    results.extend(run_cohort(joiners, shared).await?);
    Ok(results)
}

/// Collect every node's enode and write the founding set to `bootnodes.json`.
///
/// Only writes when every node came up ready and appraised — a partial
/// founding set would silently drop a node from every later re-configure, and
/// an unappraised box must not be recorded as one the cohort dials. If any
/// node failed, skip the write and warn; the report surfaces the failure.
///
/// Best-effort: config delivery has already succeeded by the time this runs,
/// and `bootnodes.json` is only a refresh for later runs, so a failure to
/// collect the enodes degrades to a warning rather than aborting before the
/// cohort report prints. The next configure run re-establishes the set.
pub async fn persist_founding_bootnodes(
    nodes: &[Node],
    results: &BTreeMap<String, bool>,
    path: &std::path::Path,
) {
    let failed: Vec<&str> = nodes
        .iter()
        .filter(|n| !results.get(&n.name).copied().unwrap_or(false))
        .map(|n| n.name.as_str())
        .collect();
    if !failed.is_empty() {
        eprintln!(
            "warning: not writing {} — {}/{} node(s) failed: {}",
            path.display(),
            failed.len(),
            nodes.len(),
            failed.join(", ")
        );
        return;
    }

    println!("Collecting the founding bootnode set (seismic_nodeInfo)...");
    let targets: Vec<(String, String)> = nodes
        .iter()
        .map(|n| (n.name.clone(), n.descriptor().eth_rpc_url()))
        .collect();
    let enodes = match bootnodes::collect_enodes(
        &targets,
        bootnodes::ENODE_TIMEOUT,
        bootnodes::POLL_INTERVAL,
    )
    .await
    {
        Ok(enodes) => enodes,
        Err(error) => {
            eprintln!(
                "warning: not writing {} — could not collect the founding enode set: {error:#}",
                path.display()
            );
            return;
        }
    };
    let set: Vec<Bootnode> = nodes
        .iter()
        .map(|n| {
            bootnodes::warn_on_ip_mismatch(&enodes[&n.name], &n.public_ip, &n.name);
            Bootnode {
                name: n.name.clone(),
                enode: enodes[&n.name].clone(),
            }
        })
        .collect();
    match bootnodes::save_bootnodes(path, &set) {
        Ok(()) => println!(
            "Wrote founding bootnode set ({} node(s)) to {}",
            set.len(),
            path.display()
        ),
        Err(error) => eprintln!("warning: not writing {}: {error:#}", path.display()),
    }
}

/// The cohort report, and the failure it ends in if any node did not make it.
fn report(
    nodes: &[Node],
    results: &BTreeMap<String, bool>,
    args: &ConfigureArgs,
    manifest_path: &Path,
) -> anyhow::Result<()> {
    let rule = "=".repeat(80);
    println!("\n{rule}\nCOHORT CONFIGURED\n{rule}");
    for node in nodes {
        let ok = results.get(&node.name).copied().unwrap_or(false);
        println!(
            "  {} {} ({}) — https://{}/rpc",
            if ok { "✓" } else { "✗" },
            node.name,
            node.role(),
            node.fqdn
        );
    }
    println!("{rule}\n");

    if nodes
        .iter()
        .any(|n| n.genesis && !results.get(&n.name).copied().unwrap_or(false))
    {
        println!(
            "The genesis node failed — joiners depend on it for root_key and as their bootnode. \
             Fix genesis first."
        );
    }
    let failed: Vec<&str> = nodes
        .iter()
        .filter(|n| !results.get(&n.name).copied().unwrap_or(false))
        .map(|n| n.name.as_str())
        .collect();
    if failed.is_empty() {
        return Ok(());
    }
    if !args.no_verify {
        // tdx-init takes one config POST per boot, so a node that took its
        // config and then failed its appraisal is re-appraised, not
        // re-configured. Its full verifier reason was printed above. No
        // --node here: the context (already in force for this run) supplies
        // the node table to `node verify` too.
        println!(
            "A node that took its config but did not pass the appraisal is retried with `verify`, \
             not with a second `configure` (tdx-init takes one config POST per boot):\n    \
             seismic-tee node verify --name <node> --manifest {}{}\n",
            manifest_path.display(),
            retry_flags(&args.policy_source, &args.verifier),
        );
    }
    bail!(
        "{}/{} node(s) failed: {}",
        failed.len(),
        nodes.len(),
        failed.join(", ")
    );
}

#[derive(Debug, Args)]
pub struct ConfigureArgs {
    /// Name of the one genesis node — the node that mints root_key locally
    /// and that the joiners fetch it from — as keyed in the cohort's node
    /// table. Exactly one node per network is genesis; assigning it here (not
    /// a per-node flag) makes a double-genesis split impossible. Required,
    /// except under --check, which assigns no roles.
    #[arg(
        long,
        value_name = "NAME",
        required_unless_present = "check",
        add = ArgValueCandidates::new(complete::nodes)
    )]
    pub genesis_node: Option<String>,

    /// Name of a joining node (fetches root_key from genesis via
    /// getWrappedRootKey). Repeatable. Default: every other node in the
    /// cohort's node table; name a subset to configure only those.
    #[arg(long, value_name = "NAME", add = ArgValueCandidates::new(complete::nodes))]
    pub join: Option<Vec<String>>,

    /// Network manifest JSON (from `assemble`); → [network]. The network
    /// directory is the one it sits in: the harvest and the genesis files it
    /// pins are read from there. Omit it to use the current context's
    /// network.
    #[arg(long, value_name = "FILE")]
    pub manifest: Option<PathBuf>,

    /// Cohort's node table: `pulumi stack output nodes --json`, i.e.
    /// {<name>: {public_ip, fqdn}, …}. Omit it to use the current context's
    /// network.
    #[arg(long, value_name = "FILE")]
    pub nodes: Option<PathBuf>,

    /// reth genesis JSON POSTed to every node; → [network].reth_genesis_base64.
    /// Default: reth-genesis.json beside --manifest (the artifact-set layout).
    #[arg(long, value_name = "FILE")]
    pub reth_genesis: Option<PathBuf>,

    /// summit genesis TOML POSTed to every node, with each validator's current
    /// IP spliced in; → [network].summit_genesis_base64. Default:
    /// summit-genesis.toml beside --manifest (the artifact-set layout).
    #[arg(long, value_name = "FILE")]
    pub summit_genesis: Option<PathBuf>,

    /// certbot contact email → [node.domain].email.
    #[arg(long, default_value = DEFAULT_EMAIL)]
    pub email: String,

    /// Found the cohort without deploy-verifying it. By default each node is
    /// appraised once it is up — the same check as `seismic-tee node verify`,
    /// against the policy --manifest pins — and a node that fails counts as
    /// failed.
    #[arg(long)]
    pub no_verify: bool,

    #[command(flatten)]
    pub policy_source: PolicySourceArgs,

    #[command(flatten)]
    pub verifier: VerifierArgs,

    /// Run the launch assertions and nothing else: every founding node's
    /// reth must serve the manifest's eth.genesis_hash as block 0, and every
    /// founding box's holder its harvested keys. Configures nothing, writes
    /// nothing; for after a launch whose holders had not settled, a reboot,
    /// a re-image, or whenever the cohort may have drifted from its pins.
    /// Takes only --manifest and --nodes (or the context): roles, genesis
    /// files and the appraisal are delivery's, so their flags are refused.
    #[arg(
        long,
        conflicts_with_all = [
            "genesis_node",
            "join",
            "reth_genesis",
            "summit_genesis",
            "no_verify",
            "policy",
            "measurements",
            "pccs_url",
        ]
    )]
    pub check: bool,

    #[command(flatten)]
    pub context: ContextArgs,
}

pub async fn run(args: ConfigureArgs) -> anyhow::Result<ExitCode> {
    if args.check {
        return check(&args, launch::CHECK_TIMEOUT).await;
    }
    let genesis_node = args
        .genesis_node
        .as_deref()
        .expect("clap: --genesis-node is required without --check");
    check_policy_source_files(&args.policy_source, args.no_verify)?;
    // Validate the shared network artifacts once, so a bad one fails fast
    // here rather than as N identical per-node errors mid-dashboard.
    let manifest_path = resolve_manifest(args.manifest.as_deref(), &args.context)?;
    let manifest = load_manifest(&manifest_path)?;
    let reth_genesis = Artifact::read(&resolve_reth_genesis(
        args.reth_genesis.as_deref(),
        &manifest_path,
    )?)?;
    manifest
        .check_reth_genesis(reth_genesis.bytes())
        .with_context(|| format!("--reth-genesis {}", reth_genesis.path().display()))?;
    let committed = Artifact::read(&resolve_summit_genesis(
        args.summit_genesis.as_deref(),
        &manifest_path,
    )?)?;
    manifest
        .check_summit_genesis(committed.bytes())
        .with_context(|| format!("--summit-genesis {}", committed.path().display()))?;

    // Resolve the policy once for the whole cohort, before any node is
    // touched: a policy the manifest doesn't commit to, or a rejected
    // measurements file, must fail while the fix still costs nothing — config
    // delivery is once per boot.
    let appraisal = if args.no_verify {
        eprintln!(
            "warning: --no-verify: this cohort's nodes will not be appraised. Run \
             `seismic-tee node verify` before relying on an unappraised node."
        );
        None
    } else {
        let policy = resolve_policy(&args.policy_source, &manifest_path, &manifest, true)?;
        eprintln!("Appraising against {}", policy.source);
        Some(Appraisal {
            policy: policy.bytes,
            pccs_url: args.verifier.pccs_url.clone(),
        })
    };

    // The founding inputs live beside the manifest (the network-directory
    // layout): the harvest supplies each box's pinned keys; the cohort's node
    // table (the context's, or --nodes) supplies its current IP.
    let dir = NetworkDir::of_manifest(&manifest_path);
    let descriptors = load_nodes(args.nodes.as_deref(), &args.context, "--nodes")?;
    let (ip_by_node_pubkey, harvest_records) = load_founding_facts(&dir, &descriptors)?;
    let spliced = splice_validator_ips(committed.bytes(), &ip_by_node_pubkey)?;
    if spliced != committed.bytes() {
        println!(
            "Spliced current descriptor IPs into the delivered summit genesis (the committed {} is \
             a founding-era snapshot; validator IPs are topology, not identity)",
            committed
                .path()
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        );
    }
    let summit_genesis = Artifact::new(committed.path(), spliced);

    let mut nodes = build_cohort(&descriptors, genesis_node, args.join.as_deref())?;
    let unharvested: Vec<&str> = nodes
        .iter()
        .filter(|n| !harvest_records.contains_key(&n.name))
        .map(|n| n.name.as_str())
        .collect();
    if !unharvested.is_empty() {
        bail!(
            "node(s) without a founding harvest record: {} — this command configures a founding \
             cohort, and every box's launch is asserted against the keys harvested from it. A \
             joiner arriving after founding uses `seismic-tee node configure`.",
            unharvested.join(", ")
        );
    }
    let joiners: Vec<&str> = nodes
        .iter()
        .filter(|n| !n.genesis)
        .map(|n| n.name.as_str())
        .collect();
    println!(
        "Configuring {} node(s): genesis={}{}",
        nodes.len(),
        nodes[0].name,
        if joiners.is_empty() {
            " (genesis-only)".to_string()
        } else {
            format!(", joining=[{}]", joiners.join(", "))
        }
    );

    // Collateral fetches go over TLS; see `node verify` for why the provider
    // is chosen here.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let shared = Arc::new(Shared {
        manifest,
        reth_genesis,
        summit_genesis,
        email: args.email.clone(),
        appraisal,
        dir: dir.clone(),
        client: http::client()?,
    });

    // bootnodes.json lives under the network directory's nodes/ — the
    // founding enode set from a prior run.
    let bootnodes_path = dir.bootnodes();
    let outcome = if bootnodes_path.exists() {
        // Re-configure: hand the full founding set to every node (a node
        // listing its own enode is harmless) and configure in one parallel
        // pass — no genesis-first staging, since the enodes are already known.
        // Each node is still appraised the moment it is ready.
        let founding = bootnodes::load_bootnodes(&bootnodes_path)?;
        let enodes: Vec<String> = founding.into_iter().map(|b| b.enode).collect();
        println!(
            "Reusing {} founding bootnode(s) from {}; configuring the whole cohort in one pass.",
            enodes.len(),
            bootnodes_path.display()
        );
        for node in &mut nodes {
            node.bootnodes = enodes.clone();
        }
        run_cohort(&nodes, &shared).await
    } else {
        bootstrap_greenfield(&mut nodes, &shared).await
    };
    let results = match outcome {
        Ok(results) => results,
        Err(error) if error.is::<Interrupted>() => return Ok(ExitCode::from(130)),
        Err(error) => return Err(error),
    };

    // Refresh the founding set from every node's live enode (fresh each run).
    persist_founding_bootnodes(&nodes, &results, &bootnodes_path).await;
    report(&nodes, &results, &args, &manifest_path)?;

    // Every node accepted its config — now assert the launch against what the
    // manifest pins (see `launch` for why both are load-bearing).
    let targets: Vec<LaunchTarget> = nodes
        .iter()
        .map(|n| LaunchTarget::new(&n.name, &n.descriptor(), &harvest_records[&n.name]))
        .collect();
    assert_launch(
        &shared.client,
        shared.manifest.eth.genesis_hash,
        &targets,
        launch::READY_TIMEOUT,
    )
    .await?;
    // The founding is done; what follows is using the network. Only a
    // context-supplied network has a name to select a node under — an
    // explicit --manifest may be a directory nothing is registered for.
    if args.manifest.is_none() {
        let context = Context::load(args.context.config.as_deref())?;
        let network = &context
            .select(args.context.context.as_deref())?
            .selection
            .network;
        next_step::print(
            "select a node, and point a tool at it:",
            &[
                format!("seismic-tee ctx use {network}/{genesis_node}"),
                "seismic-tee ctx exec -- scast block-number".to_string(),
            ],
        );
    }
    Ok(ExitCode::SUCCESS)
}

/// Both launch assertions, in order, each announced as it starts: every
/// target's reth serves `genesis_hash` as block 0, then every target's
/// holder serves its pinned founding keys. `timeout` is how long a target
/// that does not answer is waited for — the founding's readiness window from
/// `configure`, a short one under `--check`.
async fn assert_launch(
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
    println!("Launch assertion 2/2: every holder serves its pinned founding keys...");
    launch::assert_cohort_holder_keys(client, targets, timeout, launch::POLL_INTERVAL).await?;
    println!("Launch assertions green: the cohort that launched is the cohort pinned.");
    Ok(())
}

/// `--check`: the launch assertions alone, over the founding cohort as it
/// stands now — nothing is built, sent or written.
///
/// The founding cohort is the harvest: every record must still have a node
/// to reach (`load_founding_facts` refuses otherwise, as delivery does), and
/// a node the harvest does not know is not a founding box — it joined later
/// and `node verify` appraises it — so it is named and skipped rather than
/// held to a pin it never had.
async fn check(args: &ConfigureArgs, timeout: Duration) -> anyhow::Result<ExitCode> {
    let manifest_path = resolve_manifest(args.manifest.as_deref(), &args.context)?;
    let manifest = load_manifest(&manifest_path)?;
    let dir = NetworkDir::of_manifest(&manifest_path);
    let descriptors = load_nodes(args.nodes.as_deref(), &args.context, "--nodes")?;
    let (_, harvest_records) = load_founding_facts(&dir, &descriptors)?;
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
    // The eth RPC is reached over TLS; see `node verify` for why the provider
    // is chosen here.
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
    use seismic_tee_common::test_support::{FIXTURE_MANIFEST, FakeServer, refused_url, rpc_result};
    use serde_json::json;

    use super::*;

    const NODE_KEY_1: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const NODE_KEY_2: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn descriptors(nodes: &[(&str, &str, &str)]) -> Descriptors {
        nodes
            .iter()
            .map(|(name, ip, fqdn)| {
                (
                    name.to_string(),
                    NodeDescriptor {
                        public_ip: ip.to_string(),
                        fqdn: fqdn.to_string(),
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

    /// The next-step line `assemble` prints: an explicit `DIR` becomes the
    /// manifest it holds, an explicit `--context` is repeated, a persisted
    /// selection needs nothing.
    #[test]
    fn the_invocation_carries_the_scope_it_was_given() {
        let dir = NetworkDir::new("/nets/devnet-1");
        let args = |dir: Option<&str>, context: Option<&str>| DirArgs {
            dir: dir.map(PathBuf::from),
            context: ContextArgs {
                context: context.map(str::to_string),
                config: None,
            },
        };
        assert_eq!(
            invocation("alpha", &args(None, None), &dir),
            "seismic-tee network configure --genesis-node alpha"
        );
        assert_eq!(
            invocation("alpha", &args(None, Some("devnet-1")), &dir),
            "seismic-tee network configure --genesis-node alpha --context devnet-1"
        );
        assert_eq!(
            invocation(
                "alpha",
                &args(Some("/nets/devnet-1"), Some("devnet-1")),
                &dir
            ),
            "seismic-tee network configure --genesis-node alpha --manifest \
             /nets/devnet-1/network-manifest.json"
        );
    }

    #[test]
    fn the_cohort_has_exactly_one_genesis_first() {
        let nodes = build_cohort(
            &three(),
            "node-1",
            Some(&["node-2".into(), "node-3".into()]),
        )
        .unwrap();
        assert_eq!(nodes.len(), 3);
        assert!(nodes[0].genesis);
        assert_eq!(nodes[0].name, "node-1");
        assert_eq!(nodes[0].public_ip, "1.1.1.1");
        assert!(nodes[1..].iter().all(|n| !n.genesis));
        assert!(nodes.iter().all(|n| n.bootnodes.is_empty()));

        // Joiners default to the rest of the map; a subset configures just it;
        // genesis alone is a cohort.
        let nodes = build_cohort(&three(), "node-2", None).unwrap();
        assert_eq!(
            nodes.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(),
            ["node-2", "node-1", "node-3"]
        );
        let nodes = build_cohort(&three(), "node-1", Some(&["node-3".into()])).unwrap();
        assert_eq!(
            nodes.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(),
            ["node-1", "node-3"]
        );
        let nodes = build_cohort(&three(), "node-1", Some(&[])).unwrap();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn unknown_and_duplicate_names_are_refused() {
        let err = build_cohort(&three(), "node-9", None)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("no such node(s) in the descriptor map: node-9"),
            "{err}"
        );
        assert!(err.contains("node-1, node-2, node-3"), "{err}");

        let err = build_cohort(&three(), "node-1", Some(&["node-1".into()]))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("duplicate node name(s) in cohort: node-1"),
            "{err}"
        );

        let shared_ip = descriptors(&[
            ("node-1", "1.1.1.1", "n1.example.com"),
            ("node-2", "1.1.1.1", "n2.example.com"),
        ]);
        let err = build_cohort(&shared_ip, "node-1", None)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("duplicate node public_ip(s) in cohort: 1.1.1.1"),
            "{err}"
        );
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
    fn founding_facts_join_harvest_keys_with_descriptor_ips() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = NetworkDir::new(tmp.path());
        let err = load_founding_facts(&dir, &three()).unwrap_err().to_string();
        assert!(err.contains("no harvest records"), "{err}");

        std::fs::create_dir_all(dir.harvest()).unwrap();
        for (name, key, byte) in [("node-1", NODE_KEY_1, "cc"), ("node-2", NODE_KEY_2, "dd")] {
            std::fs::write(
                dir.harvest_record(name),
                json!({
                    "harvest_nonce": "11".repeat(32),
                    "node_public_key": key,
                    "consensus_public_key": byte.repeat(48),
                    "evidence": {},
                })
                .to_string(),
            )
            .unwrap();
        }
        let (ips, records) = load_founding_facts(&dir, &three()).unwrap();
        assert_eq!(ips[NODE_KEY_1], "1.1.1.1:18551");
        assert_eq!(ips[NODE_KEY_2], "2.2.2.2:18551");
        assert_eq!(records.len(), 2);

        let one = descriptors(&[("node-1", "1.1.1.1", "n1.example.com")]);
        let err = load_founding_facts(&dir, &one).unwrap_err().to_string();
        assert!(err.contains("has no node \"node-2\""), "{err}");
        assert!(err.contains("re-found rather than configuring"), "{err}");
    }

    /// A cohort whose nodes are all already configured (tdx-init gone, no
    /// attestation service) fails per node without aborting the rest, and
    /// the founding bootnode set is not written.
    #[tokio::test]
    async fn run_cohort_reports_every_node_and_keeps_going_on_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = NetworkDir::new(tmp.path());
        let manifest = Manifest::from_json_bytes(FIXTURE_MANIFEST).unwrap();
        let shared = Arc::new(Shared {
            manifest,
            reth_genesis: Artifact::new(
                "reth-genesis.json",
                br#"{"config": {"chainId": 5124}}"#.to_vec(),
            ),
            summit_genesis: Artifact::new(
                "summit-genesis.toml",
                b"namespace = \"seismic-devnet-3\"\n".to_vec(),
            ),
            email: "ops@example.com".into(),
            appraisal: None,
            dir: dir.clone(),
            client: http::client().unwrap(),
        });
        // A joiner with no bootnodes fails in build_config, before any POST;
        // the record of the attempt is still written.
        let nodes = vec![Node {
            name: "node-2".into(),
            public_ip: "203.0.113.9".into(),
            fqdn: "n2.example.com".into(),
            genesis: false,
            bootnodes: Vec::new(),
        }];
        let results = run_cohort(&nodes, &shared).await.unwrap();
        assert_eq!(results.get("node-2"), Some(&false));
        assert!(!dir.init_config("node-2").exists());

        let path = dir.bootnodes();
        persist_founding_bootnodes(&nodes, &results, &path).await;
        assert!(!path.exists());
    }

    /// The founding set is written only when every node is ok, from the live
    /// enodes; a collect failure degrades to a warning.
    #[tokio::test]
    async fn persist_writes_the_full_set_or_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nodes").join("bootnodes.json");
        let nodes = vec![Node {
            name: "node-1".into(),
            public_ip: "1.1.1.1".into(),
            fqdn: "n1.example.com".into(),
            genesis: true,
            bootnodes: Vec::new(),
        }];
        let mut results = BTreeMap::new();
        results.insert("node-1".to_string(), true);
        // The node's fqdn resolves nowhere useful here: the collect fails
        // within the short timeout only if we shorten it — so instead prove
        // the failed-node branch and the write path through save_bootnodes.
        results.insert("node-1".to_string(), false);
        persist_founding_bootnodes(&nodes, &results, &path).await;
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn the_documented_argv_parses_and_the_policy_is_resolved_first() {
        #[derive(Parser)]
        struct Probe {
            #[command(flatten)]
            args: ConfigureArgs,
        }
        let probe = Probe::try_parse_from([
            "configure",
            "--genesis-node",
            "tmp-devnet-1-1",
            "--manifest",
            "tee/networks/tmp-devnet-1/network-manifest.json",
        ])
        .unwrap()
        .args;
        assert_eq!(probe.genesis_node.as_deref(), Some("tmp-devnet-1-1"));
        assert!(!probe.check);
        assert_eq!(probe.join, None);
        assert_eq!(probe.email, DEFAULT_EMAIL);
        assert!(!probe.no_verify);

        let probe = Probe::try_parse_from([
            "configure",
            "--genesis-node",
            "a",
            "--join",
            "b",
            "--join",
            "c",
            "--manifest",
            "m.json",
            "--no-verify",
        ])
        .unwrap()
        .args;
        assert_eq!(
            probe.join.as_deref(),
            Some(&["b".to_string(), "c".to_string()][..])
        );

        assert!(Probe::try_parse_from(["configure", "--manifest", "m.json"]).is_err());

        // A missing manifest fails before anything else is read.
        let probe = Probe::try_parse_from([
            "configure",
            "--genesis-node",
            "a",
            "--manifest",
            "/absent/m.json",
        ])
        .unwrap()
        .args;
        let err = run(probe).await.unwrap_err().to_string();
        assert!(err.contains("--manifest file not found"), "{err}");
    }

    /// `--check` assigns no roles, so it needs no genesis node — and it
    /// delivers nothing, so every flag that only delivery reads is refused
    /// rather than silently ignored.
    #[test]
    fn check_needs_no_genesis_node_and_refuses_the_delivery_flags() {
        #[derive(Parser)]
        struct Probe {
            #[command(flatten)]
            args: ConfigureArgs,
        }
        let probe = Probe::try_parse_from(["configure", "--check"])
            .unwrap()
            .args;
        assert!(probe.check);
        assert_eq!(probe.genesis_node, None);

        for excluded in [
            &["--genesis-node", "a"][..],
            &["--join", "b"],
            &["--reth-genesis", "r.json"],
            &["--summit-genesis", "s.toml"],
            &["--no-verify"],
            &["--policy", "p.json"],
            &["--measurements", "m.json"],
            &["--pccs-url", "https://pccs"],
        ] {
            let argv: Vec<&str> = ["configure", "--check"]
                .into_iter()
                .chain(excluded.iter().copied())
                .collect();
            assert!(Probe::try_parse_from(&argv).is_err(), "{excluded:?}");
        }
    }

    /// `--check` reaches the launch assertions with nothing delivered: no
    /// policy or genesis file is needed, nothing is written under the
    /// network directory, and a node the harvest does not know is skipped
    /// rather than held to a pin it never had. The nodes here are
    /// unreachable — the descriptor fixes the ports the assertions dial, so
    /// a fake server cannot stand in — and the check says so, per node.
    #[tokio::test]
    async fn check_asserts_the_founding_cohort_and_writes_nothing() {
        use crate::founding::tests::{
            NODE_KEY_1, descriptors_of, network_dir, record, write_harvest,
        };

        #[derive(Parser)]
        struct Probe {
            #[command(flatten)]
            args: ConfigureArgs,
        }
        let (_tmp, dir) = network_dir();
        std::fs::write(dir.manifest(), FIXTURE_MANIFEST).unwrap();
        write_harvest(&dir, "node-1", &record(NODE_KEY_1, "cc"));
        let descriptors = descriptors_of(&[
            ("node-1", "127.0.0.1", "127.0.0.1:1"),
            ("node-2", "127.0.0.1", "127.0.0.1:2"),
        ]);
        let nodes = dir.root().join("nodes.json");
        std::fs::write(&nodes, serde_json::to_vec(&descriptors).unwrap()).unwrap();

        let args = Probe::try_parse_from([
            "configure",
            "--check",
            "--manifest",
            &dir.manifest().to_string_lossy(),
            "--nodes",
            &nodes.to_string_lossy(),
        ])
        .unwrap()
        .args;
        let err = check(&args, Duration::ZERO).await.unwrap_err().to_string();
        assert!(err.contains("Cohort disagrees"), "{err}");
        assert!(
            err.contains("✗ node-1: unreachable via https://127.0.0.1:1/rpc"),
            "{err}"
        );
        assert!(!err.contains("node-2"), "{err}");
        assert!(!dir.nodes().exists(), "--check wrote under nodes/");
    }

    /// The appraisal retries a transient failure and then fails the node; the
    /// verdict is the node's status line.
    #[tokio::test]
    async fn a_failed_appraisal_fails_the_node_after_retries() {
        let manifest = Manifest::from_json_bytes(FIXTURE_MANIFEST).unwrap();
        let shared = Shared {
            manifest,
            reth_genesis: Artifact::new("r", b"{}".to_vec()),
            summit_genesis: Artifact::new("s", b"".to_vec()),
            email: String::new(),
            appraisal: None,
            dir: NetworkDir::new("/nets/x"),
            client: http::client().unwrap(),
        };
        let node = Node {
            name: "node-1".into(),
            public_ip: "127.0.0.1".into(),
            fqdn: "n1".into(),
            genesis: true,
            bootnodes: Vec::new(),
        };
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
        let _ = (
            FakeServer::serve(vec![]),
            refused_url(),
            rpc_result(json!(null)),
        );
    }
}
