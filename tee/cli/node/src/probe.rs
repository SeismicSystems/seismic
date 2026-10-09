//! The live state of a node table: what each node answers on its two
//! attestation-service ports, and what those answers say about the network.
//!
//! Per node, two reads, made together:
//!
//! - `GET :7879/v1/quote` (operator-only): 200 while the node awaits its
//!   config POST this boot, serving its candidate `tx_io_pk@0`; 410 once it
//!   took one. The same endpoint `harvest` reads, through the same
//!   [`fetch_quote`].
//! - `getLuksProvisioningStatus` on `:7878` (public): the attestation service
//!   binds that port only while the custodian holds `root_key`, so any answer
//!   marks a key holder, and the answer itself is the first-boot disk wipe's
//!   progress.
//!
//! The readings are not DCAP-verified: the quote is read for its candidate
//! and its status code, never appraised. They inform; a decision that rests on
//! one verifies it where the decision is made.

use std::collections::{BTreeMap, BTreeSet};

use jsonrpsee::rpc_params;
use seismic_tee_common::founding::{FoundingRecords, box_holding_the_pin};
use seismic_tee_common::{Descriptors, Error, Manifest, rpc};
use serde_json::Value;
use tokio::task::JoinSet;

use crate::harvest::{FetchError, HarvestTarget, fetch_quote};
use crate::status::RPC_METHOD;

/// Where one node answers: its name in the node table and its two ports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeTarget {
    pub name: String,
    /// `http://<ip>:7879`.
    pub harvest_url: String,
    /// `http://<ip>:7878`.
    pub attestation_rpc_url: String,
}

/// One target per node in `descriptors`, in node-name order.
pub fn targets(descriptors: &Descriptors) -> Vec<ProbeTarget> {
    descriptors
        .iter()
        .map(|(name, descriptor)| ProbeTarget {
            name: name.clone(),
            harvest_url: descriptor.harvest_url(),
            attestation_rpc_url: descriptor.attestation_rpc_url(),
        })
        .collect()
}

/// What `:7879` says about this boot's config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigState {
    /// 200: awaiting its config POST, with the candidate its custodian
    /// minted at boot.
    Awaiting {
        node_public_key: String,
        candidate_tx_io_public_key: String,
    },
    /// 410: took its config POST this boot.
    Configured,
    /// 5xx, or connected but no full answer in time: up, with its summit
    /// keys or its candidate not yet written, or its quote queued behind
    /// another's.
    NotReady(String),
    /// Any other 4xx, or a body that is not a quote: an image and a CLI that
    /// disagree on the harvest API.
    Refused(String),
    /// No connection: still booting, gone, or — the port being operator-only —
    /// another operator's node.
    NoAnswer(String),
}

/// What `:7878` answered `getLuksProvisioningStatus` with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Disk {
    /// The status, undecoded — read with
    /// [`crate::status::LuksProvisioningStatus::from_result`].
    Status(Value),
    /// A JSON-RPC error object: the port is bound, the status is not served.
    Error(String),
}

/// One node's live state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeState {
    pub config: ConfigState,
    /// `Some` while the node serves `:7878` — a key holder — with what that
    /// port said about the disk.
    pub key_holder: Option<Disk>,
}

impl NodeState {
    /// The node answered on either port.
    pub fn reachable(&self) -> bool {
        self.key_holder.is_some() || !matches!(self.config, ConfigState::NoAnswer(_))
    }
}

/// Read one node's two ports, concurrently.
pub async fn probe_node(client: &reqwest::Client, target: &ProbeTarget) -> NodeState {
    let quote_target = HarvestTarget {
        name: target.name.clone(),
        harvest_url: target.harvest_url.clone(),
        nonce: rand::random(),
    };
    let (quote, key_holder) = tokio::join!(
        fetch_quote(client, &quote_target),
        read_disk(&target.attestation_rpc_url),
    );
    let config = match quote {
        Ok(quote) => ConfigState::Awaiting {
            node_public_key: quote.node_public_key,
            candidate_tx_io_public_key: quote.candidate_tx_io_public_key,
        },
        Err(FetchError::WindowClosed(_)) => ConfigState::Configured,
        Err(FetchError::NotReady(message)) => ConfigState::NotReady(message),
        Err(FetchError::Rejected(status, url)) => {
            ConfigState::Refused(format!("HTTP {status} from {url}"))
        }
        Err(FetchError::Malformed(message)) => ConfigState::Refused(message),
        Err(FetchError::Unreachable(message)) => ConfigState::NoAnswer(message),
    };
    NodeState { config, key_holder }
}

/// `:7878`'s answer, or `None` when nothing answered there.
async fn read_disk(url: &str) -> Option<Disk> {
    let client = rpc::Client::new(url).ok()?;
    match client.call(RPC_METHOD, rpc_params![]).await {
        Ok(status) => Some(Disk::Status(status)),
        Err(error @ Error::Rpc { .. }) => Some(Disk::Error(error.to_string())),
        Err(_) => None,
    }
}

/// Probe every target at once, keyed by name.
pub async fn probe_all(
    client: &reqwest::Client,
    targets: &[ProbeTarget],
) -> BTreeMap<String, NodeState> {
    let mut tasks = JoinSet::new();
    for target in targets.iter().cloned() {
        let client = client.clone();
        tasks.spawn(async move {
            let state = probe_node(&client, &target).await;
            (target.name, state)
        });
    }
    let mut states = BTreeMap::new();
    while let Some(joined) = tasks.join_next().await {
        let (name, state) = joined.expect("a probe task never panics");
        states.insert(name, state);
    }
    states
}

/// What the network directory pins about the founding: the candidate the
/// manifest pins, the box that harvested it, and every founding box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Founding {
    /// The manifest's `founding_tx_io_pk`, bare hex as the harvest endpoint
    /// spells a candidate.
    pub pin: String,
    /// The harvested box whose candidate is the pin. `None` when no harvest
    /// record holds it.
    pub pinned_box: Option<String>,
    /// Every harvested box: the founding nodes.
    pub founders: BTreeSet<String>,
}

impl Founding {
    /// The founding as `manifest` and its harvest `records` pin it. Without
    /// records, only the pin is known.
    pub fn new(manifest: &Manifest, records: Option<&FoundingRecords>) -> Self {
        Self {
            pin: hex::encode(manifest.founding_tx_io_pk),
            pinned_box: records
                .and_then(|r| box_holding_the_pin(r, &manifest.founding_tx_io_pk))
                .map(str::to_string),
            founders: records
                .map(|r| r.keys().cloned().collect())
                .unwrap_or_default(),
        }
    }

    /// Whether `candidate` is the pin. Both are lowercase bare hex: the
    /// harvest endpoint's spelling is checked by [`fetch_quote`].
    pub fn pins(&self, candidate: &str) -> bool {
        candidate == self.pin
    }
}

/// The network, as the nodes asked show it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkState {
    /// Some node asked serves `:7878`.
    Live { key_holders: Vec<String> },
    /// No key holder, and the pinned box awaits its config still holding the
    /// pinned candidate: a founding can start.
    Unfounded { pinned_box: String },
    /// Neither.
    NoReachableKeyHolder(Asked),
}

/// Who was asked, for a [`NetworkState::NoReachableKeyHolder`]: it means
/// something different to each.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asked {
    /// Every founding node, the pinned box among them. When the pinned box
    /// answered, the founding `root_key` is gone; when it did not, it may
    /// still be booting.
    EveryFounder {
        pinned_box: String,
        pinned_box_answered: bool,
    },
    /// Only some of the network's nodes, or nodes whose founding is unknown
    /// here: their silence says nothing about the rest of the network.
    Some,
}

/// Read the network line from the nodes' states.
pub fn assess(states: &BTreeMap<String, NodeState>, founding: Option<&Founding>) -> NetworkState {
    let key_holders: Vec<String> = states
        .iter()
        .filter(|(_, state)| state.key_holder.is_some())
        .map(|(name, _)| name.clone())
        .collect();
    if !key_holders.is_empty() {
        return NetworkState::Live { key_holders };
    }
    let Some((founding, pinned_box)) =
        founding.and_then(|f| f.pinned_box.as_deref().map(|pinned| (f, pinned)))
    else {
        return NetworkState::NoReachableKeyHolder(Asked::Some);
    };
    if let Some(pinned) = states.get(pinned_box)
        && let ConfigState::Awaiting {
            candidate_tx_io_public_key,
            ..
        } = &pinned.config
        && founding.pins(candidate_tx_io_public_key)
    {
        return NetworkState::Unfounded {
            pinned_box: pinned_box.to_string(),
        };
    }
    if founding
        .founders
        .iter()
        .all(|name| states.contains_key(name))
    {
        return NetworkState::NoReachableKeyHolder(Asked::EveryFounder {
            pinned_box: pinned_box.to_string(),
            pinned_box_answered: states[pinned_box].reachable(),
        });
    }
    NetworkState::NoReachableKeyHolder(Asked::Some)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::test_support::{
        FIXTURE_FOUNDING_TX_IO_PK, FakeServer, azure_evidence, candidate_tx_io_public_key,
        consensus_key, refused_url, rpc_result,
    };

    const NODE_KEY: &str = "abababababababababababababababababababababababababababababababab";

    fn quote_body(candidate: &str) -> String {
        json!({
            "node_public_key": NODE_KEY,
            "consensus_public_key": consensus_key("cd"),
            "candidate_tx_io_public_key": candidate,
            "evidence": azure_evidence(),
        })
        .to_string()
    }

    fn target(harvest_url: &str, attestation_rpc_url: &str) -> ProbeTarget {
        ProbeTarget {
            name: "n".to_string(),
            harvest_url: harvest_url.to_string(),
            attestation_rpc_url: attestation_rpc_url.to_string(),
        }
    }

    fn awaiting(candidate: &str) -> NodeState {
        NodeState {
            config: ConfigState::Awaiting {
                node_public_key: NODE_KEY.to_string(),
                candidate_tx_io_public_key: candidate.to_string(),
            },
            key_holder: None,
        }
    }

    fn configured() -> NodeState {
        NodeState {
            config: ConfigState::Configured,
            key_holder: None,
        }
    }

    fn holder() -> NodeState {
        NodeState {
            config: ConfigState::Configured,
            key_holder: Some(Disk::Status(json!({"state": "idle"}))),
        }
    }

    fn silent() -> NodeState {
        NodeState {
            config: ConfigState::NoAnswer("connection refused".to_string()),
            key_holder: None,
        }
    }

    fn states(entries: &[(&str, NodeState)]) -> BTreeMap<String, NodeState> {
        entries
            .iter()
            .map(|(name, state)| (name.to_string(), state.clone()))
            .collect()
    }

    /// alpha harvested the pin; beta and gamma are the other founders.
    fn founding() -> Founding {
        Founding {
            pin: FIXTURE_FOUNDING_TX_IO_PK.to_string(),
            pinned_box: Some("alpha".to_string()),
            founders: ["alpha", "beta", "gamma"].map(String::from).into(),
        }
    }

    /// A node awaiting its config serves its candidate on `:7879` and nothing
    /// on `:7878`.
    #[tokio::test]
    async fn an_unconfigured_node_serves_its_candidate_and_holds_no_key() {
        let harvest = FakeServer::serve(vec![(200, quote_body(FIXTURE_FOUNDING_TX_IO_PK))]);
        let state = probe_node(
            &seismic_tee_common::http::client().unwrap(),
            &target(&harvest.url, &refused_url()),
        )
        .await;

        assert_eq!(state, awaiting(FIXTURE_FOUNDING_TX_IO_PK));
        assert!(state.reachable());
        assert!(
            harvest.requests()[0].path.starts_with("/v1/quote?nonce="),
            "{:?}",
            harvest.requests()
        );
    }

    /// A key holder answers 410 on `:7879` and its disk status on `:7878`.
    #[tokio::test]
    async fn a_key_holder_is_configured_and_serves_its_disk_status() {
        let harvest = FakeServer::serve(vec![(410, "gone".to_string())]);
        let rpc = FakeServer::serve(vec![(
            200,
            rpc_result(json!({"state": "provisioning", "bytes_done": 1, "bytes_total": 2})),
        )]);
        let state = probe_node(
            &seismic_tee_common::http::client().unwrap(),
            &target(&harvest.url, &rpc.url),
        )
        .await;

        assert_eq!(state.config, ConfigState::Configured);
        assert_eq!(
            state.key_holder,
            Some(Disk::Status(
                json!({"state": "provisioning", "bytes_done": 1, "bytes_total": 2})
            ))
        );
    }

    /// Neither port answering is a node still booting, or gone; a 5xx on
    /// `:7879` is one that answers but is not ready yet.
    #[tokio::test]
    async fn no_answer_is_unreachable_and_a_5xx_is_not_ready() {
        let client = seismic_tee_common::http::client().unwrap();

        let state = probe_node(&client, &target(&refused_url(), &refused_url())).await;
        assert!(
            matches!(state.config, ConfigState::NoAnswer(_)),
            "{state:?}"
        );
        assert!(!state.reachable());

        let harvest = FakeServer::serve(vec![(503, "not yet".to_string())]);
        let state = probe_node(&client, &target(&harvest.url, &refused_url())).await;
        assert!(
            matches!(state.config, ConfigState::NotReady(_)),
            "{state:?}"
        );
        assert!(state.reachable());
    }

    /// A JSON-RPC error on `:7878` still marks a key holder: the port is
    /// bound only while the custodian holds `root_key`.
    #[tokio::test]
    async fn an_rpc_error_on_7878_is_still_a_key_holder() {
        let rpc = FakeServer::serve(vec![(
            200,
            r#"{"jsonrpc": "2.0", "id": 1, "error": {"code": -32601, "message": "no such method"}}"#
                .to_string(),
        )]);
        let state = probe_node(
            &seismic_tee_common::http::client().unwrap(),
            &target(&refused_url(), &rpc.url),
        )
        .await;
        assert!(
            matches!(&state.key_holder, Some(Disk::Error(message)) if message.contains("no such method")),
            "{state:?}"
        );
        assert!(state.reachable());
    }

    #[tokio::test]
    async fn probe_all_keys_each_state_by_node_name() {
        let harvest = FakeServer::serve(vec![(410, "gone".to_string())]);
        let targets = vec![
            ProbeTarget {
                name: "a".to_string(),
                harvest_url: harvest.url.clone(),
                attestation_rpc_url: refused_url(),
            },
            ProbeTarget {
                name: "b".to_string(),
                harvest_url: refused_url(),
                attestation_rpc_url: refused_url(),
            },
        ];
        let states = probe_all(&seismic_tee_common::http::client().unwrap(), &targets).await;
        assert_eq!(states["a"].config, ConfigState::Configured);
        assert!(!states["b"].reachable());
    }

    #[test]
    fn any_key_holder_makes_the_network_live() {
        let state = assess(
            &states(&[("alpha", silent()), ("beta", holder()), ("gamma", holder())]),
            Some(&founding()),
        );
        assert_eq!(
            state,
            NetworkState::Live {
                key_holders: vec!["beta".to_string(), "gamma".to_string()]
            }
        );
        // The founding is not needed to see a live network.
        assert!(matches!(
            assess(&states(&[("x", holder())]), None),
            NetworkState::Live { .. }
        ));
    }

    #[test]
    fn the_pinned_box_awaiting_with_the_pin_is_unfounded() {
        let state = assess(
            &states(&[
                ("alpha", awaiting(FIXTURE_FOUNDING_TX_IO_PK)),
                ("beta", awaiting(&candidate_tx_io_public_key("bb"))),
            ]),
            Some(&founding()),
        );
        assert_eq!(
            state,
            NetworkState::Unfounded {
                pinned_box: "alpha".to_string()
            }
        );
    }

    /// The pinned box rebooted since the harvest: it re-minted, so its
    /// candidate is no longer the pin and nothing can found.
    #[test]
    fn a_pinned_box_with_another_candidate_cannot_found() {
        let state = assess(
            &states(&[
                ("alpha", awaiting(&candidate_tx_io_public_key("aa"))),
                ("beta", configured()),
                ("gamma", configured()),
            ]),
            Some(&founding()),
        );
        assert_eq!(
            state,
            NetworkState::NoReachableKeyHolder(Asked::EveryFounder {
                pinned_box: "alpha".to_string(),
                pinned_box_answered: true,
            })
        );
    }

    #[test]
    fn a_silent_pinned_box_is_told_apart_from_a_lost_key() {
        let state = assess(
            &states(&[("alpha", silent()), ("beta", silent()), ("gamma", silent())]),
            Some(&founding()),
        );
        assert_eq!(
            state,
            NetworkState::NoReachableKeyHolder(Asked::EveryFounder {
                pinned_box: "alpha".to_string(),
                pinned_box_answered: false,
            })
        );
    }

    /// A table missing a founding node — an operator's own nodes, or one
    /// narrowed by --name — says nothing about the network.
    #[test]
    fn a_partial_table_says_nothing_about_the_network() {
        for table in [
            states(&[("beta", configured()), ("gamma", configured())]),
            states(&[("operator-1", configured())]),
        ] {
            assert_eq!(
                assess(&table, Some(&founding())),
                NetworkState::NoReachableKeyHolder(Asked::Some)
            );
        }
        // Nor does any table whose founding is unknown.
        assert_eq!(
            assess(&states(&[("alpha", configured())]), None),
            NetworkState::NoReachableKeyHolder(Asked::Some)
        );
    }

    #[test]
    fn the_founding_reads_the_pin_and_its_box_from_the_manifest_and_harvest() {
        let manifest = Manifest::from_json_bytes(crate::test_support::FIXTURE_MANIFEST).unwrap();
        let founding = Founding::new(&manifest, None);
        assert_eq!(founding.pin, FIXTURE_FOUNDING_TX_IO_PK);
        assert!(founding.pinned_box.is_none() && founding.founders.is_empty());
        assert!(founding.pins(FIXTURE_FOUNDING_TX_IO_PK));
        assert!(!founding.pins(&candidate_tx_io_public_key("aa")));
    }
}
