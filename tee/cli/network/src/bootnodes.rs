//! Founding bootnode set: fetch, persist, and reuse cohort enodes.
//!
//! A freshly-configured reth node's enode (its devp2p node record) isn't known
//! until reth is up, so a greenfield cohort can't be handed a static bootnode
//! list up front — the genesis node must come up first, then its enode seeds
//! the joiners (the two-stage bootstrap in `configure`). This module fetches a
//! node's enode from its `seismic_nodeInfo` RPC and persists the founding set
//! beside the descriptors as `nodes/bootnodes.json`.
//!
//! `seismic_nodeInfo` is read over the node's public JSON-RPC
//! (`https://<fqdn>/rpc`, nginx in front of reth) with the same client the
//! attestation service is read with ([`rpc::Client`]); the `seismic` namespace
//! deliberately keeps nodeInfo public (reth's `admin` namespace is disabled on
//! these nodes).
//!
//! `bootnodes.json` is runtime infra state, exactly like the node descriptors
//! it sits next to (`nodes/`, gitignored): regenerated each configure run, and
//! read back on a later run to reconfigure the whole cohort (reboots wipe the
//! tmpfs conf dir) or to seed a joiner — the founding enodes, unlike live IPs,
//! are stable because each node's devp2p key lives on its encrypted disk.
//!
//! reth's NodeRecord Display appends `?discport=<udp_port>` when its devp2p
//! UDP port differs from its TCP port; tdx-init rejects that form at POST time
//! (the `[network].bootnodes` grammar is strictly `enode://<128 hex>@host:port`).
//! So every fetched enode is normalized ([`normalize_enode`]) — the query
//! dropped, bracketed IPv6 preserved — before it is delivered or persisted.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use jsonrpsee::rpc_params;
use seismic_tee_common::rpc;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Enode-readiness polling. reth serves `seismic_nodeInfo` only once it is up
/// (root_key → LUKS open → reth start), so an early miss is the normal boot
/// tail — polled until timeout.
pub const POLL_INTERVAL: Duration = Duration::from_secs(5);
pub const ENODE_TIMEOUT: Duration = Duration::from_secs(15 * 60);
pub const WAIT_LOG_INTERVAL: Duration = Duration::from_secs(30);

/// One founding bootnode: the node's name and its enode URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bootnode {
    pub name: String,
    pub enode: String,
}

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

/// Persist the founding bootnode set (pretty JSON, trailing newline).
/// Overwrites: it's a fresh snapshot each configure run.
pub fn save_bootnodes(path: &Path, bootnodes: &[Bootnode]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let mut bytes = serde_json::to_vec_pretty(&json!({"bootnodes": bootnodes}))
        .context("rendering the bootnode set")?;
    bytes.push(b'\n');
    std::fs::write(path, bytes).with_context(|| format!("writing {}", path.display()))
}

/// Load a `bootnodes.json` written by [`save_bootnodes`].
pub fn load_bootnodes(path: &Path) -> anyhow::Result<Vec<Bootnode>> {
    #[derive(Deserialize)]
    struct File {
        #[serde(default)]
        bootnodes: Vec<Bootnode>,
    }
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let file: File = serde_json::from_slice(&bytes)
        .with_context(|| format!("{} is not a bootnode set", path.display()))?;
    Ok(file.bootnodes)
}

#[cfg(test)]
mod tests {
    use seismic_tee_common::test_support::{FakeServer, refused_url, rpc_result};

    use super::*;

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

    #[test]
    fn save_then_load_round_trips_and_an_empty_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nodes").join("bootnodes.json");
        let set = vec![
            Bootnode {
                name: "node-1".into(),
                enode: format!("enode://{}@1.1.1.1:30303", id()),
            },
            Bootnode {
                name: "node-2".into(),
                enode: format!("enode://{}@2.2.2.2:30303", "cd".repeat(64)),
            },
        ];
        save_bootnodes(&path, &set).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.ends_with("}\n"), "{text:?}");
        assert!(text.contains("\"bootnodes\": ["), "{text}");
        assert_eq!(load_bootnodes(&path).unwrap(), set);

        std::fs::write(&path, "{}").unwrap();
        assert!(load_bootnodes(&path).unwrap().is_empty());
    }
}
