//! The founding inputs a network directory holds before `assemble` runs.
//!
//! Two files plus the cohort's node table, read by three commands: the
//! authored withdrawal credentials
//! (`inputs/founder-withdrawal-credentials.json`), the harvested founding
//! records (`inputs/harvest/<node>.json`), and the cohort's node table —
//! the context file's `[networks.<name>.nodes]`, imported by `ctx set-nodes`,
//! or `--nodes FILE` for a script's complete record. `harvest` reads the
//! credentials and the table to size the cohort before it fetches anything;
//! `assemble` pairs all three into the founding validator set it pins (under
//! `--check`, the IPs come from the summit genesis on disk instead of the
//! table — see [`ValidatorIps`]); `configure` joins the records with the
//! table for the IPs it delivers and the keys it asserts the launch against.
//! Each reader validates everything
//! it reads even when an earlier command already did: these are plain
//! committed files (or a re-imported cohort) that may have changed between
//! commands.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context as _, bail};
use seismic_tee_common::{Descriptors, NetworkDir};
use serde::Serialize;

/// Summit's consensus (BLS) port: each validator entry in the completed summit
/// genesis pins `<ip>:<port>`. IPs are operational data — the config digest
/// excludes them — so they are delivered but never pinned.
pub const SUMMIT_CONSENSUS_PORT: u16 = 18551;

/// `s` is exactly `nbytes` of bare lowercase hex — summit's keystore wire
/// spelling, the form the genesis config digest commits to. Any other spelling
/// is rejected, never normalized, so nothing non-canonical is laundered into
/// the pinned set.
pub fn is_bare_hex(s: &str, nbytes: usize) -> bool {
    s.len() == 2 * nbytes
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// `s` is `0x` + 40 hex digits (either case): an address as the withdrawal
/// credentials spell one.
pub fn is_address(s: &str) -> bool {
    s.strip_prefix("0x")
        .is_some_and(|digits| digits.len() == 40 && digits.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// Load the authored `founder-withdrawal-credentials.json`: one 0x-prefixed
/// address per founder, in a JSON array.
///
/// A list rather than a node-name mapping so the founders' addresses are
/// authorable before any box exists — they are a fact about the founders, not
/// about the infrastructure. [`load_founding_set`] pairs the i-th address with
/// the i-th founding validator in node-name order.
pub fn load_founder_credentials(path: &Path) -> anyhow::Result<Vec<String>> {
    if !path.is_file() {
        bail!(
            "{} not found — author it as a JSON array of the founders' withdrawal credentials \
             (0x-prefixed addresses), one per founding node",
            path.display()
        );
    }
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("{}: not valid JSON", path.display()))?;
    let Some(entries) = value.as_array() else {
        bail!(
            "{}: expected a JSON array of withdrawal credentials (0x-prefixed addresses), one \
             per founding node",
            path.display()
        );
    };
    let mut addresses = Vec::with_capacity(entries.len());
    for entry in entries {
        let Some(address) = entry.as_str() else {
            bail!(
                "{}: expected a JSON array of withdrawal credentials (0x-prefixed addresses), \
                 one per founding node",
                path.display()
            );
        };
        addresses.push(address.to_string());
    }
    let mut bad: Vec<&str> = addresses
        .iter()
        .map(String::as_str)
        .filter(|a| !is_address(a))
        .collect();
    bad.sort_unstable();
    bad.dedup();
    if !bad.is_empty() {
        bail!(
            "{}: withdrawal credentials must be 0x + 40 hex chars; bad entr(ies): {}",
            path.display(),
            bad.join(", ")
        );
    }
    Ok(addresses)
}

/// One archived founding record, as the fields the founding is built from.
///
/// `document` is the whole file as archived — the input to the verify-quote
/// library's harvest check, handed back unchanged so what is re-verified is
/// what was verified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundingRecord {
    /// The ed25519 node pubkey, bare lowercase hex (32 bytes).
    pub node_public_key: String,
    /// The BLS consensus pubkey, bare lowercase hex (48 bytes).
    pub consensus_public_key: String,
    /// The archived file, whole.
    pub document: serde_json::Value,
}

/// The harvested founding records, keyed by node name (the file stem).
pub type FoundingRecords = BTreeMap<String, FoundingRecord>;

/// Read the harvested founding records (`inputs/harvest/<node>.json`).
///
/// Validates the fields the founding set is built from — nonce, both pubkeys,
/// the evidence object — and rejects a pubkey repeated across boxes: summit's
/// genesis keys validator accounts by node pubkey, so a repeated key silently
/// collapses the set, and a shared consensus key is accidental-equivocation
/// material.
pub fn load_harvest_records(dir: &NetworkDir) -> anyhow::Result<FoundingRecords> {
    let harvest_dir = dir.harvest();
    let mut paths: Vec<_> = match std::fs::read_dir(&harvest_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "json"))
            .collect(),
        Err(_) => Vec::new(),
    };
    paths.sort();
    if paths.is_empty() {
        bail!(
            "no harvest records in {} — assemble pins the founding validator set from them; \
             provision the cohort (the Pulumi program's `nodes` map) and run `seismic-tee network \
             harvest` first",
            harvest_dir.display()
        );
    }

    let mut records = FoundingRecords::new();
    for path in paths {
        let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        let document: serde_json::Value = serde_json::from_slice(&bytes)
            .with_context(|| format!("{}: not valid JSON", path.display()))?;
        let Some(object) = document.as_object() else {
            bail!("{}: expected a JSON object", path.display());
        };
        let field = |name: &str, nbytes: usize| -> anyhow::Result<String> {
            match object.get(name).and_then(serde_json::Value::as_str) {
                Some(value) if is_bare_hex(value, nbytes) => Ok(value.to_string()),
                other => bail!(
                    "{}: {name}: expected {nbytes}-byte lowercase bare hex, got {}",
                    path.display(),
                    other.map_or_else(|| "nothing".to_string(), |v| format!("{v:?}")),
                ),
            }
        };
        field("harvest_nonce", 32)?;
        let node_public_key = field("node_public_key", 32)?;
        let consensus_public_key = field("consensus_public_key", 48)?;
        if !object
            .get("evidence")
            .is_some_and(serde_json::Value::is_object)
        {
            bail!(
                "{}: no evidence object — without the archived quote the record cannot be \
                 re-verified, so it must not be pinned",
                path.display()
            );
        }
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        records.insert(
            name,
            FoundingRecord {
                node_public_key,
                consensus_public_key,
                document,
            },
        );
    }

    for (key_field, key_of) in [
        (
            "node_public_key",
            (|r: &FoundingRecord| r.node_public_key.as_str()) as fn(&FoundingRecord) -> &str,
        ),
        ("consensus_public_key", |r: &FoundingRecord| {
            r.consensus_public_key.as_str()
        }),
    ] {
        let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
        for (name, record) in &records {
            let key = key_of(record);
            if let Some(first) = seen.get(key) {
                bail!(
                    "{first} and {name} carry the same {key_field} ({key}); the harvest is not \
                     the distinct founder set being pinned — re-found and re-harvest"
                );
            }
            seen.insert(key, name);
        }
    }
    Ok(records)
}

/// One founding validator as `summit genesis set-validators` takes it:
/// harvested keys in summit's bare-lowercase-hex keystore spelling, the
/// authored credentials, and the descriptor IP with the consensus port.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Validator {
    pub node_public_key: String,
    pub consensus_public_key: String,
    pub ip_address: String,
    pub withdrawal_credentials: String,
}

/// The founding cohort as `assemble` pins it: the summit validator entries and
/// the harvest records they came from (for quote re-verification).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundingSet {
    pub validators: Vec<Validator>,
    pub records: FoundingRecords,
}

/// Where each founding validator's `ip_address` comes from.
///
/// IPs are topology, not identity: delivered in the summit genesis but
/// excluded from the config digest the manifest pins, so a box whose IP
/// rotates after the founding is still the same pinned validator. That is
/// why there are two sources. `assemble` seats the cohort's current IPs;
/// `assemble --check` re-derives with the IPs the genesis on disk already
/// seats, so the comparison asks whether the *identity* still derives from
/// the inputs and never fails on an IP that moved since the founding — and
/// needs no node table, so it runs on a committed directory as
/// `verify-founding` does.
#[derive(Debug, Clone)]
pub enum ValidatorIps<'a> {
    /// The cohort's node table (the context's, or `--nodes`): each box's
    /// current public IP, given the consensus port.
    Cohort(&'a Descriptors),
    /// The IPs a completed summit genesis seats, by node pubkey, verbatim —
    /// see [`seated_validator_ips`].
    Seated(BTreeMap<String, String>),
}

/// The `ip_address` each validator of a completed summit genesis seats, by
/// `node_public_key`, as the file spells it.
///
/// Read as [`ValidatorIps::Seated`] for `assemble --check`. Only the pairing
/// is read here — the keys' spelling and the seated set's agreement with the
/// harvest are `verify-founding`'s checks, and the re-derivation `--check`
/// runs holds every identity field to the inputs anyway.
pub fn seated_validator_ips(summit_genesis: &[u8]) -> anyhow::Result<BTreeMap<String, String>> {
    let text = std::str::from_utf8(summit_genesis).context("summit genesis is not valid TOML")?;
    let genesis: toml::Table = toml::from_str(text).context("summit genesis is not valid TOML")?;
    let Some(validators) = genesis.get("validators").and_then(toml::Value::as_array) else {
        bail!(
            "summit genesis has no validators array — an assembled genesis carries the founding \
             set; an authored input does not"
        );
    };
    let mut seated = BTreeMap::new();
    for (index, validator) in validators.iter().enumerate() {
        let field = |name: &str| -> anyhow::Result<String> {
            match validator.get(name).and_then(toml::Value::as_str) {
                Some(value) => Ok(value.to_string()),
                None => bail!("summit genesis validators[{index}].{name}: expected a string"),
            }
        };
        seated.insert(field("node_public_key")?, field("ip_address")?);
    }
    Ok(seated)
}

/// Pair the harvested cohort with its authored withdrawal credentials and
/// IPs into summit validator entries.
///
/// The credentials are positional: the i-th authored address goes to the i-th
/// harvested box in node-name order, and the counts must match exactly — one
/// address short means a box can't be pinned, one too many means the harvest
/// isn't the cohort the founders authored for, and either way assembling would
/// pin a set other than the intended one. The pairing is printed and lands
/// visibly in the emitted genesis, since nothing downstream can tell a swapped
/// pair from an intended one. IPs come from `ips` — the cohort's node table,
/// or the genesis on disk under `--check` — and are delivered in the genesis
/// file but excluded from its config digest, so the committed file is a
/// founding-era snapshot and IP churn never re-founds.
pub fn load_founding_set(dir: &NetworkDir, ips: &ValidatorIps<'_>) -> anyhow::Result<FoundingSet> {
    let founders = load_founder_credentials(&dir.founders())?;
    let records = load_harvest_records(dir)?;
    if founders.len() != records.len() {
        bail!(
            "{} carries {} withdrawal credential(s) but {} box(es) were harvested into {} ({}) — \
             author one address per founding node",
            dir.founders().display(),
            founders.len(),
            records.len(),
            dir.harvest().display(),
            records.keys().cloned().collect::<Vec<_>>().join(", "),
        );
    }
    let mut validators = Vec::with_capacity(records.len());
    for ((name, record), credentials) in records.iter().zip(&founders) {
        let ip_address = match ips {
            ValidatorIps::Cohort(descriptors) => {
                let Some(descriptor) = descriptors.get(name) else {
                    bail!(
                        "the cohort has no node {name:?} — the cohort's node table supplies each \
                         founding validator's IP. A harvested box that is gone from the cohort \
                         means it changed under the harvest: re-found rather than assembling",
                    );
                };
                format!("{}:{SUMMIT_CONSENSUS_PORT}", descriptor.public_ip)
            }
            ValidatorIps::Seated(seated) => {
                let Some(ip_address) = seated.get(&record.node_public_key) else {
                    bail!(
                        "{} seats no validator with {name}'s node key {} — the artifact set on \
                         disk was not assembled from this harvest; re-run `assemble` without \
                         --check",
                        dir.summit_genesis().display(),
                        record.node_public_key,
                    );
                };
                ip_address.clone()
            }
        };
        eprintln!("founding validator {name}: withdrawals to {credentials}");
        validators.push(Validator {
            node_public_key: record.node_public_key.clone(),
            consensus_public_key: record.consensus_public_key.clone(),
            ip_address,
            withdrawal_credentials: credentials.clone(),
        });
    }
    Ok(FoundingSet {
        validators,
        records,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use seismic_tee_common::NodeDescriptor;
    use serde_json::json;

    use super::*;

    pub(crate) const NODE_KEY_1: &str =
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    pub(crate) const NODE_KEY_2: &str =
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    pub(crate) fn consensus_key(byte: &str) -> String {
        byte.repeat(48)
    }

    /// Evidence in the backend's own serialization, claiming Azure TDX: a
    /// stand-in quote (`[1, 2, 3]` as base64) under the platform metadata
    /// the holder serves. Parses as an `AttestationExchangeMessage`; never
    /// verifies.
    pub(crate) fn azure_evidence() -> serde_json::Value {
        json!({
            "attestation_evidence": {
                "quote": "AQID",
                "platform": {
                    "attestation_type": "azure-tdx",
                    "ram_bytes": 0,
                    "num_disks": 0,
                    "acpi": null,
                },
            },
        })
    }

    /// Evidence declaring no attestation, in the backend's serialization.
    pub(crate) fn no_attestation_evidence() -> serde_json::Value {
        json!({"attestation_evidence": null})
    }

    /// A harvest record as the harvest builds it from the holder's answer.
    pub(crate) fn record(node_key: &str, consensus_byte: &str) -> serde_json::Value {
        json!({
            "harvest_nonce": "11".repeat(32),
            "node_public_key": node_key,
            "consensus_public_key": consensus_key(consensus_byte),
            "evidence": azure_evidence(),
        })
    }

    pub(crate) fn write(dir: &NetworkDir, relative: &Path, contents: &str) {
        let path = dir.root().join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    pub(crate) fn write_harvest(dir: &NetworkDir, name: &str, record: &serde_json::Value) {
        write(
            dir,
            &Path::new("inputs/harvest").join(format!("{name}.json")),
            &record.to_string(),
        );
    }

    pub(crate) fn network_dir() -> (tempfile::TempDir, NetworkDir) {
        let tmp = tempfile::tempdir().unwrap();
        let dir = NetworkDir::new(tmp.path());
        (tmp, dir)
    }

    #[test]
    fn hex_spellings_are_checked_not_normalized() {
        assert!(is_bare_hex(NODE_KEY_1, 32));
        assert!(!is_bare_hex(&NODE_KEY_1.to_uppercase(), 32));
        assert!(!is_bare_hex(&format!("0x{NODE_KEY_1}"), 32));
        assert!(!is_bare_hex(&NODE_KEY_1[1..], 32));

        assert!(is_address("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"));
        assert!(!is_address("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"));
        assert!(!is_address("0xf39F"));
    }

    #[test]
    fn founder_credentials_are_a_list_of_addresses() {
        let (_tmp, dir) = network_dir();
        let path = dir.founders();

        let err = load_founder_credentials(&path).unwrap_err().to_string();
        assert!(err.contains("not found"), "{err}");
        assert!(err.contains("one per founding node"), "{err}");

        write(
            &dir,
            Path::new("inputs/founder-withdrawal-credentials.json"),
            r#"["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", "0x0000000000000000000000000000000000000001"]"#,
        );
        assert_eq!(load_founder_credentials(&path).unwrap().len(), 2);

        write(
            &dir,
            Path::new("inputs/founder-withdrawal-credentials.json"),
            r#"["0xf39F", "nope", "0xf39F"]"#,
        );
        let err = load_founder_credentials(&path).unwrap_err().to_string();
        assert!(err.contains("bad entr(ies): 0xf39F, nope"), "{err}");

        write(
            &dir,
            Path::new("inputs/founder-withdrawal-credentials.json"),
            r#"{"node-1": "0xf39F"}"#,
        );
        let err = load_founder_credentials(&path).unwrap_err().to_string();
        assert!(err.contains("expected a JSON array"), "{err}");
    }

    #[test]
    fn harvest_records_are_read_in_name_order_with_their_documents() {
        let (_tmp, dir) = network_dir();
        write_harvest(&dir, "node-2", &record(NODE_KEY_2, "dd"));
        write_harvest(&dir, "node-1", &record(NODE_KEY_1, "cc"));

        let records = load_harvest_records(&dir).unwrap();
        assert_eq!(records.keys().collect::<Vec<_>>(), ["node-1", "node-2"]);
        assert_eq!(records["node-1"].node_public_key, NODE_KEY_1);
        assert_eq!(records["node-1"].document, record(NODE_KEY_1, "cc"));
    }

    #[test]
    fn an_empty_harvest_names_the_prerequisite() {
        let (_tmp, dir) = network_dir();
        let err = load_harvest_records(&dir).unwrap_err().to_string();
        assert!(err.contains("no harvest records"), "{err}");
        assert!(err.contains("seismic-tee network harvest"), "{err}");
    }

    #[test]
    fn a_record_with_a_bad_field_is_rejected_by_file() {
        let (_tmp, dir) = network_dir();
        let mut bad = record(NODE_KEY_1, "cc");
        bad["node_public_key"] = json!(NODE_KEY_1.to_uppercase());
        write_harvest(&dir, "node-1", &bad);
        let err = load_harvest_records(&dir).unwrap_err().to_string();
        assert!(err.contains("node-1.json"), "{err}");
        assert!(err.contains("node_public_key"), "{err}");

        let mut bad = record(NODE_KEY_1, "cc");
        bad.as_object_mut().unwrap().remove("evidence");
        write_harvest(&dir, "node-1", &bad);
        let err = load_harvest_records(&dir).unwrap_err().to_string();
        assert!(err.contains("no evidence object"), "{err}");
    }

    #[test]
    fn a_repeated_pubkey_is_not_a_distinct_founder_set() {
        let (_tmp, dir) = network_dir();
        write_harvest(&dir, "node-1", &record(NODE_KEY_1, "cc"));
        write_harvest(&dir, "node-2", &record(NODE_KEY_1, "dd"));
        let err = load_harvest_records(&dir).unwrap_err().to_string();
        assert!(err.contains("node-1 and node-2"), "{err}");
        assert!(err.contains("node_public_key"), "{err}");

        write_harvest(&dir, "node-2", &record(NODE_KEY_2, "cc"));
        let err = load_harvest_records(&dir).unwrap_err().to_string();
        assert!(err.contains("consensus_public_key"), "{err}");
    }

    /// A cohort's node table, built in memory: `load_founding_set`'s caller
    /// resolves this from the context or `--nodes` before calling it, so the
    /// tests here build it directly rather than through a file.
    pub(crate) fn descriptors_of(nodes: &[(&str, &str, &str)]) -> Descriptors {
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

    #[test]
    fn the_founding_set_pairs_records_credentials_and_ips_in_name_order() {
        let (_tmp, dir) = network_dir();
        write_harvest(&dir, "node-2", &record(NODE_KEY_2, "dd"));
        write_harvest(&dir, "node-1", &record(NODE_KEY_1, "cc"));
        write(
            &dir,
            Path::new("inputs/founder-withdrawal-credentials.json"),
            &format!(r#"["0x{}", "0x{}"]"#, "01".repeat(20), "02".repeat(20)),
        );
        let descriptors = descriptors_of(&[
            ("node-1", "203.0.113.7", "n1.example.com"),
            ("node-2", "203.0.113.8", "n2.example.com"),
        ]);

        let set = load_founding_set(&dir, &ValidatorIps::Cohort(&descriptors)).unwrap();
        assert_eq!(set.validators.len(), 2);
        assert_eq!(set.validators[0].node_public_key, NODE_KEY_1);
        assert_eq!(set.validators[0].ip_address, "203.0.113.7:18551");
        assert_eq!(
            set.validators[0].withdrawal_credentials,
            format!("0x{}", "01".repeat(20))
        );
        assert_eq!(set.validators[1].node_public_key, NODE_KEY_2);
        assert_eq!(set.validators[1].ip_address, "203.0.113.8:18551");
        assert_eq!(set.records.len(), 2);

        // Seated IPs stand in for the table verbatim — the founding-era ones,
        // whatever the boxes' IPs are today — and a harvested key the genesis
        // does not seat is not this set's.
        let seated: BTreeMap<String, String> = [
            (NODE_KEY_1.to_string(), "198.51.100.1:18551".to_string()),
            (NODE_KEY_2.to_string(), "198.51.100.2:18551".to_string()),
        ]
        .into();
        let set = load_founding_set(&dir, &ValidatorIps::Seated(seated.clone())).unwrap();
        assert_eq!(set.validators[0].ip_address, "198.51.100.1:18551");
        assert_eq!(set.validators[1].ip_address, "198.51.100.2:18551");

        let mut one_short = seated;
        one_short.remove(NODE_KEY_2);
        let err = load_founding_set(&dir, &ValidatorIps::Seated(one_short))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("seats no validator with node-2's node key"),
            "{err}"
        );
        assert!(err.contains(NODE_KEY_2), "{err}");
        assert!(err.contains("summit-genesis.toml"), "{err}");
        assert!(err.contains("without --check"), "{err}");
    }

    /// The seated IPs are read by node key from a completed genesis, as the
    /// file spells them; an authored input, which seats nobody, is refused.
    #[test]
    fn seated_ips_are_read_by_node_key() {
        let genesis = format!(
            "namespace = \"n\"\n\n[[validators]]\nnode_public_key = {NODE_KEY_2:?}\n\
             ip_address = \"203.0.113.8:18551\"\n\n[[validators]]\nnode_public_key = \
             {NODE_KEY_1:?}\nip_address = \"203.0.113.7:18551\"\n"
        );
        let seated = seated_validator_ips(genesis.as_bytes()).unwrap();
        assert_eq!(seated.len(), 2);
        assert_eq!(seated[NODE_KEY_1], "203.0.113.7:18551");
        assert_eq!(seated[NODE_KEY_2], "203.0.113.8:18551");

        let err = seated_validator_ips(b"namespace = \"n\"\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("no validators array"), "{err}");

        let err = seated_validator_ips(
            format!("[[validators]]\nnode_public_key = {NODE_KEY_1:?}\n").as_bytes(),
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("validators[0].ip_address"), "{err}");
    }

    #[test]
    fn a_count_mismatch_and_a_box_gone_from_the_cohort_are_cohort_changes() {
        let (_tmp, dir) = network_dir();
        write_harvest(&dir, "node-1", &record(NODE_KEY_1, "cc"));
        write_harvest(&dir, "node-2", &record(NODE_KEY_2, "dd"));
        write(
            &dir,
            Path::new("inputs/founder-withdrawal-credentials.json"),
            &format!(r#"["0x{}"]"#, "01".repeat(20)),
        );
        let err = load_founding_set(&dir, &ValidatorIps::Cohort(&Descriptors::new()))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("1 withdrawal credential(s) but 2 box(es)"),
            "{err}"
        );
        assert!(err.contains("node-1, node-2"), "{err}");

        write(
            &dir,
            Path::new("inputs/founder-withdrawal-credentials.json"),
            &format!(r#"["0x{}", "0x{}"]"#, "01".repeat(20), "02".repeat(20)),
        );
        let descriptors = descriptors_of(&[("node-1", "203.0.113.7", "n1.example.com")]);
        let err = load_founding_set(&dir, &ValidatorIps::Cohort(&descriptors))
            .unwrap_err()
            .to_string();
        assert!(err.contains("has no node \"node-2\""), "{err}");
        assert!(err.contains("re-found"), "{err}");
    }
}
