//! Launch assertions: prove the cohort that launched is the cohort pinned.
//!
//! Run by `configure` after every node accepts its config, these are the
//! founding design's must-build guard: admission alone cannot catch a founder
//! that rebooted inside the harvest → LUKS-open window, because such a box
//! regenerates fresh RAM keys, persists *those*, and passes admission fine —
//! launching a validator whose pinned pubkey nobody holds (a silent dead
//! consensus slot). Two checks, both against values the network manifest
//! pins, both hard failures:
//!
//! 1. **reth block 0** — every node's live reth must serve the manifest's
//!    `eth.genesis_hash` as block 0. summit uses that hash as its initial
//!    forkchoice head, and a hash reth doesn't know parks reth in SYNCING
//!    forever with no error on either side. This check doubles as the cohort
//!    barrier: reth answers only after the root_key → LUKS-open boot tail, so
//!    it is polled with the same dashboard + disk-provisioning pause the
//!    config watch uses. A *wrong* answer fails immediately — waiting can't
//!    fix a node booted from a stale image or different genesis.
//!
//! 2. **holder keys** — every box's summit-key-holder (`GET /v1/keys`) must
//!    serve exactly the pubkeys harvested from it, i.e. the keys the
//!    assembled genesis pins. A mismatch is retried until the deadline, not
//!    failed fast: the holder serves this boot's RAM keys until the LUKS
//!    volume is open and the keystore visible, so an early read can
//!    transiently show fresh unpinned keys on a healthy node. A mismatch that
//!    *persists* is the dead-slot case — the fix is a re-found (`pulumi
//!    destroy` + fresh `up`), never launching around it.
//!
//! Each node is located by its descriptor-map entry; the expected keys come
//! from the committed harvest records (`inputs/harvest/<node>.json`), whose
//! pairing with the pinned validator set `assemble` already enforced.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use anyhow::bail;
use jsonrpsee::rpc_params;
use seismic_tee_common::{NodeDescriptor, rpc};
use seismic_tee_node::status::{LuksProvisioningStatus, fetch_status, format_provisioning};
use serde_json::Value;

use crate::dashboard::CohortDashboard;
use crate::founding::FoundingRecord;
use crate::gates::hex_0x;

/// Cohort-readiness polling. `configure` watches the first-boot disk wipe; if
/// the reth probe observes one still running, it displays that progress and
/// pauses the residual readiness timeout.
pub const POLL_INTERVAL: Duration = Duration::from_secs(5);
pub const READY_TIMEOUT: Duration = Duration::from_secs(15 * 60);
/// How long `configure --check` waits for a node that does not answer. The
/// cohort is supposed to be up, so this covers a blip, not a boot: a node
/// still down after it is reported as such, and the check is re-run.
pub const CHECK_TIMEOUT: Duration = Duration::from_secs(30);
pub const WAIT_LOG_INTERVAL: Duration = Duration::from_secs(30);
/// The status read beside a reth probe is a side glance, not the wait.
const STATUS_GLANCE: Duration = Duration::from_secs(2);

/// One configured cohort box and the founding keys pinned for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchTarget {
    pub name: String,
    /// `https://<fqdn>/rpc`.
    pub eth_rpc_url: String,
    /// `http://<ip>:7878`.
    pub attestation_rpc_url: String,
    /// `http://<ip>:7879`.
    pub holder_url: String,
    pub node_public_key: String,
    pub consensus_public_key: String,
}

impl LaunchTarget {
    pub fn new(name: &str, descriptor: &NodeDescriptor, record: &FoundingRecord) -> Self {
        Self {
            name: name.to_string(),
            eth_rpc_url: descriptor.eth_rpc_url(),
            attestation_rpc_url: descriptor.attestation_rpc_url(),
            holder_url: descriptor.key_holder_url(),
            node_public_key: record.node_public_key.clone(),
            consensus_public_key: record.consensus_public_key.clone(),
        }
    }
}

/// Block 0's hash as reth serves it, lowercase. `client` is the node's public
/// JSON-RPC (`https://<fqdn>/rpc`).
pub async fn fetch_block0_hash(client: &rpc::Client) -> anyhow::Result<String> {
    let block = client
        .call("eth_getBlockByNumber", rpc_params!["0x0", false])
        .await?;
    match block.get("hash").and_then(Value::as_str) {
        Some(hash) => Ok(hash.to_lowercase()),
        None => bail!(
            "eth_getBlockByNumber returned no block 0 from {}: {block}",
            client.url()
        ),
    }
}

/// Round `elapsed` down and `remaining` up to the log interval, so the
/// dashboard's countdown ticks in the same steps a log line would.
fn countdown(started: Instant, deadline: Instant, now: Instant) -> (u64, u64) {
    let step = WAIT_LOG_INTERVAL.as_secs();
    let elapsed = now.duration_since(started).as_secs();
    let remaining = deadline.saturating_duration_since(now).as_secs();
    (elapsed - elapsed % step, remaining.div_ceil(step) * step)
}

/// Assert every cohort node's reth serves `expected` as block 0.
///
/// A node that doesn't answer is polled until `timeout` — reth comes up only
/// after root_key → LUKS open, so early unreachability is the normal boot
/// tail. Active disk provisioning is shown through the shared cohort dashboard
/// and pauses this timeout. A *wrong* answer fails immediately: waiting can't
/// fix a node booted from a stale image or different genesis, and which nodes
/// match is exactly the diagnostic (one stale node vs. a manifest that matches
/// nobody), so the failure lists the whole cohort.
pub async fn assert_cohort_genesis_hash(
    targets: &[LaunchTarget],
    expected: [u8; 32],
    timeout: Duration,
    interval: Duration,
) -> anyhow::Result<()> {
    let expected = hex_0x(&expected);
    let eth_clients: BTreeMap<&str, rpc::Client> = targets
        .iter()
        .map(|t| Ok((t.name.as_str(), rpc::Client::new(&t.eth_rpc_url)?)))
        .collect::<anyhow::Result<_>>()?;
    let status_clients: BTreeMap<&str, rpc::Client> = targets
        .iter()
        .map(|t| Ok((t.name.as_str(), rpc::Client::new(&t.attestation_rpc_url)?)))
        .collect::<anyhow::Result<_>>()?;

    let mut dashboard = CohortDashboard::new(
        targets
            .iter()
            .map(|t| (t.name.clone(), t.name.clone()))
            .collect(),
    );
    let mut states: BTreeMap<String, String> = targets
        .iter()
        .map(|t| (t.name.clone(), "waiting for reth block 0".to_string()))
        .collect();
    let mut observed: BTreeMap<&str, String> = BTreeMap::new();
    let mut last_error: BTreeMap<&str, String> = BTreeMap::new();
    let started = Instant::now();
    let mut deadline = started + timeout;
    let mut provisioning_active = false;
    let mut mismatch;
    let mut pending: Vec<&LaunchTarget>;
    loop {
        let iteration_started = Instant::now();
        for target in targets {
            if observed.contains_key(target.name.as_str()) {
                continue;
            }
            match fetch_block0_hash(&eth_clients[target.name.as_str()]).await {
                Ok(hash) => {
                    states.insert(
                        target.name.clone(),
                        if hash == expected {
                            "reth block 0 matches".to_string()
                        } else {
                            format!("wrong reth block 0: {hash}")
                        },
                    );
                    observed.insert(&target.name, hash);
                }
                Err(error) => {
                    last_error.insert(
                        &target.name,
                        format!("unreachable via {}: {error:#}", target.eth_rpc_url),
                    );
                }
            }
        }
        pending = targets
            .iter()
            .filter(|t| !observed.contains_key(t.name.as_str()))
            .collect();
        mismatch = observed.values().any(|hash| *hash != expected);
        if mismatch || pending.is_empty() {
            dashboard.render(&states);
            break;
        }

        let reth_probe_finished = Instant::now();
        if provisioning_active {
            deadline += reth_probe_finished - iteration_started;
        }

        let mut provisioning: BTreeSet<&str> = BTreeSet::new();
        let mut details: BTreeMap<&str, String> = BTreeMap::new();
        for target in &pending {
            let name = target.name.as_str();
            let status =
                tokio::time::timeout(STATUS_GLANCE, fetch_status(&status_clients[name])).await;
            let Ok(Ok(result)) = status else {
                details.insert(name, "waiting for reth block 0".to_string());
                continue;
            };
            match LuksProvisioningStatus::from_result(&result) {
                LuksProvisioningStatus::Provisioning {
                    bytes_done,
                    bytes_total,
                    eta_seconds,
                } => {
                    provisioning.insert(name);
                    states.insert(
                        name.to_string(),
                        format!(
                            "{}  (readiness timeout paused)",
                            format_provisioning(bytes_done, bytes_total, eta_seconds)
                        ),
                    );
                }
                LuksProvisioningStatus::Error { error } => {
                    details.insert(
                        name,
                        format!("disk provisioning error, auto-retrying: {error}"),
                    );
                }
                LuksProvisioningStatus::Idle => {
                    details.insert(name, "disk idle; waiting for reth block 0".to_string());
                }
                LuksProvisioningStatus::Unknown => {
                    details.insert(
                        name,
                        "disk status \"unknown\"; waiting for reth block 0".to_string(),
                    );
                }
                LuksProvisioningStatus::Other(state) => {
                    details.insert(
                        name,
                        format!("disk status {state:?}; waiting for reth block 0"),
                    );
                }
            }
        }

        let now = Instant::now();
        if !provisioning.is_empty() {
            let paused_since = if provisioning_active {
                reth_probe_finished
            } else {
                iteration_started
            };
            deadline += now - paused_since;
        }
        let (elapsed, remaining) = countdown(started, deadline, now);
        for target in &pending {
            let name = target.name.as_str();
            if !provisioning.contains(name) {
                states.insert(
                    name.to_string(),
                    format!(
                        "{} ({elapsed}s elapsed, {remaining}s until timeout)",
                        details[name]
                    ),
                );
            }
        }
        dashboard.render(&states);

        if now >= deadline {
            break;
        }
        let sleep_started = Instant::now();
        tokio::time::sleep(interval).await;
        if !provisioning.is_empty() {
            deadline += sleep_started.elapsed();
        }
        provisioning_active = !provisioning.is_empty();
    }

    if mismatch || !pending.is_empty() {
        let listing: Vec<String> = targets
            .iter()
            .map(|t| match observed.get(t.name.as_str()) {
                Some(hash) if *hash == expected => format!("  ✓ {}: {hash}", t.name),
                Some(hash) => format!("  ✗ {}: {hash}", t.name),
                None => format!(
                    "  ✗ {}: {}",
                    t.name,
                    last_error
                        .get(t.name.as_str())
                        .map_or("never answered", String::as_str)
                ),
            })
            .collect();
        bail!(
            "Cohort disagrees with the pinned eth_genesis_hash (stale image, wrong reth genesis, \
             or a node that never became ready); the launch assertion failed:\n    pinned: \
             {expected}\n{}",
            listing.join("\n")
        );
    }
    Ok(())
}

/// Why one holder read did not produce keys.
#[derive(Debug)]
pub enum HolderError {
    /// Transport/HTTP failure: callers retry a still-booting box.
    Retry(String),
    /// A malformed response body: retrying can't fix a holder serving the
    /// wrong shape.
    Malformed(String),
}

/// Fetch one box's live summit pubkeys from its summit-key-holder.
pub async fn fetch_holder_keys(
    client: &reqwest::Client,
    holder_url: &str,
) -> Result<(String, String), HolderError> {
    let url = format!("{holder_url}/v1/keys");
    let response = client
        .get(&url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| HolderError::Retry(e.to_string()))?;
    let body: Value = response
        .json()
        .await
        .map_err(|e| HolderError::Malformed(format!("{url}: response is not JSON: {e}")))?;
    if !body.is_object() {
        return Err(HolderError::Malformed(format!(
            "{url}: expected a JSON object, got {body}"
        )));
    }
    match (
        body.get("node_public_key").and_then(Value::as_str),
        body.get("consensus_public_key").and_then(Value::as_str),
    ) {
        (Some(node), Some(consensus)) => Ok((node.to_string(), consensus.to_string())),
        _ => Err(HolderError::Malformed(format!(
            "{url}: response carries no node_public_key/consensus_public_key strings: {body}"
        ))),
    }
}

fn describe_served(served: Option<&(String, String)>, error: Option<&String>) -> String {
    match served {
        Some((node, consensus)) => format!("serving node={node} consensus={consensus}"),
        None => error
            .cloned()
            .unwrap_or_else(|| "never answered".to_string()),
    }
}

/// Assert every box's holder serves exactly its pinned founding keys.
///
/// A mismatch is retried, not failed fast: until the LUKS volume is open and
/// the keystore visible, the holder serves this boot's fresh RAM keys, so an
/// early read on a healthy rebooted node can transiently disagree with the
/// pin. A mismatch still standing at the deadline is the real failure — a box
/// persisted keys the manifest never pinned (a reboot inside the founding
/// window), and the network must not be trusted as launched: re-found rather
/// than running with a dead consensus slot.
pub async fn assert_cohort_holder_keys(
    client: &reqwest::Client,
    targets: &[LaunchTarget],
    timeout: Duration,
    interval: Duration,
) -> anyhow::Result<()> {
    let mut served: BTreeMap<&str, (String, String)> = BTreeMap::new();
    let mut matched: BTreeSet<&str> = BTreeSet::new();
    let mut last_error: BTreeMap<&str, String> = BTreeMap::new();
    let started = Instant::now();
    let deadline = started + timeout;
    let mut next_log = started;
    loop {
        for target in targets {
            let name = target.name.as_str();
            if matched.contains(name) {
                continue;
            }
            match fetch_holder_keys(client, &target.holder_url).await {
                Ok(keys) => {
                    if keys.0 == target.node_public_key && keys.1 == target.consensus_public_key {
                        matched.insert(name);
                        println!("  ✓ {name}: holder serves its pinned founding keys");
                    }
                    served.insert(name, keys);
                }
                Err(HolderError::Malformed(message)) => bail!("{name}: {message}"),
                Err(HolderError::Retry(message)) => {
                    last_error.insert(name, message);
                }
            }
        }
        let pending: Vec<&str> = targets
            .iter()
            .map(|t| t.name.as_str())
            .filter(|name| !matched.contains(name))
            .collect();
        if pending.is_empty() {
            return Ok(());
        }
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        if now >= next_log {
            println!(
                "waiting for pinned holder keys ({}s elapsed, {}s until timeout): {}",
                now.duration_since(started).as_secs(),
                deadline.saturating_duration_since(now).as_secs(),
                pending.join(", ")
            );
            next_log = now + WAIT_LOG_INTERVAL;
        }
        tokio::time::sleep(interval).await;
    }

    let listing: Vec<String> = targets
        .iter()
        .map(|t| {
            let name = t.name.as_str();
            if matched.contains(name) {
                format!("  ✓ {name}: holder serves its pinned founding keys")
            } else {
                format!(
                    "  ✗ {name}: {}",
                    describe_served(served.get(name), last_error.get(name))
                )
            }
        })
        .collect();
    let mismatched = targets
        .iter()
        .any(|t| !matched.contains(t.name.as_str()) && served.contains_key(t.name.as_str()));
    let advice = if mismatched {
        "A box still serving keys the manifest never pinned launched from a reboot inside the \
         founding window — its pinned validator slot is dead. Re-found (`pulumi destroy` + fresh \
         `up`) rather than running degraded."
    } else {
        "Holders that never answered may still be booting — re-assert once the cohort settles: \
         `seismic-tee network configure --check`."
    };
    bail!(
        "{} box(es) not serving their pinned founding keys after {}s; the launch assertion \
         failed:\n{}\n{advice}",
        targets.len() - matched.len(),
        timeout.as_secs(),
        listing.join("\n")
    );
}

#[cfg(test)]
mod tests {
    use seismic_tee_common::http;
    use seismic_tee_common::test_support::{FakeServer, refused_url, rpc_result};
    use serde_json::json;

    use super::*;

    const HASH: [u8; 32] = [0xab; 32];

    fn block(hash: &str) -> String {
        rpc_result(json!({"number": "0x0", "hash": hash}))
    }

    fn target(name: &str, eth_rpc_url: &str, holder_url: &str) -> LaunchTarget {
        LaunchTarget {
            name: name.to_string(),
            eth_rpc_url: eth_rpc_url.to_string(),
            attestation_rpc_url: refused_url(),
            holder_url: holder_url.to_string(),
            node_public_key: "aa".repeat(32),
            consensus_public_key: "cc".repeat(48),
        }
    }

    fn keys(node: &str, consensus: &str) -> String {
        json!({"node_public_key": node, "consensus_public_key": consensus}).to_string()
    }

    async fn genesis(targets: &[LaunchTarget], timeout: Duration) -> anyhow::Result<()> {
        assert_cohort_genesis_hash(targets, HASH, timeout, Duration::from_millis(20)).await
    }

    async fn holders(targets: &[LaunchTarget], timeout: Duration) -> anyhow::Result<()> {
        assert_cohort_holder_keys(
            &http::client().unwrap(),
            targets,
            timeout,
            Duration::from_millis(20),
        )
        .await
    }

    #[tokio::test]
    async fn a_matching_cohort_passes_case_insensitively() {
        let n1 = FakeServer::serve(vec![(200, block(&hex_0x(&HASH)))]);
        let n2 = FakeServer::serve(vec![(
            200,
            block(&hex_0x(&HASH).to_uppercase().replace("0X", "0x")),
        )]);
        genesis(
            &[
                target("node-1", &n1.url, "http://h1"),
                target("node-2", &n2.url, "http://h2"),
            ],
            Duration::from_secs(10),
        )
        .await
        .unwrap();
        let body: Value = serde_json::from_slice(&n1.requests()[0].body).unwrap();
        assert_eq!(body["method"], "eth_getBlockByNumber");
        assert_eq!(body["params"], json!(["0x0", false]));
    }

    /// A wrong answer fails at once, without waiting for stragglers, and the
    /// listing covers the whole cohort.
    #[tokio::test]
    async fn a_mismatching_node_fails_fast_listing_the_whole_cohort() {
        let good = FakeServer::serve(vec![(200, block(&hex_0x(&HASH)))]);
        let bad = FakeServer::serve(vec![(200, block(&hex_0x(&[0xff; 32])))]);
        let started = Instant::now();
        let err = genesis(
            &[
                target("node-1", &good.url, "http://h1"),
                target("node-2", &bad.url, "http://h2"),
                target("node-3", &refused_url(), "http://h3"),
            ],
            Duration::from_secs(30),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "waited for the straggler"
        );
        assert!(err.contains("Cohort disagrees"), "{err}");
        assert!(err.contains(&format!("pinned: {}", hex_0x(&HASH))), "{err}");
        assert!(
            err.contains(&format!("✓ node-1: {}", hex_0x(&HASH))),
            "{err}"
        );
        assert!(
            err.contains(&format!("✗ node-2: {}", hex_0x(&[0xff; 32]))),
            "{err}"
        );
        assert!(err.contains("✗ node-3: unreachable via"), "{err}");
    }

    #[tokio::test]
    async fn an_unreachable_node_is_polled_until_it_answers_or_the_deadline() {
        let late = FakeServer::serve_after(
            Duration::from_millis(200),
            vec![(200, block(&hex_0x(&HASH)))],
        );
        genesis(
            &[target("node-1", &late.url, "http://h1")],
            Duration::from_secs(10),
        )
        .await
        .unwrap();

        let err = genesis(
            &[target("node-1", &refused_url(), "http://h1")],
            Duration::from_millis(100),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(err.contains("never became ready"), "{err}");
        assert!(err.contains("✗ node-1: unreachable via"), "{err}");
    }

    /// An RPC error object is a wrong answer's neighbour: reth is up but does
    /// not serve block 0 — reported and retried like unreachability, since a
    /// node mid-init can answer this way.
    #[tokio::test]
    async fn an_rpc_error_is_reported_as_the_last_error() {
        let not_ready =
            r#"{"jsonrpc": "2.0", "id": 1, "error": {"code": -32000, "message": "not ready"}}"#;
        let server = FakeServer::serve(vec![(200, not_ready.to_string()); 40]);
        let err = genesis(
            &[target("node-1", &server.url, "http://h1")],
            Duration::from_millis(100),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(err.contains("not ready"), "{err}");
    }

    /// A disk wipe in progress is shown and pauses the readiness deadline:
    /// with a status server reporting provisioning, a deadline shorter than
    /// the run does not fire.
    #[tokio::test]
    async fn a_running_wipe_pauses_the_readiness_timeout() {
        let provisioning =
            rpc_result(json!({"state": "provisioning", "bytes_done": 1, "bytes_total": 10}));
        let status = FakeServer::serve(vec![
            (200, provisioning.clone()),
            (200, provisioning.clone()),
            (200, provisioning.clone()),
            (200, provisioning),
            (200, rpc_result(json!({"state": "idle"}))),
        ]);
        let reth = FakeServer::serve_after(
            Duration::from_millis(350),
            vec![(200, block(&hex_0x(&HASH)))],
        );
        let mut target = target("node-1", &reth.url, "http://h1");
        target.attestation_rpc_url = status.url.clone();
        // Would fire after 150ms without the pause; the wipe holds it open
        // long enough for reth to answer.
        assert_cohort_genesis_hash(
            &[target],
            HASH,
            Duration::from_millis(150),
            Duration::from_millis(50),
        )
        .await
        .unwrap();
    }

    #[test]
    fn the_countdown_ticks_in_log_steps() {
        let started = Instant::now();
        let (elapsed, remaining) = countdown(
            started,
            started + Duration::from_secs(900),
            started + Duration::from_secs(47),
        );
        assert_eq!((elapsed, remaining), (30, 870));
        let (elapsed, remaining) = countdown(
            started,
            started + Duration::from_secs(10),
            started + Duration::from_secs(20),
        );
        assert_eq!((elapsed, remaining), (0, 0));
    }

    #[tokio::test]
    async fn matching_holders_pass_and_hit_the_keys_endpoint() {
        let h1 = FakeServer::serve(vec![(200, keys(&"aa".repeat(32), &"cc".repeat(48)))]);
        holders(
            &[target("node-1", "http://r", &h1.url)],
            Duration::from_secs(10),
        )
        .await
        .unwrap();
        assert_eq!(h1.requests()[0].path, "/v1/keys");
        assert_eq!(h1.requests()[0].method, "GET");
    }

    /// A transient mismatch (fresh RAM keys before the keystore is visible)
    /// is retried until the pinned keys appear.
    #[tokio::test]
    async fn a_transient_mismatch_is_retried_until_the_keystore_is_visible() {
        let holder = FakeServer::serve(vec![
            (200, keys(&"ff".repeat(32), &"ee".repeat(48))),
            (200, keys(&"aa".repeat(32), &"cc".repeat(48))),
        ]);
        holders(
            &[target("node-1", "http://r", &holder.url)],
            Duration::from_secs(10),
        )
        .await
        .unwrap();
        assert_eq!(holder.requests().len(), 2);
    }

    #[tokio::test]
    async fn a_standing_mismatch_exits_with_refound_advice() {
        let fresh = keys(&"ff".repeat(32), &"ee".repeat(48));
        let holder = FakeServer::serve(vec![(200, fresh.clone()); 40]);
        let err = holders(
            &[target("node-1", "http://r", &holder.url)],
            Duration::from_millis(100),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("1 box(es) not serving their pinned founding keys"),
            "{err}"
        );
        assert!(
            err.contains(&format!("✗ node-1: serving node={}", "ff".repeat(32))),
            "{err}"
        );
        assert!(err.contains("Re-found"), "{err}");
    }

    #[tokio::test]
    async fn an_unreachable_holder_is_retried_then_told_to_reassert() {
        let late = FakeServer::serve_after(
            Duration::from_millis(200),
            vec![(200, keys(&"aa".repeat(32), &"cc".repeat(48)))],
        );
        holders(
            &[target("node-1", "http://r", &late.url)],
            Duration::from_secs(10),
        )
        .await
        .unwrap();

        let err = holders(
            &[target("node-1", "http://r", &refused_url())],
            Duration::from_millis(100),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(err.contains("configure --check"), "{err}");
        assert!(!err.contains("Re-found"), "{err}");
    }

    #[tokio::test]
    async fn a_malformed_holder_response_exits_immediately() {
        let holder = FakeServer::serve(vec![(200, "[]".to_string()), (200, "{}".to_string())]);
        let err = holders(
            &[target("node-1", "http://r", &holder.url)],
            Duration::from_secs(10),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(err.starts_with("node-1: "), "{err}");
        assert!(err.contains("expected a JSON object"), "{err}");
        assert_eq!(holder.requests().len(), 1);
    }
}
