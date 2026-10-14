//! What a `configure` run does, read from the network's live state.
//!
//! [`decide`] takes the node table, its [`crate::probe`], and the nodes the
//! run names, and parses them into the [`Plan`] — each node carrying what the
//! run does to it — or the refusal that nothing can be sent. It reads, never
//! sends: the probe is not DCAP-verified, so a plan that rests on a reading
//! verifies it before the first POST (the pinned box's quote, the key
//! holders' appraisals).
//!
//! Per node named:
//!
//! - **Awaiting** (`:7879` 200): configured.
//! - **Configured** (`:7879` 410, or a key holder): took its config this
//!   boot. Skipped, and said so.
//! - **Not up yet** (no answer, or not ready): configured once it answers,
//!   within tdx-init's listener patience.
//! - **Refused** (another 4xx, a body that is not a quote): an image and a
//!   CLI that disagree on the harvest API. Nothing is sent.
//!
//! The network, as [`assess`] reads it:
//!
//! - **Unfounded**: the run founds. The pinned box goes first and alone, so
//!   the run must name it; the rest fetch `root_key` from it. A founding
//!   node whose `:7879` serves a node key other than its harvest record's
//!   rebooted since the harvest — its consensus slot is dead — so the run
//!   refuses before any POST and says to re-found. Each founding box is
//!   checked again right before its own POST
//!   ([`Planned::harvested_node_key`]): the rest of a founding wait out the
//!   pinned box's wipe, and a node not up at the probe is read only once it
//!   answers.
//! - **Live**: every node fetches `root_key` from the key holders that pass
//!   their appraisal.
//! - **No reachable key holder**: refused, worded by who was asked, unless
//!   `--bootnode` names a live peer.
//!
//! The plan decides orchestration only. Safety stays in the custodian, which
//! keeps or discards its candidate and checks every installed key against
//! the manifest's pin, whatever the CLI concluded.

use std::collections::{BTreeMap, BTreeSet};
use std::net::IpAddr;

use anyhow::bail;
use seismic_tee_common::founding::{FoundingRecord, FoundingRecords};
use seismic_tee_common::http::HARVEST_PORT;
use seismic_tee_common::{Descriptors, NodeDescriptor};

use crate::bootnodes::KeyHolder;
use crate::probe::{Asked, ConfigState, Founding, NetworkState, NodeState, assess};
use crate::status::network_line;

/// Whether a node answered the probe awaiting its config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Up {
    /// `:7879` served its quote, and the plan's gates passed on it.
    Awaiting,
    /// No answer yet: waited for, then gated as the plan gated the rest,
    /// before its POST.
    NotUp,
}

/// A node the run configures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    pub name: String,
    pub descriptor: NodeDescriptor,
    pub up: Up,
    /// In a founding, the node key harvested from it, checked again right
    /// before its POST: a box serving another rebooted since the harvest.
    pub harvested_node_key: Option<String>,
    /// Its harvest record, when it is a founding box: what the launch
    /// assertions hold it to.
    pub record: Option<FoundingRecord>,
}

/// The pinned box of a founding: awaiting its config by the plan's reading,
/// and always a founding box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pinned {
    pub name: String,
    pub descriptor: NodeDescriptor,
    pub record: FoundingRecord,
}

/// Where a joining run's nodes fetch `root_key` from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Through {
    /// The node table's key holders, once each passes its appraisal.
    KeyHolders(Vec<KeyHolder>),
    /// The peers `--bootnode` names.
    NamedPeers,
}

/// What the run does, with the nodes it configures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Run {
    /// The network is unfounded: the pinned box first and alone, with no
    /// bootnodes, then the rest through it.
    Found { pinned: Pinned, rest: Vec<Planned> },
    /// Nodes join a live network.
    Join {
        nodes: Vec<Planned>,
        through: Through,
    },
}

impl Run {
    /// The nodes configured, in the order the run takes them: the pinned box
    /// first, then the rest in name order.
    pub fn names(&self) -> Vec<&str> {
        match self {
            Self::Found { pinned, rest } => std::iter::once(pinned.name.as_str())
                .chain(rest.iter().map(|node| node.name.as_str()))
                .collect(),
            Self::Join { nodes, .. } => nodes.iter().map(|node| node.name.as_str()).collect(),
        }
    }

    /// The nodes configured that did not answer the probe.
    pub fn not_up(&self) -> Vec<&str> {
        let rest = match self {
            Self::Found { rest, .. } => rest,
            Self::Join { nodes, .. } => nodes,
        };
        rest.iter()
            .filter(|node| node.up == Up::NotUp)
            .map(|node| node.name.as_str())
            .collect()
    }
}

/// What one `configure` run does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// `None` when every node named took its config this boot.
    pub run: Option<Run>,
    /// The nodes named that took their config this boot.
    pub skipped: Vec<String>,
}

/// What a refusal suggests instead, spelled with this run's flags so it
/// pastes. Each field is empty or leading-space-prefixed.
#[derive(Debug, Clone, Default)]
pub struct Remedy {
    /// The flags naming the node table.
    pub table: String,
    /// The nodes the run named.
    pub names: String,
    /// ` --manifest <FILE>`, when one was given.
    pub manifest: String,
    /// The network directory, for the re-found harvest, when `--manifest`
    /// named one; the context supplies it otherwise.
    pub dir: String,
    /// The appraisal's non-default flags (`--policy`, `--measurements`,
    /// `--pccs-url`), so a suggested appraisal runs the one this run chose.
    pub appraisal: String,
}

impl Remedy {
    /// This run, joining through a peer named by hand.
    pub fn bootnode(&self) -> String {
        format!(
            "seismic-tee node configure{}{} --bootnode <ENODE>{}",
            self.table, self.names, self.manifest
        )
    }

    /// `configure` over `names`, with this run's table and manifest.
    pub fn configure(&self, names: &[&str]) -> String {
        let names: String = names.iter().map(|name| format!(" {name}")).collect();
        format!(
            "seismic-tee node configure{}{names}{}",
            self.table, self.manifest
        )
    }

    /// `verify` of `name`, appraised as this run appraises.
    pub fn verify(&self, name: &str) -> String {
        format!(
            "seismic-tee node verify{} --name {name}{}{}",
            self.table, self.manifest, self.appraisal
        )
    }

    /// `node status --watch` over this run's table.
    pub fn watch(&self) -> String {
        format!(
            "seismic-tee node status --watch{}{}",
            self.table, self.manifest
        )
    }

    pub fn re_found(&self) -> String {
        format!(
            "re-found: `pulumi destroy`, a fresh `pulumi up`, then `seismic-tee node harvest{}{} \
             --force`",
            self.dir, self.table
        )
    }
}

/// The nodes a run names, checked against the node table: `names`, or every
/// node in the table when none is named, in name order. A name the table
/// does not hold is a typo, not a node; two named nodes sharing an IP are
/// one box configured twice.
fn named<'a>(
    table: &'a Descriptors,
    names: &[String],
) -> anyhow::Result<Vec<(&'a str, &'a NodeDescriptor)>> {
    let wanted: BTreeSet<&str> = names.iter().map(String::as_str).collect();
    let unknown: Vec<&str> = wanted
        .iter()
        .copied()
        .filter(|name| !table.contains_key(*name))
        .collect();
    if !unknown.is_empty() {
        bail!(
            "no such node(s) in the node table: {} — it holds {}",
            unknown.join(", "),
            table.keys().cloned().collect::<Vec<_>>().join(", ")
        );
    }
    let named: Vec<(&str, &NodeDescriptor)> = table
        .iter()
        .filter(|(name, _)| wanted.is_empty() || wanted.contains(name.as_str()))
        .map(|(name, descriptor)| (name.as_str(), descriptor))
        .collect();
    let mut by_ip: BTreeMap<IpAddr, Vec<&str>> = BTreeMap::new();
    for (name, descriptor) in &named {
        by_ip.entry(descriptor.public_ip).or_default().push(name);
    }
    let shared: Vec<String> = by_ip
        .iter()
        .filter(|(_, names)| names.len() > 1)
        .map(|(ip, names)| format!("{ip} ({})", names.join(", ")))
        .collect();
    if !shared.is_empty() {
        bail!(
            "node(s) sharing a public_ip in the node table: {} — one box would be configured \
             twice",
            shared.join("; ")
        );
    }
    Ok(named)
}

/// Decide the run over `names` (every node when empty), nodes of `table`,
/// whose probe is `states`.
///
/// `founding` is what the network directory pins, `records` its harvest
/// records, and `named_peers` whether `--bootnode` was given.
pub fn decide(
    table: &Descriptors,
    names: &[String],
    states: &BTreeMap<String, NodeState>,
    founding: &Founding,
    records: Option<&FoundingRecords>,
    named_peers: bool,
    remedy: &Remedy,
) -> anyhow::Result<Plan> {
    // The probe read every node in the table, so each named node has its
    // reading: from here on a node travels with it.
    let named: Vec<(&str, &NodeDescriptor, &NodeState)> = named(table, names)?
        .into_iter()
        .map(|(name, descriptor)| (name, descriptor, &states[name]))
        .collect();
    let refused: Vec<String> = named
        .iter()
        .filter_map(|(name, _, state)| match &state.config {
            ConfigState::Refused(message) => {
                Some(format!("  ✗ {name}: :{HARVEST_PORT}: {message}"))
            }
            _ => None,
        })
        .collect();
    if !refused.is_empty() {
        bail!(
            "node(s) whose harvest endpoint answers in a way this CLI does not read:\n{}\nNothing \
             was sent. The image and this CLI disagree on the harvest API: upgrade seismic-tee \
             (`seismic-tee --upgrade`), or check which image the nodes run.",
            refused.join("\n")
        );
    }

    let (skipped, configure): (Vec<_>, Vec<_>) = named.into_iter().partition(|(_, _, state)| {
        state.key_holder.is_some() || state.config == ConfigState::Configured
    });
    let skipped: Vec<String> = skipped
        .into_iter()
        .map(|(name, ..)| name.to_string())
        .collect();
    if configure.is_empty() {
        return Ok(Plan { run: None, skipped });
    }
    let record = |name: &str| records.and_then(|records| records.get(name)).cloned();
    // `founding` decides whether a node is held to its harvested node key
    // right before its POST.
    let planned = |(name, descriptor, state): (&str, &NodeDescriptor, &NodeState),
                   founding: bool| Planned {
        name: name.to_string(),
        descriptor: descriptor.clone(),
        up: match state.config {
            ConfigState::Awaiting { .. } => Up::Awaiting,
            _ => Up::NotUp,
        },
        harvested_node_key: record(name)
            .filter(|_| founding)
            .map(|record| record.node_public_key),
        // Every boot serves fresh keys until the config POST, and
        // summit-persist then restores a persisted keystore: only the launch
        // assertions, which wait for it, can tell a joining founder's dead
        // slot from a live one.
        record: record(name),
    };

    let network = assess(states, Some(founding));
    let run = match &network {
        NetworkState::Unfounded { pinned_box } => {
            if named_peers {
                bail!(
                    "{}\nNothing was sent. A founding fetches root_key from the pinned box, not \
                     from a named peer: drop --bootnode to found it.",
                    network_line(&network)
                );
            }
            let (pinned, rest): (Vec<_>, Vec<_>) = configure
                .into_iter()
                .partition(|(name, ..)| *name == pinned_box.as_str());
            let [(_, descriptor, _)] = pinned[..] else {
                let names: Vec<&str> = std::iter::once(pinned_box.as_str())
                    .chain(rest.iter().map(|(name, ..)| *name))
                    .collect();
                bail!(
                    "{}\nNothing was sent: the pinned box goes first, and the nodes named would \
                     have no peer to fetch root_key from. Found it with them:\n    {}",
                    network_line(&network),
                    remedy.configure(&names)
                );
            };
            refuse_rebooted(pinned.iter().chain(&rest), records, remedy)?;
            // `assess` names the pinned box from the harvest records, so this
            // holds; the probe's `Founding` carries the name, not the record.
            let Some(pinned_record) = record(pinned_box) else {
                bail!(
                    "the pinned box {pinned_box} has no harvest record: re-run `seismic-tee \
                     network assemble` over this harvest, or re-found"
                );
            };
            Run::Found {
                pinned: Pinned {
                    name: pinned_box.clone(),
                    descriptor: descriptor.clone(),
                    record: pinned_record,
                },
                rest: rest.into_iter().map(|node| planned(node, true)).collect(),
            }
        }
        _ if named_peers => Run::Join {
            nodes: configure
                .into_iter()
                .map(|node| planned(node, false))
                .collect(),
            through: Through::NamedPeers,
        },
        NetworkState::Live { key_holders } => Run::Join {
            nodes: configure
                .into_iter()
                .map(|node| planned(node, false))
                .collect(),
            through: Through::KeyHolders(
                key_holders
                    .iter()
                    .map(|name| KeyHolder::new(name, &table[name]))
                    .collect(),
            ),
        },
        NetworkState::NoReachableKeyHolder(asked) => {
            let instead = match asked {
                Asked::EveryFounder {
                    pinned_box_answered: true,
                    ..
                } => format!(
                    "{}; or name a live peer:\n    {}",
                    remedy.re_found(),
                    remedy.bootnode()
                ),
                Asked::EveryFounder {
                    pinned_box_answered: false,
                    ..
                } => format!(
                    "watch for it to come up (`{}`), or name a live peer:\n    {}",
                    remedy.watch(),
                    remedy.bootnode()
                ),
                Asked::Some => format!(
                    "name a live peer by its enode (its `seismic_nodeInfo`):\n    {}",
                    remedy.bootnode()
                ),
            };
            bail!(
                "{}\nNothing was sent: no node would have a peer to fetch root_key from. Instead, \
                 {instead}",
                network_line(&network)
            )
        }
    };
    Ok(Plan {
        run: Some(run),
        skipped,
    })
}

/// Whether `state` serves a node key other than `record`'s: the box rebooted
/// since the harvest. A box not up at the probe reads as not rebooted.
fn rebooted(state: &NodeState, record: &FoundingRecord) -> bool {
    matches!(
        &state.config,
        ConfigState::Awaiting { node_public_key, .. } if *node_public_key != record.node_public_key
    )
}

/// A founding's refusal of the nodes it would configure that rebooted since
/// the harvest: each serves a node key other than its harvest record's.
fn refuse_rebooted<'a: 'b, 'b>(
    configure: impl Iterator<Item = &'b (&'a str, &'a NodeDescriptor, &'a NodeState)>,
    records: Option<&FoundingRecords>,
    remedy: &Remedy,
) -> anyhow::Result<()> {
    let refused: Vec<String> = configure
        .filter_map(|(name, _, state)| {
            let record = records?.get(*name)?;
            match &state.config {
                ConfigState::Awaiting {
                    node_public_key, ..
                } if rebooted(state, record) => Some(format!(
                    "  ✗ {name}: serves node key {node_public_key}, harvested {}",
                    record.node_public_key
                )),
                _ => None,
            }
        })
        .collect();
    if !refused.is_empty() {
        bail!(
            "founding node(s) rebooted since the harvest:\n{}\nNothing was sent. A box mints \
             fresh keys at every boot, so the keys the summit genesis pins for it are gone and \
             its consensus slot is dead. Instead, {}.",
            refused.join("\n"),
            remedy.re_found()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use seismic_tee_common::test_support::{FIXTURE_FOUNDING_TX_IO_PK, candidate_tx_io_public_key};
    use serde_json::json;

    use super::*;
    use crate::probe::Disk;

    const ALPHA_KEY: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const BETA_KEY: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn remedy() -> Remedy {
        Remedy {
            table: " --node nodes.json".to_string(),
            names: String::new(),
            manifest: " --manifest net/network-manifest.json".to_string(),
            dir: " net".to_string(),
            appraisal: String::new(),
        }
    }

    fn awaiting(node_public_key: &str, candidate: &str) -> NodeState {
        NodeState {
            config: ConfigState::Awaiting {
                node_public_key: node_public_key.to_string(),
                candidate_tx_io_public_key: candidate.to_string(),
            },
            key_holder: None,
        }
    }

    fn holder() -> NodeState {
        NodeState {
            config: ConfigState::Configured,
            key_holder: Some(Disk::Status(json!({"state": "idle"}))),
        }
    }

    fn configured() -> NodeState {
        NodeState {
            config: ConfigState::Configured,
            key_holder: None,
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

    fn record(node_public_key: &str, candidate: &str) -> FoundingRecord {
        FoundingRecord {
            node_public_key: node_public_key.to_string(),
            consensus_public_key: "cc".repeat(48),
            candidate_tx_io_public_key: candidate.to_string(),
            document: json!({}),
        }
    }

    /// alpha harvested the pin; beta is the other founder.
    fn records() -> FoundingRecords {
        BTreeMap::from([
            (
                "alpha".to_string(),
                record(ALPHA_KEY, FIXTURE_FOUNDING_TX_IO_PK),
            ),
            (
                "beta".to_string(),
                record(BETA_KEY, &candidate_tx_io_public_key("bb")),
            ),
        ])
    }

    fn founding() -> Founding {
        Founding {
            pin: FIXTURE_FOUNDING_TX_IO_PK.to_string(),
            pinned_box: Some("alpha".to_string()),
            founders: ["alpha", "beta"].map(String::from).into(),
        }
    }

    fn names(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn unfounded() -> BTreeMap<String, NodeState> {
        states(&[
            ("alpha", awaiting(ALPHA_KEY, FIXTURE_FOUNDING_TX_IO_PK)),
            (
                "beta",
                awaiting(BETA_KEY, &candidate_tx_io_public_key("bb")),
            ),
        ])
    }

    fn descriptor(ip: &str, name: &str) -> NodeDescriptor {
        NodeDescriptor {
            public_ip: ip.parse().unwrap(),
            fqdn: format!("{name}.example.com").parse().unwrap(),
        }
    }

    /// The node table the probe `states` read: one IP per node.
    fn table_of(states: &BTreeMap<String, NodeState>) -> Descriptors {
        states
            .keys()
            .enumerate()
            .map(|(i, name)| (name.clone(), descriptor(&format!("10.0.0.{i}"), name)))
            .collect()
    }

    fn decide_over(
        named: &[&str],
        states: &BTreeMap<String, NodeState>,
        named_peers: bool,
    ) -> anyhow::Result<Plan> {
        decide(
            &table_of(states),
            &names(named),
            states,
            &founding(),
            Some(&records()),
            named_peers,
            &remedy(),
        )
    }

    /// `name` as a founding plans it (held to its harvested node key), or a
    /// join.
    fn planned(
        states: &BTreeMap<String, NodeState>,
        name: &str,
        up: Up,
        record: Option<FoundingRecord>,
        founding: bool,
    ) -> Planned {
        Planned {
            name: name.to_string(),
            descriptor: table_of(states)[name].clone(),
            up,
            harvested_node_key: record
                .as_ref()
                .filter(|_| founding)
                .map(|record| record.node_public_key.clone()),
            record,
        }
    }

    /// No names is the whole table; names narrow it, in name order, each
    /// once.
    #[test]
    fn the_nodes_named_default_to_the_whole_table() {
        let table = table_of(&unfounded());
        let named_over = |wanted: &[&str]| -> Vec<String> {
            named(&table, &names(wanted))
                .unwrap()
                .into_iter()
                .map(|(name, _)| name.to_string())
                .collect()
        };
        assert_eq!(named_over(&[]), ["alpha", "beta"]);
        assert_eq!(named_over(&["beta", "alpha", "beta"]), ["alpha", "beta"]);
        assert_eq!(named_over(&["beta"]), ["beta"]);
    }

    /// A name the table does not hold is a typo; two named nodes sharing an
    /// IP are one box configured twice.
    #[test]
    fn unknown_names_and_shared_ips_are_refused() {
        let table = table_of(&unfounded());
        let err = named(&table, &names(&["gamma"])).unwrap_err().to_string();
        assert!(
            err.contains("no such node(s) in the node table: gamma — it holds alpha, beta"),
            "{err}"
        );

        let shared_ip = Descriptors::from([
            ("alpha".to_string(), descriptor("1.1.1.1", "alpha")),
            ("beta".to_string(), descriptor("1.1.1.1", "beta")),
        ]);
        let err = named(&shared_ip, &[]).unwrap_err().to_string();
        assert!(err.contains("1.1.1.1 (alpha, beta)"), "{err}");
    }

    /// A fresh founding cohort: the whole table, the pinned box first, each
    /// node with its harvest record.
    #[test]
    fn an_unfounded_network_is_founded_from_its_pinned_box() {
        let states = unfounded();
        let plan = decide_over(&["alpha", "beta"], &states, false).unwrap();
        assert_eq!(
            plan,
            Plan {
                run: Some(Run::Found {
                    pinned: Pinned {
                        name: "alpha".to_string(),
                        descriptor: table_of(&states)["alpha"].clone(),
                        record: records()["alpha"].clone(),
                    },
                    rest: vec![planned(
                        &states,
                        "beta",
                        Up::Awaiting,
                        Some(records()["beta"].clone()),
                        true
                    )],
                }),
                skipped: Vec::new(),
            }
        );
        assert_eq!(plan.run.unwrap().names(), ["alpha", "beta"]);
    }

    /// A founding node, up at the probe or not, is held to its harvested node
    /// key right before its POST; joining, nothing is pinned for it.
    #[test]
    fn a_founder_is_held_to_its_harvested_key_only_in_a_founding() {
        let mut states = unfounded();
        let founders_rest = |states: &BTreeMap<String, NodeState>| {
            let Some(Run::Found { rest, .. }) = decide_over(&[], states, false).unwrap().run else {
                panic!("an unfounded network is founded");
            };
            rest
        };
        let rest = founders_rest(&states);
        assert_eq!(rest[0].up, Up::Awaiting);
        assert_eq!(rest[0].harvested_node_key.as_deref(), Some(BETA_KEY));

        states.insert("beta".to_string(), silent());
        let rest = founders_rest(&states);
        assert_eq!(rest[0].up, Up::NotUp);
        assert_eq!(rest[0].harvested_node_key.as_deref(), Some(BETA_KEY));

        states.insert("alpha".to_string(), holder());
        let run = decide_over(&[], &states, false).unwrap().run.unwrap();
        let Run::Join { nodes, .. } = &run else {
            panic!("a network with a key holder is joined");
        };
        assert_eq!(nodes[0].up, Up::NotUp);
        assert_eq!(nodes[0].harvested_node_key, None);
        assert_eq!(nodes[0].record, Some(records()["beta"].clone()));
        assert_eq!(run.not_up(), ["beta"]);
    }

    /// A founding run names the pinned box, or there is nobody to fetch
    /// root_key from; the refusal spells the run that includes it.
    #[test]
    fn a_founding_without_the_pinned_box_is_refused() {
        let err = decide_over(&["beta"], &unfounded(), false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("network: unfounded"), "{err}");
        assert!(err.contains("Nothing was sent"), "{err}");
        assert!(
            err.contains(
                "seismic-tee node configure --node nodes.json alpha beta --manifest \
                 net/network-manifest.json"
            ),
            "{err}"
        );
    }

    #[test]
    fn a_founding_refuses_a_named_peer() {
        let err = decide_over(&["alpha", "beta"], &unfounded(), true)
            .unwrap_err()
            .to_string();
        assert!(err.contains("drop --bootnode"), "{err}");
    }

    /// A founding node serving a node key other than the one harvested from
    /// it rebooted: the run refuses before any POST, naming both keys.
    #[test]
    fn a_founding_node_rebooted_since_the_harvest_is_refused() {
        let mut states = unfounded();
        states.insert(
            "beta".to_string(),
            awaiting(&"ee".repeat(32), &candidate_tx_io_public_key("ee")),
        );
        let err = decide_over(&["alpha", "beta"], &states, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("rebooted since the harvest"), "{err}");
        assert!(
            err.contains(&format!(
                "✗ beta: serves node key {}, harvested {BETA_KEY}",
                "ee".repeat(32)
            )),
            "{err}"
        );
        assert!(
            err.contains("seismic-tee node harvest net --node nodes.json --force"),
            "{err}"
        );
    }

    /// A partially failed founding: the pinned box holds the key, so the
    /// rest join through it, and the nodes that took their POST are skipped.
    /// beta serves fresh keys, as every reboot does until summit-persist
    /// restores its keystore: it is still held to its harvested keys.
    #[test]
    fn a_rerun_skips_the_configured_and_joins_the_rest() {
        let states = states(&[
            ("alpha", holder()),
            (
                "beta",
                awaiting(&"ee".repeat(32), &candidate_tx_io_public_key("ee")),
            ),
            ("gamma", configured()),
            ("delta", silent()),
        ]);
        let plan = decide_over(&["alpha", "beta", "delta", "gamma"], &states, false).unwrap();
        assert_eq!(
            plan,
            Plan {
                run: Some(Run::Join {
                    nodes: vec![
                        planned(
                            &states,
                            "beta",
                            Up::Awaiting,
                            Some(records()["beta"].clone()),
                            false
                        ),
                        planned(&states, "delta", Up::NotUp, None, false),
                    ],
                    through: Through::KeyHolders(vec![KeyHolder::new(
                        "alpha",
                        &table_of(&states)["alpha"]
                    )]),
                }),
                skipped: names(&["alpha", "gamma"]),
            }
        );
    }

    /// A node whose `:7879` cannot answer but which serves `:7878` holds the
    /// key: it is configured, and skipped.
    #[test]
    fn a_key_holder_is_skipped_whatever_7879_says() {
        let mut holder = holder();
        holder.config = ConfigState::NoAnswer("refused".to_string());
        let plan = decide_over(
            &["alpha"],
            &states(&[("alpha", holder), ("beta", silent())]),
            false,
        )
        .unwrap();
        assert_eq!(plan.run, None);
        assert_eq!(plan.skipped, ["alpha"]);
    }

    /// `--bootnode` joins through the peers named, whatever the table holds.
    #[test]
    fn named_peers_join_a_table_without_key_holders() {
        let states = states(&[("operator-1", awaiting(&"ee".repeat(32), &"02".repeat(33)))]);
        let plan = decide_over(&["operator-1"], &states, true).unwrap();
        let run = plan.run.unwrap();
        assert!(
            matches!(
                run,
                Run::Join {
                    through: Through::NamedPeers,
                    ..
                }
            ),
            "{run:?}"
        );
        assert_eq!(run.names(), ["operator-1"]);
    }

    /// No key holder: the refusal reads the network as `node status` does,
    /// and names the way on for that state.
    #[test]
    fn no_reachable_key_holder_is_refused_by_who_was_asked() {
        // The pinned box re-minted: the founding root_key is gone.
        let lost = states(&[
            (
                "alpha",
                awaiting(ALPHA_KEY, &candidate_tx_io_public_key("aa")),
            ),
            (
                "beta",
                awaiting(BETA_KEY, &candidate_tx_io_public_key("bb")),
            ),
        ]);
        let err = decide_over(&["beta"], &lost, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("the founding root_key is gone"), "{err}");
        assert!(
            err.contains("seismic-tee node harvest net --node nodes.json --force"),
            "{err}"
        );
        assert!(
            err.contains(
                "seismic-tee node configure --node nodes.json --bootnode <ENODE> --manifest \
                 net/network-manifest.json"
            ),
            "{err}"
        );

        let booting = states(&[("alpha", silent()), ("beta", silent())]);
        let err = decide_over(&["beta"], &booting, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("does not answer"), "{err}");
        assert!(
            err.contains("node status --watch --node nodes.json"),
            "{err}"
        );

        // An operator's own table says nothing about the network.
        let own = states(&[("operator-1", awaiting(&"ee".repeat(32), &"02".repeat(33)))]);
        let err = decide_over(&["operator-1"], &own, false)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("no reachable key holder among these nodes"),
            "{err}"
        );
        assert!(err.contains("--bootnode <ENODE>"), "{err}");
        assert!(!err.contains("re-found"), "{err}");
    }

    /// A node every attempt is refused by is a version skew, not a boot tail.
    #[test]
    fn a_harvest_endpoint_this_cli_cannot_read_refuses_the_run() {
        let mut states = unfounded();
        states.insert(
            "beta".to_string(),
            NodeState {
                config: ConfigState::Refused("HTTP 404 from http://b:7879/v1/quote".to_string()),
                key_holder: None,
            },
        );
        let err = decide_over(&["alpha", "beta"], &states, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("✗ beta: :7879: HTTP 404"), "{err}");
        assert!(err.contains("Nothing was sent"), "{err}");
    }

    /// Every node named already took its config: nothing to do, nothing
    /// refused.
    #[test]
    fn a_table_already_configured_plans_nothing() {
        let states = states(&[("alpha", holder()), ("beta", configured())]);
        let plan = decide_over(&["alpha", "beta"], &states, false).unwrap();
        assert_eq!(plan.run, None);
        assert_eq!(plan.skipped, ["alpha", "beta"]);
    }
}
