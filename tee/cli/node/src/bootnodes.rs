//! Bootnodes: where a node's `[network].bootnodes` come from.
//!
//! tdx-init derives a node's `root_key` fetch peers from its bootnodes
//! (`http://<host>:7878`), and reth dials them, so a bootnode is a box the
//! node is about to trust with its key. Two sources:
//!
//! - **Founding.** A freshly-configured reth node's enode (its devp2p node
//!   record) isn't known until reth is up, so a greenfield cohort can't be
//!   handed a static bootnode list up front: the genesis node comes up first,
//!   then its enode seeds the joiners (the two-stage bootstrap in
//!   [`crate::cohort`]), polled with [`collect_enodes`].
//! - **Joining.** Every other config POST — a node joining after the
//!   founding, a founding node rebooted — goes to a live network, whose key
//!   holders (nodes serving `:7878`) are found by the [`crate::probe`] of the
//!   node table ([`from_key_holders`]). Nothing is remembered between runs,
//!   so each run re-applies the founding's rule that a node's enode reaches
//!   others only after it passes its appraisal: every key holder is
//!   deploy-verified, and only those that pass are handed out. With none,
//!   the run refuses before anything is sent.
//!
//! `seismic_nodeInfo` is read over the node's public JSON-RPC
//! (`https://<fqdn>/rpc`, the HTTPS proxy in front of reth) with the same client the
//! attestation service is read with ([`rpc::Client`]); the `seismic` namespace
//! deliberately keeps nodeInfo public (reth's `admin` namespace is disabled on
//! these nodes).
//!
//! reth's NodeRecord Display appends `?discport=<udp_port>` when its devp2p
//! UDP port differs from its TCP port; tdx-init rejects that form at POST time
//! (the `[network].bootnodes` grammar is strictly `enode://<128 hex>@host:port`).
//! So every fetched enode is normalized ([`normalize_enode`]) — the query
//! dropped, bracketed IPv6 preserved — before it is delivered.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use jsonrpsee::rpc_params;
use seismic_tee_common::{Descriptors, Manifest, NodeDescriptor, rpc};
use tokio::task::JoinSet;

use crate::cohort::Appraisal;
use crate::probe::{self, Asked, Founding, NetworkState, NodeState, assess, probe_all};
use crate::status::network_line;
use crate::verify::challenge_node;

/// Enode-readiness polling. reth serves `seismic_nodeInfo` only once it is up
/// (root_key → LUKS open → reth start), so an early miss is the normal boot
/// tail — polled until timeout.
pub const POLL_INTERVAL: Duration = Duration::from_secs(5);
pub const ENODE_TIMEOUT: Duration = Duration::from_secs(15 * 60);
pub const WAIT_LOG_INTERVAL: Duration = Duration::from_secs(30);

/// The pieces of an enode URL, as tdx-init's grammar reads them.
struct Enode<'a> {
    id: &'a str,
    /// Unbracketed.
    host: &'a str,
    port: u16,
    query: Option<&'a str>,
}

fn parse_enode(enode: &str) -> anyhow::Result<Enode<'_>> {
    let Some(rest) = enode.strip_prefix("enode://") else {
        bail!("not an enode URL: {enode:?}");
    };
    let Some((id, rest)) = rest.split_once('@') else {
        bail!("enode has no host: {enode:?}");
    };
    if id.len() != 128 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("enode node id is not 128 hex chars: {enode:?}");
    }
    let (hostport, query) = match rest.split_once('?') {
        Some((hostport, query)) => (hostport, Some(query)),
        None => (rest, None),
    };
    let (host, port) = if let Some(after_bracket) = hostport.strip_prefix('[') {
        let Some((host, port)) = after_bracket.split_once(']') else {
            bail!("enode has no host: {enode:?}");
        };
        let Some(port) = port.strip_prefix(':') else {
            bail!("enode has no port: {enode:?}");
        };
        (host, port)
    } else {
        let Some((host, port)) = hostport.rsplit_once(':') else {
            bail!("enode has no port: {enode:?}");
        };
        (host, port)
    };
    if host.is_empty() {
        bail!("enode has no host: {enode:?}");
    }
    let port: u16 = port
        .parse()
        .map_err(|_| anyhow::anyhow!("enode port is not a valid u16: {enode:?}"))?;
    Ok(Enode {
        id,
        host,
        port,
        query,
    })
}

/// Coerce a NodeRecord enode into the strict `enode://<128 hex>@host:port` form
/// tdx-init accepts, dropping any `?discport=` query reth appends when its
/// devp2p TCP and UDP ports differ (seismic nodes run them equal, so this is
/// normally a no-op). Bracketed IPv6 is preserved. Fails if the record isn't a
/// well-formed enode — a founding node advertising a malformed record is a
/// hard error, not something to deliver verbatim.
pub fn normalize_enode(enode: &str) -> anyhow::Result<String> {
    let parsed = parse_enode(enode)?;
    if let Some(query) = parsed.query {
        eprintln!(
            "warning: enode {enode} carries a query string ({query}); dropping it — tdx-init \
             accepts only enode://<id>@host:port. seismic nodes run matched devp2p TCP/UDP ports, \
             so this is normally absent."
        );
    }
    let host = if parsed.host.contains(':') {
        format!("[{}]", parsed.host)
    } else {
        parsed.host.to_string()
    };
    Ok(format!("enode://{}@{host}:{}", parsed.id, parsed.port))
}

/// Host part of an enode URL (unbracketed), or `None` if it can't be parsed.
pub fn enode_host(enode: &str) -> Option<String> {
    parse_enode(enode).ok().map(|e| e.host.to_string())
}

/// Warn (never fail) if an enode's host isn't the node's public IP.
///
/// With reth's `--nat extip <public_ip>` (from `[node].external_ip`) the
/// advertised enode host should equal the descriptor's public_ip. A mismatch
/// usually means extip wasn't applied, so reth advertised a private or
/// auto-detected address peers can't dial — worth surfacing loudly, but not
/// fatal to configuration.
pub fn warn_on_ip_mismatch(enode: &str, expected_ip: &str, label: &str) {
    let host = enode_host(enode);
    if host.as_deref() != Some(expected_ip) {
        eprintln!(
            "warning: {label}: enode host {} does not match descriptor public_ip {expected_ip} — \
             is reth's --nat extip set to the public IP? peers may be unable to reach this node's \
             enode.",
            host.unwrap_or_else(|| "?".to_string())
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
) -> anyhow::Result<BTreeMap<String, String>> {
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

/// What a refusal suggests instead, spelled with this run's flags so it
/// pastes. Each field is empty or leading-space-prefixed.
#[derive(Debug, Clone, Default)]
pub struct Remedy {
    /// The flags naming the node table, without `--name`.
    pub table: String,
    /// ` --manifest <FILE>`, when one was given.
    pub manifest: String,
    /// The network directory, for the re-found harvest, when `--manifest`
    /// named one; the context supplies it otherwise.
    pub dir: String,
}

impl Remedy {
    pub(crate) fn bootnode(&self, joining: &str) -> String {
        format!(
            "seismic-tee node configure{} --name {joining} --bootnode <ENODE>{}",
            self.table, self.manifest
        )
    }

    fn re_found(&self) -> String {
        format!(
            "re-found: `pulumi destroy`, a fresh `pulumi up`, then `seismic-tee node harvest{}{} \
             --force`",
            self.dir, self.table
        )
    }
}

/// A key holder as the joining run vets it: where its attestation service
/// is appraised, and where its enode is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyHolder {
    pub name: String,
    pub public_ip: String,
    /// `http://<ip>:7878`, the endpoint appraised.
    pub attestation_rpc_url: String,
    /// `https://<fqdn>/rpc`, where `seismic_nodeInfo` is read.
    pub eth_rpc_url: String,
}

impl KeyHolder {
    pub fn new(name: &str, descriptor: &NodeDescriptor) -> Self {
        Self {
            name: name.to_string(),
            public_ip: descriptor.public_ip.clone(),
            attestation_rpc_url: descriptor.attestation_rpc_url(),
            eth_rpc_url: descriptor.eth_rpc_url(),
        }
    }
}

/// The bootnodes for `joining`, a node in `table` about to take its config:
/// the enodes of the table's other key holders that pass their appraisal.
///
/// Every node in the table is probed, `joining` among them: the network's
/// state is read from all of them, as `node status` reads it, and a refusal
/// is worded by it. Every key holder is appraised, `--no-verify` or not: that
/// flag is about `joining`. Refuses — nothing has been sent — when `joining`
/// already holds `root_key`, when no other node does, or when none that does
/// passes.
pub async fn from_key_holders(
    client: &reqwest::Client,
    table: &Descriptors,
    joining: &str,
    founding: Option<&Founding>,
    manifest: &Manifest,
    appraisal: &Appraisal,
    remedy: &Remedy,
) -> anyhow::Result<Vec<String>> {
    eprintln!(
        "Finding bootnodes: reading the {} node(s) in the node table...",
        table.len()
    );
    let states = probe_all(client, &probe::targets(table)).await;
    let holders = key_holders(&states, joining, founding, remedy)?;
    eprintln!(
        "{} key holder(s): {}; deploy-verifying each before handing out an enode...",
        holders.len(),
        holders.join(", ")
    );
    let mut tasks = JoinSet::new();
    for name in holders {
        let holder = KeyHolder::new(&name, &table[&name]);
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
    handed_out(&vetted, joining, remedy)
}

/// The key holders among `states` other than `joining`, in name order, or the
/// refusal that there are none.
pub fn key_holders(
    states: &BTreeMap<String, NodeState>,
    joining: &str,
    founding: Option<&Founding>,
    remedy: &Remedy,
) -> anyhow::Result<Vec<String>> {
    if states
        .get(joining)
        .is_some_and(|state| state.key_holder.is_some())
    {
        bail!(
            "{joining} already holds root_key: it took its config this boot, and tdx-init takes \
             one per boot. Nothing was sent. To appraise it:\n    seismic-tee node verify{} \
             --name {joining}{}",
            remedy.table,
            remedy.manifest
        );
    }
    let holders: Vec<String> = states
        .iter()
        .filter(|(name, state)| *name != joining && state.key_holder.is_some())
        .map(|(name, _)| name.clone())
        .collect();
    if !holders.is_empty() {
        return Ok(holders);
    }
    let network = assess(states, founding);
    let instead = match &network {
        NetworkState::Unfounded { pinned_box } => format!(
            "found it first:\n    seismic-tee node configure{} --genesis-node {pinned_box}{}",
            remedy.table, remedy.manifest
        ),
        NetworkState::NoReachableKeyHolder(Asked::EveryFounder {
            pinned_box_answered: true,
            ..
        }) => format!(
            "{}; or name a live peer:\n    {}",
            remedy.re_found(),
            remedy.bootnode(joining)
        ),
        NetworkState::NoReachableKeyHolder(Asked::EveryFounder {
            pinned_box_answered: false,
            ..
        }) => format!(
            "watch for it to come up (`seismic-tee node status --watch{}{}`), or name a live \
             peer:\n    {}",
            remedy.table,
            remedy.manifest,
            remedy.bootnode(joining)
        ),
        NetworkState::NoReachableKeyHolder(Asked::Some) | NetworkState::Live { .. } => format!(
            "name a live peer by its enode (its `seismic_nodeInfo`):\n    {}",
            remedy.bootnode(joining)
        ),
    };
    bail!(
        "{}\nNothing was sent: {joining} would have no peer to fetch root_key from. Instead, \
         {instead}",
        network_line(&network)
    )
}

/// Appraise one key holder, then read its enode. The verdict is the enode to
/// hand out, or why not.
pub async fn vet(
    holder: &KeyHolder,
    manifest: &Manifest,
    appraisal: &Appraisal,
) -> Result<String, String> {
    challenge_node(
        &holder.attestation_rpc_url,
        manifest,
        &appraisal.policy,
        appraisal.pccs_url.as_deref(),
    )
    .await
    .map_err(|error| format!("deploy verification FAILED: {error:#}"))?;
    read_enode(holder).await
}

/// A key holder's enode, normalized, or why it cannot be handed out.
///
/// The enode's host must be the IP whose attestation service was appraised:
/// tdx-init fetches `root_key` from the enode's host, so an enode elsewhere
/// would send the joiner to a box nobody appraised.
pub async fn read_enode(holder: &KeyHolder) -> Result<String, String> {
    let client = rpc::Client::new(&holder.eth_rpc_url).map_err(|error| format!("{error:#}"))?;
    let raw = fetch_enode(&client)
        .await
        .map_err(|error| format!("no enode: {error:#}"))?;
    let enode = normalize_enode(&raw).map_err(|error| format!("{error:#}"))?;
    if enode_host(&enode).as_deref() != Some(holder.public_ip.as_str()) {
        return Err(format!(
            "its enode {enode} is not at its public_ip {}: the joining node would fetch root_key \
             from a box other than the one appraised",
            holder.public_ip
        ));
    }
    Ok(enode)
}

/// The enodes that passed, or the refusal that none did, naming each key
/// holder's reason.
pub fn handed_out(
    vetted: &BTreeMap<String, Result<String, String>>,
    joining: &str,
    remedy: &Remedy,
) -> anyhow::Result<Vec<String>> {
    let enodes: Vec<String> = vetted.values().filter_map(|v| v.clone().ok()).collect();
    if !enodes.is_empty() {
        return Ok(enodes);
    }
    let reasons: Vec<String> = vetted
        .iter()
        .map(|(name, verdict)| {
            format!(
                "  ✗ {name}: {}",
                verdict.as_ref().err().map_or("", String::as_str)
            )
        })
        .collect();
    bail!(
        "no key holder in the node table can be handed to {joining} as a bootnode:\n{}\nNothing \
         was sent. A key holder that fails its appraisal must not serve root_key to anyone: \
         {}; or name a live peer:\n    {}",
        reasons.join("\n"),
        remedy.re_found(),
        remedy.bootnode(joining)
    )
}

#[cfg(test)]
mod tests {
    use seismic_tee_common::test_support::{
        FIXTURE_FOUNDING_TX_IO_PK, FIXTURE_MANIFEST, FakeServer, candidate_tx_io_public_key,
        refused_url, rpc_result,
    };
    use serde_json::json;

    use super::*;
    use crate::probe::{ConfigState, Disk};

    fn id() -> String {
        "ab".repeat(64)
    }

    #[test]
    fn normalize_keeps_canonical_drops_discport_and_keeps_ipv6_brackets() {
        let canonical = format!("enode://{}@1.2.3.4:30303", id());
        assert_eq!(normalize_enode(&canonical).unwrap(), canonical);
        assert_eq!(
            normalize_enode(&format!("{canonical}?discport=30304")).unwrap(),
            canonical
        );
        let ipv6 = format!("enode://{}@[2001:db8::1]:30303", id());
        assert_eq!(normalize_enode(&ipv6).unwrap(), ipv6);
        assert_eq!(enode_host(&ipv6).unwrap(), "2001:db8::1");
        assert_eq!(enode_host(&canonical).unwrap(), "1.2.3.4");
        assert_eq!(enode_host("nonsense"), None);
    }

    #[test]
    fn malformed_enodes_are_rejected_by_reason() {
        for (enode, reason) in [
            (
                format!("enode://{}@1.2.3.4:30303", "ab".repeat(8)),
                "128 hex",
            ),
            (format!("http://{}@1.2.3.4:30303", id()), "not an enode"),
            (format!("enode://{}@1.2.3.4", id()), "no port"),
            (format!("enode://{}@1.2.3.4:99999", id()), "valid u16"),
            (format!("enode://{}@:30303", id()), "no host"),
            (format!("enode://{}", id()), "no host"),
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
        assert_eq!(enodes["node-1"], format!("enode://{}@1.1.1.1:30303", id()));
        assert_eq!(
            enodes["node-2"],
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
            manifest: " --manifest net/network-manifest.json".to_string(),
            dir: " net".to_string(),
        }
    }

    fn holder() -> NodeState {
        NodeState {
            config: ConfigState::Configured,
            key_holder: Some(Disk::Status(json!({"state": "idle"}))),
        }
    }

    fn awaiting(candidate: &str) -> NodeState {
        NodeState {
            config: ConfigState::Awaiting {
                node_public_key: "ab".repeat(32),
                candidate_tx_io_public_key: candidate.to_string(),
            },
            key_holder: None,
        }
    }

    fn states(entries: &[(&str, NodeState)]) -> BTreeMap<String, NodeState> {
        entries
            .iter()
            .map(|(name, state)| (name.to_string(), state.clone()))
            .collect()
    }

    /// alpha harvested the pin; beta is the other founder.
    fn founding() -> Founding {
        Founding {
            pin: FIXTURE_FOUNDING_TX_IO_PK.to_string(),
            pinned_box: Some("alpha".to_string()),
            founders: ["alpha", "beta"].map(String::from).into(),
        }
    }

    /// Every other key holder is a candidate, in name order; the joining
    /// node is never its own bootnode.
    #[test]
    fn the_other_key_holders_are_the_candidates() {
        let table = states(&[
            ("gamma", holder()),
            ("alpha", holder()),
            ("beta", awaiting(&candidate_tx_io_public_key("bb"))),
        ]);
        assert_eq!(
            key_holders(&table, "beta", Some(&founding()), &remedy()).unwrap(),
            ["alpha", "gamma"]
        );
    }

    /// A joining node that serves :7878 took its config this boot: there is
    /// nothing to send it.
    #[test]
    fn a_joining_node_that_holds_the_key_is_refused() {
        let table = states(&[("alpha", holder()), ("beta", holder())]);
        let err = key_holders(&table, "beta", None, &remedy())
            .unwrap_err()
            .to_string();
        assert!(err.contains("beta already holds root_key"), "{err}");
        assert!(
            err.contains(
                "seismic-tee node verify --node nodes.json --name beta --manifest \
                 net/network-manifest.json"
            ),
            "{err}"
        );
    }

    /// No key holder: the refusal reads the network as `node status` does,
    /// and names the command for that state.
    #[test]
    fn no_key_holder_is_refused_by_the_network_state() {
        let unfounded = states(&[
            ("alpha", awaiting(FIXTURE_FOUNDING_TX_IO_PK)),
            ("beta", awaiting(&candidate_tx_io_public_key("bb"))),
        ]);
        let err = key_holders(&unfounded, "beta", Some(&founding()), &remedy())
            .unwrap_err()
            .to_string();
        assert!(err.contains("network: unfounded"), "{err}");
        assert!(
            err.contains(
                "seismic-tee node configure --node nodes.json --genesis-node alpha --manifest \
                 net/network-manifest.json"
            ),
            "{err}"
        );

        // The pinned box re-minted: the founding root_key is gone.
        let lost = states(&[
            ("alpha", awaiting(&candidate_tx_io_public_key("aa"))),
            ("beta", awaiting(&candidate_tx_io_public_key("bb"))),
        ]);
        let err = key_holders(&lost, "beta", Some(&founding()), &remedy())
            .unwrap_err()
            .to_string();
        assert!(err.contains("the founding root_key is gone"), "{err}");
        assert!(
            err.contains("seismic-tee node harvest net --node nodes.json --force"),
            "{err}"
        );
        assert!(
            err.contains(
                "seismic-tee node configure --node nodes.json --name beta --bootnode <ENODE> \
                 --manifest net/network-manifest.json"
            ),
            "{err}"
        );

        // An operator's own table says nothing about the network.
        let own = states(&[("operator-1", awaiting(&candidate_tx_io_public_key("cc")))]);
        let err = key_holders(&own, "operator-1", Some(&founding()), &remedy())
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("no reachable key holder among these nodes"),
            "{err}"
        );
        assert!(
            err.contains("--name operator-1 --bootnode <ENODE>"),
            "{err}"
        );
        assert!(!err.contains("re-found"), "{err}");
    }

    fn key_holder(public_ip: &str, eth_rpc_url: &str) -> KeyHolder {
        KeyHolder {
            name: "alpha".to_string(),
            public_ip: public_ip.to_string(),
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
            read_enode(&key_holder("1.1.1.1", &server.url)).await,
            Ok(format!("enode://{}@1.1.1.1:30303", id()))
        );

        let server = FakeServer::serve(vec![(200, rpc_result(enode("10.0.0.4")))]);
        let reason = read_enode(&key_holder("1.1.1.1", &server.url))
            .await
            .unwrap_err();
        assert!(
            reason.contains("is not at its public_ip 1.1.1.1"),
            "{reason}"
        );

        let reason = read_enode(&key_holder("1.1.1.1", &refused_url()))
            .await
            .unwrap_err();
        assert!(reason.starts_with("no enode: "), "{reason}");
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
        assert!(reason.starts_with("deploy verification FAILED"), "{reason}");
        assert!(server.requests().is_empty());
    }

    /// Only the key holders that passed are handed out; with none, the
    /// refusal names each one's reason and the way on.
    #[test]
    fn only_the_passing_key_holders_are_handed_out() {
        let enode = format!("enode://{}@2.2.2.2:30303", id());
        let vetted = BTreeMap::from([
            (
                "alpha".to_string(),
                Err("deploy verification FAILED: x".to_string()),
            ),
            ("beta".to_string(), Ok(enode.clone())),
        ]);
        assert_eq!(handed_out(&vetted, "gamma", &remedy()).unwrap(), [enode]);

        let vetted = BTreeMap::from([(
            "alpha".to_string(),
            Err("deploy verification FAILED: x".to_string()),
        )]);
        let err = handed_out(&vetted, "gamma", &remedy())
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("  ✗ alpha: deploy verification FAILED: x"),
            "{err}"
        );
        assert!(err.contains("Nothing was sent"), "{err}");
        assert!(
            err.contains("seismic-tee node harvest net --node nodes.json --force"),
            "{err}"
        );
        assert!(err.contains("--name gamma --bootnode <ENODE>"), "{err}");
    }
}
