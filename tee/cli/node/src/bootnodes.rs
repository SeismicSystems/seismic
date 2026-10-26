//! Bootnodes: where a node's `[network].bootnodes` come from.
//!
//! tdx-init derives a node's `root_key` fetch peers from its bootnodes
//! (`http://<host>:7878`), and reth dials them, so a bootnode is a box the
//! node is about to trust with its key. Two sources, as [`crate::plan`]
//! decides:
//!
//! - **Founding.** A freshly-configured reth node's enode (its devp2p node
//!   record) isn't known until reth is up, so a greenfield cohort can't be
//!   handed a static bootnode list up front: the pinned box comes up first,
//!   then its enode seeds the rest (the two-stage bootstrap in
//!   [`crate::cohort`]), polled with [`collect_enodes`].
//! - **Joining.** Every other config POST goes to a live network, whose key
//!   holders (nodes serving `:7878`) the [`crate::probe`] of the node table
//!   found ([`from_key_holders`]). Nothing is remembered between runs, so
//!   each run re-applies the founding's rule that a node's enode reaches
//!   others only after it passes its appraisal: every key holder is
//!   deploy-verified, and only those that pass are handed out. With none,
//!   the run refuses before anything is sent.
//!
//! `seismic_nodeInfo` is read from reth's JSON-RPC on the node's IP
//! (`http://<ip>:8545`), the IP the key holder's appraisal names, with the
//! same client the attestation service is read with ([`rpc::Client`]); the `seismic` namespace
//! deliberately keeps nodeInfo public (reth's `admin` namespace is disabled on
//! these nodes).
//!
//! reth's NodeRecord Display appends `?discport=<udp_port>` when its devp2p
//! UDP port differs from its TCP port; tdx-init rejects that form at POST time
//! (the `[network].bootnodes` grammar is strictly `enode://<128 hex>@host:port`).
//! So every fetched enode is normalized ([`normalize_enode`]) — the query
//! dropped, bracketed IPv6 preserved — before it is delivered.

use std::collections::BTreeMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use jsonrpsee::rpc_params;
use seismic_tee_common::{Manifest, NodeDescriptor, rpc};
use tdx_init_config::{Bootnode, BootnodeHost};
use tokio::task::JoinSet;

use crate::cohort::Appraisal;
use crate::plan::Remedy;
use crate::verify::challenge_node;

/// Enode-readiness polling. reth serves `seismic_nodeInfo` only once it is up
/// (root_key → LUKS open → reth start), so an early miss is the normal boot
/// tail — polled until timeout.
pub const POLL_INTERVAL: Duration = Duration::from_secs(5);
pub const ENODE_TIMEOUT: Duration = Duration::from_secs(15 * 60);
pub const WAIT_LOG_INTERVAL: Duration = Duration::from_secs(30);

/// Coerce a NodeRecord enode into the strict `enode://<128 hex>@host:port` form
/// tdx-init accepts, dropping any `?discport=` query reth appends when its
/// devp2p TCP and UDP ports differ (seismic nodes run them equal, so this is
/// normally a no-op). Fails if the record isn't a well-formed enode — a
/// founding node advertising a malformed record is a hard error, not something
/// to deliver verbatim.
pub fn normalize_enode(enode: &str) -> anyhow::Result<Bootnode> {
    let (url, query) = match enode.split_once('?') {
        Some((url, query)) => (url, Some(query)),
        None => (enode, None),
    };
    let bootnode: Bootnode = url.parse()?;
    if let Some(query) = query {
        eprintln!(
            "warning: enode {enode} carries a query string ({query}); dropping it — tdx-init \
             accepts only enode://<id>@host:port. seismic nodes run matched devp2p TCP/UDP ports, \
             so this is normally absent."
        );
    }
    Ok(bootnode)
}

/// Whether an enode's host is the IP address `ip`, by value: an IPv6
/// address matches however either side spells it.
pub fn is_at_ip(enode: &Bootnode, ip: IpAddr) -> bool {
    *enode.host() == BootnodeHost::Ip(ip)
}

/// Warn (never fail) if an enode's host isn't the node's public IP.
///
/// With reth's `--nat extip <public_ip>` (from `[node].external_ip`) the
/// advertised enode host should equal the descriptor's public_ip. A mismatch
/// usually means extip wasn't applied, so reth advertised a private or
/// auto-detected address peers can't dial — worth surfacing loudly, but not
/// fatal to configuration.
pub fn warn_on_ip_mismatch(enode: &Bootnode, expected_ip: IpAddr, label: &str) {
    if !is_at_ip(enode, expected_ip) {
        eprintln!(
            "warning: {label}: enode host {} does not match descriptor public_ip {expected_ip} — \
             is reth's --nat extip set to the public IP? peers may be unable to reach this node's \
             enode.",
            enode.host()
        );
    }
}

/// Fetch a node's enode URL, as advertised, from its `seismic_nodeInfo` RPC.
/// Fails on transport/RPC error or a response without an enode, so callers
/// can retry a still-booting node.
pub async fn fetch_enode(client: &rpc::Client) -> anyhow::Result<String> {
    let result = client.call("seismic_nodeInfo", rpc_params![]).await?;
    match result.get("enode").and_then(serde_json::Value::as_str) {
        Some(enode) if !enode.is_empty() => Ok(enode.to_string()),
        _ => bail!(
            "seismic_nodeInfo returned no enode from {}: {result}",
            client.url()
        ),
    }
}

/// Poll every `(label, url)` target's `seismic_nodeInfo` until each returns an
/// enode, or `timeout` elapses. Returns `{label: normalized enode}`.
///
/// Round-robin like the other cohort gathers, so a slow node doesn't
/// serialize behind the others. A node still silent at the deadline aborts
/// with a per-node report — a founding node that can't advertise its enode is
/// a real bootstrap failure, not something waiting longer fixes. A
/// transport/RPC error is retried; a *malformed* enode (fetched but
/// unparseable) fails fast, since retrying can't fix it.
pub async fn collect_enodes(
    targets: &[(String, String)],
    timeout: Duration,
    interval: Duration,
) -> anyhow::Result<BTreeMap<String, Bootnode>> {
    // Built up front: an unusable URL is a descriptor problem, reported before
    // anything is polled.
    let clients: Vec<(&str, rpc::Client)> = targets
        .iter()
        .map(|(label, url)| Ok((label.as_str(), rpc::Client::new(url)?)))
        .collect::<anyhow::Result<_>>()?;
    let mut enodes = BTreeMap::new();
    let mut last_error: BTreeMap<&str, String> = BTreeMap::new();
    let started = Instant::now();
    let deadline = started + timeout;
    let mut next_log = started;
    loop {
        for (label, client) in &clients {
            if enodes.contains_key(*label) {
                continue;
            }
            let raw = match fetch_enode(client).await {
                Ok(raw) => raw,
                Err(error) => {
                    last_error.insert(label, format!("{error:#}"));
                    continue;
                }
            };
            let enode = normalize_enode(&raw).with_context(|| label.to_string())?;
            println!("  ✓ {label}: enode readable");
            enodes.insert(label.to_string(), enode);
        }
        let pending: Vec<&str> = targets
            .iter()
            .map(|(label, _)| label.as_str())
            .filter(|label| !enodes.contains_key(*label))
            .collect();
        if pending.is_empty() {
            return Ok(enodes);
        }
        let now = Instant::now();
        if now >= deadline {
            let listing: Vec<String> = pending
                .iter()
                .map(|label| {
                    format!(
                        "  ✗ {label}: {}",
                        last_error
                            .get(label)
                            .map_or("never answered", String::as_str)
                    )
                })
                .collect();
            bail!(
                "{} node(s) never returned an enode via seismic_nodeInfo after {}s (reth not \
                 up?):\n{}",
                pending.len(),
                timeout.as_secs(),
                listing.join("\n")
            );
        }
        if now >= next_log {
            println!(
                "waiting for enodes ({}s elapsed, {}s until timeout): {}",
                now.duration_since(started).as_secs(),
                deadline.saturating_duration_since(now).as_secs(),
                pending.join(", ")
            );
            next_log = now + WAIT_LOG_INTERVAL;
        }
        tokio::time::sleep(interval).await;
    }
}

/// A key holder as a joining run vets it: where its attestation service
/// is appraised, and where its enode is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyHolder {
    pub name: String,
    pub public_ip: IpAddr,
    /// `http://<ip>:7878`, the endpoint appraised.
    pub attestation_rpc_url: String,
    /// `http://<ip>:8545`, where `seismic_nodeInfo` is read.
    pub eth_rpc_url: String,
}

impl KeyHolder {
    pub fn new(name: &str, descriptor: &NodeDescriptor) -> Self {
        Self {
            name: name.to_string(),
            public_ip: descriptor.public_ip,
            attestation_rpc_url: descriptor.attestation_rpc_url(),
            eth_rpc_url: descriptor.eth_rpc_url(),
        }
    }
}

/// The bootnodes of a run joining a live network: the enodes of the
/// `key_holders` that pass their appraisal.
///
/// Every key holder is appraised, `--no-verify` or not: that flag is about
/// the nodes configured. Refuses — nothing has been sent — when none passes.
pub async fn from_key_holders(
    key_holders: &[KeyHolder],
    manifest: &Manifest,
    appraisal: &Appraisal,
    remedy: &Remedy,
) -> anyhow::Result<Vec<Bootnode>> {
    let names: Vec<&str> = key_holders
        .iter()
        .map(|holder| holder.name.as_str())
        .collect();
    eprintln!(
        "{} key holder(s): {}; deploy-verifying each before handing out an enode...",
        key_holders.len(),
        names.join(", ")
    );
    let mut tasks = JoinSet::new();
    for holder in key_holders {
        let holder = holder.clone();
        let manifest = manifest.clone();
        let appraisal = appraisal.clone();
        tasks.spawn(async move {
            let verdict = vet(&holder, &manifest, &appraisal).await;
            (holder.name, verdict)
        });
    }
    let mut vetted = BTreeMap::new();
    while let Some(joined) = tasks.join_next().await {
        let (name, verdict) = joined.expect("a vetting task never panics");
        vetted.insert(name, verdict);
    }
    for (name, verdict) in &vetted {
        match verdict {
            Ok(enode) => eprintln!("  ✓ {name}: {enode}"),
            Err(reason) => eprintln!("  ✗ {name}: {reason}"),
        }
    }
    handed_out(&vetted, remedy)
}

/// Why a key holder's enode is not handed out. Each calls for its own
/// remedy: re-founding, waiting, or a peer named by hand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unvetted {
    /// Its quote did not pass against the manifest's policy: it must not
    /// serve root_key to anyone.
    FailedAppraisal(String),
    /// It passed, but serves no enode yet: reth is not up (its disk wipe, or
    /// its start).
    NoEnodeYet(String),
    /// It passed, but its enode cannot be handed out — malformed, or at
    /// another host than the one appraised — and a re-run reads the same.
    UnusableEnode(String),
}

impl std::fmt::Display for Unvetted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedAppraisal(error) => write!(f, "deploy verification FAILED: {error}"),
            Self::NoEnodeYet(reason) | Self::UnusableEnode(reason) => f.write_str(reason),
        }
    }
}

/// Appraise one key holder, then read its enode. The verdict is the enode to
/// hand out, or why not.
pub async fn vet(
    holder: &KeyHolder,
    manifest: &Manifest,
    appraisal: &Appraisal,
) -> Result<Bootnode, Unvetted> {
    challenge_node(
        &holder.attestation_rpc_url,
        manifest,
        &appraisal.policy,
        appraisal.pccs_url.as_deref(),
    )
    .await
    .map_err(|error| Unvetted::FailedAppraisal(format!("{error:#}")))?;
    read_enode(holder).await
}

/// A key holder's enode, normalized, or why it cannot be handed out.
///
/// The enode's host must be the IP whose attestation service was appraised:
/// tdx-init fetches `root_key` from the enode's host, so an enode elsewhere
/// would send the joiner to a box nobody appraised.
pub async fn read_enode(holder: &KeyHolder) -> Result<Bootnode, Unvetted> {
    let client = rpc::Client::new(&holder.eth_rpc_url)
        .map_err(|error| Unvetted::UnusableEnode(format!("{error:#}")))?;
    let raw = fetch_enode(&client)
        .await
        .map_err(|error| Unvetted::NoEnodeYet(format!("no enode: {error:#}")))?;
    let enode =
        normalize_enode(&raw).map_err(|error| Unvetted::UnusableEnode(format!("{error:#}")))?;
    if !is_at_ip(&enode, holder.public_ip) {
        return Err(Unvetted::UnusableEnode(format!(
            "its enode {enode} is not at its public_ip {}: the joining node would fetch root_key \
             from a box other than the one appraised",
            holder.public_ip
        )));
    }
    Ok(enode)
}

/// The enodes that passed, or the refusal that none did, naming each key
/// holder's reason. One that passed but serves no enode yet is waited for;
/// re-founding is the way on only when every key holder failed its
/// appraisal; otherwise a re-run reads the same enodes, and only a peer named
/// by hand is left.
pub fn handed_out(
    vetted: &BTreeMap<String, Result<Bootnode, Unvetted>>,
    remedy: &Remedy,
) -> anyhow::Result<Vec<Bootnode>> {
    let enodes: Vec<Bootnode> = vetted.values().filter_map(|v| v.clone().ok()).collect();
    if !enodes.is_empty() {
        return Ok(enodes);
    }
    let reasons: Vec<String> = vetted
        .iter()
        .filter_map(|(name, verdict)| Some(format!("  ✗ {name}: {}", verdict.as_ref().err()?)))
        .collect();
    let starting = vetted
        .values()
        .any(|verdict| matches!(verdict, Err(Unvetted::NoEnodeYet(_))));
    let failed_appraisal = vetted
        .values()
        .all(|verdict| matches!(verdict, Err(Unvetted::FailedAppraisal(_))));
    let instead = if starting {
        format!(
            "A key holder that passed its appraisal but serves no enode yet is still starting: \
             watch it (`{}`) and re-run once it serves one; or name a live peer",
            remedy.watch()
        )
    } else if failed_appraisal {
        format!(
            "A key holder that fails its appraisal must not serve root_key to anyone: {}; or name \
             a live peer",
            remedy.re_found()
        )
    } else {
        "A re-run reads the same enodes: name a live peer instead".to_string()
    };
    bail!(
        "no key holder in the node table can be handed out as a bootnode:\n{}\nNothing was \
         sent. {instead}:\n    {}",
        reasons.join("\n"),
        remedy.bootnode()
    )
}

#[cfg(test)]
mod tests {
    use seismic_tee_common::test_support::{FIXTURE_MANIFEST, FakeServer, refused_url, rpc_result};
    use serde_json::json;

    use super::*;

    fn id() -> String {
        "ab".repeat(64)
    }

    #[test]
    fn normalize_keeps_canonical_drops_discport_and_keeps_ipv6_brackets() {
        let normalized = |enode: &str| normalize_enode(enode).unwrap().to_string();
        let canonical = format!("enode://{}@1.2.3.4:30303", id());
        assert_eq!(normalized(&canonical), canonical);
        assert_eq!(
            normalized(&format!("{canonical}?discport=30304")),
            canonical
        );
        let ipv6 = format!("enode://{}@[2001:db8::1]:30303", id());
        assert_eq!(normalized(&ipv6), ipv6);
    }

    #[test]
    fn an_enode_is_at_its_ip_by_value() {
        let ip = |ip: &str| ip.parse().unwrap();
        let ipv6 = normalize_enode(&format!("enode://{}@[2001:0db8::1]:30303", id())).unwrap();
        assert!(is_at_ip(&ipv6, ip("2001:db8:0:0:0:0:0:1")));
        let ipv4 = normalize_enode(&format!("enode://{}@1.2.3.4:30303", id())).unwrap();
        assert!(is_at_ip(&ipv4, ip("1.2.3.4")));
        assert!(!is_at_ip(&ipv4, ip("1.2.3.5")));
        let named = normalize_enode(&format!("enode://{}@node1.example.com:30303", id())).unwrap();
        assert!(!is_at_ip(&named, ip("1.2.3.4")));
    }

    #[test]
    fn malformed_enodes_are_rejected_by_reason() {
        for (enode, reason) in [
            (
                format!("enode://{}@1.2.3.4:30303", "ab".repeat(8)),
                "128 hex",
            ),
            (format!("http://{}@1.2.3.4:30303", id()), "enode://"),
            (format!("enode://{}@1.2.3.4", id()), ":port"),
            (
                format!("enode://{}@1.2.3.4:99999", id()),
                "non-numeric port",
            ),
            (format!("enode://{}@:30303", id()), "empty host"),
            (format!("enode://{}", id()), "'@'"),
            (
                format!("enode://{}@1.2.3.4 --http.api admin:30303", id()),
                "bracketed IPv6",
            ),
        ] {
            let err = normalize_enode(&enode).unwrap_err().to_string();
            assert!(err.contains(reason), "{enode}: {err}");
        }
    }

    #[tokio::test]
    async fn collect_returns_normalized_enodes_keyed_by_label() {
        let n1 = FakeServer::serve(vec![(
            200,
            rpc_result(json!({"enode": format!("enode://{}@1.1.1.1:30303?discport=30304", id())})),
        )]);
        let n2 = FakeServer::serve(vec![(
            200,
            rpc_result(json!({"enode": format!("enode://{}@2.2.2.2:30303", "cd".repeat(64))})),
        )]);
        let enodes = collect_enodes(
            &[
                ("node-1".to_string(), n1.url.clone()),
                ("node-2".to_string(), n2.url.clone()),
            ],
            Duration::from_secs(10),
            Duration::from_millis(20),
        )
        .await
        .unwrap();
        assert_eq!(
            enodes["node-1"].to_string(),
            format!("enode://{}@1.1.1.1:30303", id())
        );
        assert_eq!(
            enodes["node-2"].to_string(),
            format!("enode://{}@2.2.2.2:30303", "cd".repeat(64))
        );
        // The request is the JSON-RPC call, through the shared client.
        let body: serde_json::Value = serde_json::from_slice(&n1.requests()[0].body).unwrap();
        assert_eq!(body["method"], "seismic_nodeInfo");
    }

    #[tokio::test]
    async fn collect_retries_a_still_booting_node_and_lists_the_stuck() {
        let late = FakeServer::serve_after(
            Duration::from_millis(200),
            vec![(
                200,
                rpc_result(json!({"enode": format!("enode://{}@1.1.1.1:30303", id())})),
            )],
        );
        let enodes = collect_enodes(
            &[("node-1".to_string(), late.url.clone())],
            Duration::from_secs(10),
            Duration::from_millis(20),
        )
        .await
        .unwrap();
        assert_eq!(enodes.len(), 1);

        let err = collect_enodes(
            &[("node-1".to_string(), refused_url())],
            Duration::from_millis(100),
            Duration::from_millis(20),
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(err.contains("1 node(s) never returned an enode"), "{err}");
        assert!(err.contains("✗ node-1"), "{err}");
    }

    #[tokio::test]
    async fn a_malformed_enode_fails_fast() {
        let server = FakeServer::serve(vec![
            (
                200,
                rpc_result(json!({"enode": "enode://short@1.1.1.1:30303"})),
            ),
            (200, rpc_result(json!({"enode": "never reached"}))),
        ]);
        let err = format!(
            "{:?}",
            collect_enodes(
                &[("node-1".to_string(), server.url.clone())],
                Duration::from_secs(10),
                Duration::from_millis(20),
            )
            .await
            .unwrap_err()
        );
        assert!(err.contains("node-1"), "{err}");
        assert!(err.contains("128 hex"), "{err}");
        assert_eq!(server.requests().len(), 1);
    }

    /// Every way a node can fail to advertise is an error naming the
    /// endpoint, for the retry loop and its deadline listing.
    #[tokio::test]
    async fn a_response_without_an_enode_is_retried_as_not_up() {
        let server = FakeServer::serve(vec![(200, rpc_result(json!({})))]);
        let err = fetch_enode(&rpc::Client::new(&server.url).unwrap())
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("returned no enode"), "{err}");
        assert!(err.contains(&server.url), "{err}");

        let server = FakeServer::serve(vec![(
            200,
            r#"{"jsonrpc": "2.0", "id": 1, "error": {"code": -32601, "message": "no such method"}}"#
                .to_string(),
        )]);
        let err = fetch_enode(&rpc::Client::new(&server.url).unwrap())
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("no such method"), "{err}");

        let url = refused_url();
        let err = fetch_enode(&rpc::Client::new(&url).unwrap())
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains(&url), "{err}");
    }

    fn remedy() -> Remedy {
        Remedy {
            table: " --node nodes.json".to_string(),
            names: " gamma".to_string(),
            manifest: " --manifest net/network-manifest.json".to_string(),
            dir: " net".to_string(),
            appraisal: String::new(),
        }
    }

    fn key_holder(public_ip: &str, eth_rpc_url: &str) -> KeyHolder {
        KeyHolder {
            name: "alpha".to_string(),
            public_ip: public_ip.parse().unwrap(),
            attestation_rpc_url: refused_url(),
            eth_rpc_url: eth_rpc_url.to_string(),
        }
    }

    fn manifest() -> Manifest {
        Manifest::from_json_bytes(FIXTURE_MANIFEST).unwrap()
    }

    /// A key holder's enode is read normalized, and only at the IP its
    /// appraisal was of.
    #[tokio::test]
    async fn a_key_holder_is_handed_out_only_at_its_own_ip() {
        let enode =
            |host: &str| json!({"enode": format!("enode://{}@{host}:30303?discport=30304", id())});
        let server = FakeServer::serve(vec![(200, rpc_result(enode("1.1.1.1")))]);
        assert_eq!(
            read_enode(&key_holder("1.1.1.1", &server.url))
                .await
                .unwrap()
                .to_string(),
            format!("enode://{}@1.1.1.1:30303", id())
        );

        // Elsewhere, or malformed: a re-run reads the same.
        let server = FakeServer::serve(vec![(200, rpc_result(enode("10.0.0.4")))]);
        let reason = read_enode(&key_holder("1.1.1.1", &server.url))
            .await
            .unwrap_err();
        assert!(matches!(reason, Unvetted::UnusableEnode(_)), "{reason:?}");
        assert!(
            reason
                .to_string()
                .contains("is not at its public_ip 1.1.1.1"),
            "{reason}"
        );
        let server = FakeServer::serve(vec![(
            200,
            rpc_result(json!({"enode": "enode://short@1.1.1.1:30303"})),
        )]);
        let reason = read_enode(&key_holder("1.1.1.1", &server.url))
            .await
            .unwrap_err();
        assert!(matches!(reason, Unvetted::UnusableEnode(_)), "{reason:?}");

        // No answer yet: reth is still starting.
        let reason = read_enode(&key_holder("1.1.1.1", &refused_url()))
            .await
            .unwrap_err();
        assert!(matches!(reason, Unvetted::NoEnodeYet(_)), "{reason:?}");
        assert!(reason.to_string().starts_with("no enode: "), "{reason}");
    }

    /// The appraisal comes first: a key holder that does not pass is never
    /// asked for its enode.
    #[tokio::test]
    async fn a_key_holder_that_fails_its_appraisal_is_not_handed_out() {
        let server = FakeServer::serve(vec![(
            200,
            rpc_result(json!({"enode": format!("enode://{}@1.1.1.1:30303", id())})),
        )]);
        let appraisal = Appraisal {
            policy: b"{ not a policy".to_vec(),
            pccs_url: None,
        };
        let reason = vet(&key_holder("1.1.1.1", &server.url), &manifest(), &appraisal)
            .await
            .unwrap_err();
        assert!(matches!(reason, Unvetted::FailedAppraisal(_)), "{reason:?}");
        assert!(
            reason.to_string().starts_with("deploy verification FAILED"),
            "{reason}"
        );
        assert!(server.requests().is_empty());
    }

    /// Only the key holders that passed are handed out; with none, the
    /// refusal names each one's reason and the way on.
    #[test]
    fn only_the_passing_key_holders_are_handed_out() {
        let enode: Bootnode = format!("enode://{}@2.2.2.2:30303", id()).parse().unwrap();
        let failed = || Err(Unvetted::FailedAppraisal("x".to_string()));
        let vetted = BTreeMap::from([
            ("alpha".to_string(), failed()),
            ("beta".to_string(), Ok(enode.clone())),
        ]);
        assert_eq!(handed_out(&vetted, &remedy()).unwrap(), [enode]);

        let vetted = BTreeMap::from([("alpha".to_string(), failed())]);
        let err = handed_out(&vetted, &remedy()).unwrap_err().to_string();
        assert!(
            err.contains("  ✗ alpha: deploy verification FAILED: x"),
            "{err}"
        );
        assert!(err.contains("Nothing was sent"), "{err}");
        assert!(
            err.contains("seismic-tee node harvest net --node nodes.json --force"),
            "{err}"
        );
        assert!(
            err.contains("seismic-tee node configure --node nodes.json gamma --bootnode <ENODE>"),
            "{err}"
        );
    }

    /// A key holder that passed but serves no enode yet — reth still behind
    /// its disk wipe — is waited for, not a reason to re-found.
    #[test]
    fn a_key_holder_still_starting_is_waited_for() {
        let vetted = BTreeMap::from([
            (
                "alpha".to_string(),
                Err(Unvetted::FailedAppraisal("x".to_string())),
            ),
            (
                "beta".to_string(),
                Err(Unvetted::NoEnodeYet(
                    "no enode: connection refused".to_string(),
                )),
            ),
        ]);
        let err = handed_out(&vetted, &remedy()).unwrap_err().to_string();
        assert!(
            err.contains("  ✗ beta: no enode: connection refused"),
            "{err}"
        );
        assert!(err.contains("still starting"), "{err}");
        assert!(
            err.contains(
                "seismic-tee node status --watch --node nodes.json --manifest \
                 net/network-manifest.json"
            ),
            "{err}"
        );
        assert!(!err.contains("re-found"), "{err}");
        assert!(err.contains("--bootnode <ENODE>"), "{err}");
    }

    /// A key holder that passed but advertises an enode that cannot be
    /// handed out is neither waited for nor a reason to re-found: a re-run
    /// reads the same, so only a named peer is left.
    #[test]
    fn an_unusable_enode_leaves_only_a_named_peer() {
        let vetted = BTreeMap::from([
            (
                "alpha".to_string(),
                Err(Unvetted::FailedAppraisal("x".to_string())),
            ),
            (
                "beta".to_string(),
                Err(Unvetted::UnusableEnode(
                    "its enode enode://… is not at its public_ip 2.2.2.2".to_string(),
                )),
            ),
        ]);
        let err = handed_out(&vetted, &remedy()).unwrap_err().to_string();
        assert!(err.contains("  ✗ beta: its enode"), "{err}");
        assert!(err.contains("A re-run reads the same enodes"), "{err}");
        assert!(!err.contains("still starting"), "{err}");
        assert!(!err.contains("re-found"), "{err}");
        assert!(
            err.contains("seismic-tee node configure --node nodes.json gamma --bootnode <ENODE>"),
            "{err}"
        );
    }
}
