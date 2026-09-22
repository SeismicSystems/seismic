//! Node descriptors: the handoff from provisioning to configuration.
//!
//! The descriptor file is the cohort's `nodes` map — exactly what the
//! seismic_node Pulumi program's `pulumi stack output nodes --json` prints:
//!
//! ```json
//! {
//!   "devnet-3-1": {"public_ip": "203.0.113.7", "fqdn": "n1.seismicdev.net"},
//!   "devnet-3-2": {"public_ip": "203.0.113.8", "fqdn": "n2.seismicdev.net"}
//! }
//! ```
//!
//! Each entry describes one provisioned node: the public IP to reach it at
//! (`public_ip`) and the FQDN clients use (`fqdn`). The key is the node's name
//! — the name its harvest record and founding validator slot carry — so
//! identity lives in the file's content, never in how a file was saved. The
//! file is the boundary between the infrastructure layer (provisioning, owned
//! by Pulumi and run standalone) and these CLIs: a CLI consumes the map and
//! never wraps Pulumi. Saving the stack output as-is is one idempotent command
//! with no per-node bookkeeping, so re-running it after any map edit (add,
//! remove, re-image) cannot leave the file drifted from the stack.
//!
//! Each network's cohort lives in the context file's `[networks.<name>.nodes]`
//! table, imported from this same JSON shape by `ctx set-nodes`; `--node
//! FILE`/`--nodes FILE` still read one from disk directly, the way a script's
//! complete record should. A bring-your-own-infra operator (Terraform, manual
//! console, …) hand-writes a one-key map in the same shape: only
//! `public_ip`/`fqdn` are read, and any extra keys are ignored.

use std::collections::BTreeMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{Error, Result};

/// One provisioned node, as the `nodes` map describes it. Its name is its key
/// in a [`Descriptors`] map.
///
/// Round-trips through TOML as well as JSON: this is the shape of one
/// `[networks.<name>.nodes]` entry in the context file, so the context crate
/// stores and re-reads a node table with no second definition of what a node
/// is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeDescriptor {
    /// The address the operator-only ports are reached on.
    pub public_ip: String,
    /// The node's DNS name — the cert domain, and where clients reach its RPC.
    pub fqdn: String,
}

/// The cohort, keyed by node name.
///
/// A `BTreeMap` so iteration is in sorted node-name order — the order the
/// authored withdrawal credentials pair against. `pulumi stack output --json`
/// sorts keys, so a saved stack output already reads in this order; a
/// hand-written map need not.
pub type Descriptors = BTreeMap<String, NodeDescriptor>;

/// A key that must carry a usable value, in each of the states it can be in.
///
/// The nesting is what separates an absent key from a present one holding
/// `null`: the outer `Option` is serde's "was the key there at all", the inner
/// one is the `null`. Both are rejected, but they are different mistakes and an
/// operator is told which — a key that isn't there is usually a typo, while a
/// `null` is usually Pulumi reporting an output that never got set.
type RequiredKey = Option<Option<String>>;

/// Deserialize a [`RequiredKey`], recording that the key was present.
///
/// The outer `Option` has to be filled in here rather than left to serde:
/// serde reads a JSON `null` as `None` at every level of nesting, so a plain
/// `Option<Option<String>>` collapses `null` and absent back into the one value
/// this type exists to tell apart. serde reaches this function only for a key
/// that was written, which is exactly what the outer `Some` means.
fn present_key<'de, D>(deserializer: D) -> std::result::Result<RequiredKey, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

/// Exactly the two keys read out of each map entry. Unknown keys are ignored,
/// so a `pulumi stack output --json` that carries more per node still parses.
#[derive(Deserialize)]
struct DescriptorEntry {
    #[serde(default, deserialize_with = "present_key")]
    public_ip: RequiredKey,
    #[serde(default, deserialize_with = "present_key")]
    fqdn: RequiredKey,
    /// The keys this entry has that aren't read. Kept only so a failure can
    /// name them: the mistake is nearly always a neighbouring key, and an
    /// operator staring at "missing `public_ip`" wants to see what was there.
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

impl DescriptorEntry {
    /// Every key the entry actually had, sorted.
    ///
    /// "Had" means the key was written, whatever its value: a `public_ip` set
    /// to `null` is listed, because the point of this list is to show the
    /// operator what is in front of them.
    fn keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = self.extra.keys().map(String::as_str).collect();
        if self.public_ip.is_some() {
            keys.push("public_ip");
        }
        if self.fqdn.is_some() {
            keys.push("fqdn");
        }
        keys.sort_unstable();
        keys
    }

    /// Validate into a descriptor, naming the label and the node on failure.
    fn into_descriptor(self, label: &dyn Display, name: &str) -> Result<NodeDescriptor> {
        // An absent key, an explicit `null`, and an empty string all fail the
        // same way — a descriptor that can't reach a node — and all three are
        // caught here rather than at the first request. They are reported
        // apart, though, because the fix differs: only the absent case is
        // helped by listing the keys that *were* there, and only it is likely
        // to be a typo. Saying "missing" for a key the operator can plainly see
        // in the file sends them looking in the wrong place.
        let require = |key: &str, value: &RequiredKey| match value {
            Some(Some(value)) if !value.is_empty() => Ok(value.clone()),
            Some(Some(_)) => Err(Error::gate(format!(
                "descriptor file {label}: node `{name}` has an empty `{key}`",
            ))),
            Some(None) => Err(Error::gate(format!(
                "descriptor file {label}: node `{name}` has `{key}` set to null \
                 (Pulumi emits null for an output that never got set)",
            ))),
            None => Err(Error::gate(format!(
                "descriptor file {label}: node `{name}` is missing required key `{key}`; \
                 got keys {:?}",
                self.keys(),
            ))),
        };

        Ok(NodeDescriptor {
            public_ip: require("public_ip", &self.public_ip)?,
            fqdn: require("fqdn", &self.fqdn)?,
        })
    }
}

/// Read and validate a descriptor map.
///
/// Reading is the only way to build one from JSON: every failure names the
/// offending file (and the entry), which is the whole point of a hand-writable
/// format. Refused: a top level that isn't an object, an empty map, an entry
/// that isn't an object, and an entry whose `public_ip`/`fqdn` is absent,
/// `null`, or empty.
pub fn load_descriptors(path: &Path) -> Result<Descriptors> {
    let bytes = std::fs::read(path).map_err(|e| Error::read(path, e))?;
    parse_descriptors(&path.display(), &bytes)
}

/// Parse a descriptor map out of `bytes`, naming `label` in every failure.
///
/// `label` is a path's `Display` for [`load_descriptors`], and `<stdin>` for
/// `ctx set-nodes`, which parses the same bytes read off stdin through this
/// same function so a malformed map gets the same errors either way.
pub fn parse_descriptors(label: &dyn Display, bytes: &[u8]) -> Result<Descriptors> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| Error::json(PathBuf::from(label.to_string()), e))?;
    let Some(entries) = value.as_object() else {
        return Err(Error::gate(format!(
            "descriptor file {label} must be a JSON object mapping node name → \
             {{public_ip, fqdn}} (the stack's `nodes` output)",
        )));
    };
    if entries.is_empty() {
        return Err(Error::gate(format!(
            "descriptor file {label} holds no nodes — is the stack's `nodes` map empty?",
        )));
    }
    if entries.contains_key("public_ip")
        && entries.contains_key("fqdn")
        && entries.values().all(|v| !v.is_object())
    {
        // One node's `{public_ip, fqdn}` at the top level — the shape a
        // descriptor file had before it was the map. Say so rather than
        // reporting "node `fqdn` must be an object".
        return Err(Error::gate(format!(
            "descriptor file {label} is a single node's {{public_ip, fqdn}}, not the \
             map; the descriptor file is {{<name>: {{public_ip, fqdn}}, …}} — \
             wrap it under the node's name",
        )));
    }

    let mut descriptors = Descriptors::new();
    for (name, entry) in entries {
        if !entry.is_object() {
            return Err(Error::gate(format!(
                "descriptor file {label}: node `{name}` must be an object with \
                 public_ip and fqdn; got {}",
                json_kind(entry),
            )));
        }
        let entry: DescriptorEntry = serde_json::from_value(entry.clone())
            .map_err(|e| Error::json(PathBuf::from(label.to_string()), e))?;
        descriptors.insert(name.clone(), entry.into_descriptor(label, name)?);
    }
    Ok(descriptors)
}

/// Pick the one node a single-node command acts on.
///
/// With one entry in the file it is the node; with several, `name` says
/// which. Returns the node's name with it, since the descriptor itself does
/// not carry it. `holder` names what holds the map — a descriptor file's
/// path, or the context's `network <name> in <config path>` — so the same
/// error work reads well for either caller.
pub fn select_descriptor<'a>(
    descriptors: &'a Descriptors,
    name: Option<&str>,
    holder: &dyn Display,
) -> Result<(&'a str, &'a NodeDescriptor)> {
    let names = || {
        descriptors
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ")
    };
    match name {
        None if descriptors.len() == 1 => {
            let (name, descriptor) = descriptors.iter().next().expect("one entry");
            Ok((name, descriptor))
        }
        None => Err(Error::gate(format!(
            "{holder} holds {} nodes ({}); pass --name to say which",
            descriptors.len(),
            names(),
        ))),
        Some(name) => descriptors
            .get_key_value(name)
            .map(|(name, descriptor)| (name.as_str(), descriptor))
            .ok_or_else(|| {
                Error::gate(format!(
                    "{holder} has no node `{name}`; it holds {}",
                    names(),
                ))
            }),
    }
}

/// The JSON type name, for an error that says what was there instead.
fn json_kind(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "a boolean",
        serde_json::Value::Number(_) => "a number",
        serde_json::Value::String(_) => "a string",
        serde_json::Value::Array(_) => "an array",
        serde_json::Value::Object(_) => "an object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const N1: &str = r#"{"public_ip": "203.0.113.7", "fqdn": "n1.seismicdev.net"}"#;
    const N2: &str = r#"{"public_ip": "203.0.113.8", "fqdn": "n2.seismicdev.net"}"#;

    fn n1() -> NodeDescriptor {
        NodeDescriptor {
            public_ip: "203.0.113.7".into(),
            fqdn: "n1.seismicdev.net".into(),
        }
    }

    fn parse(json: &str) -> Result<Descriptors> {
        parse_descriptors(&"nodes/nodes.json", json.as_bytes())
    }

    fn fails(json: &str) -> String {
        let err = parse(json).unwrap_err().to_string();
        // Every failure names the file: the format is hand-writable.
        assert!(err.contains("nodes.json"), "{err}");
        err
    }

    #[test]
    fn the_map_is_the_stack_output() {
        let map = parse(&format!(r#"{{"dev-1": {N1}, "dev-2": {N2}}}"#)).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(map["dev-1"], n1());
        assert_eq!(map["dev-2"].fqdn, "n2.seismicdev.net");
    }

    /// The order the authored withdrawal credentials pair against, whatever
    /// order a hand-written file listed them in.
    #[test]
    fn iterates_in_node_name_order() {
        let map = parse(&format!(r#"{{"b": {N2}, "a": {N1}}}"#)).unwrap();
        assert_eq!(map.keys().collect::<Vec<_>>(), ["a", "b"]);
    }

    #[test]
    fn ignores_extra_keys_so_a_richer_stack_output_parses() {
        let map = parse(
            r#"{"dev-1": {
                "public_ip": "203.0.113.7",
                "fqdn": "n1.seismicdev.net",
                "resource_group": "dev-1",
                "vm_id": "/subscriptions/0000/resourceGroups/dev-1"
            }}"#,
        )
        .unwrap();

        assert_eq!(map["dev-1"], n1());
    }

    #[test]
    fn a_one_key_map_is_the_bring_your_own_infra_shape() {
        let map = parse(&format!(r#"{{"mine": {N1}}}"#)).unwrap();
        assert_eq!(map.keys().collect::<Vec<_>>(), ["mine"]);
    }

    #[test]
    fn the_top_level_must_be_the_map() {
        assert!(fails(&format!("[{N1}]")).contains("mapping node name"));
        assert!(fails("{}").contains("holds no nodes"));
    }

    /// The shape a descriptor file had before it was the map: refuse with the
    /// fix, rather than guess a name or report "node `fqdn` must be an object".
    #[test]
    fn a_bare_single_node_object_is_told_to_wrap_itself() {
        let err = fails(N1);
        assert!(err.contains("wrap it under the node's name"), "{err}");
        assert!(!err.contains("must be an object"), "{err}");
    }

    #[test]
    fn an_entry_must_be_an_object() {
        let err = fails(r#"{"dev-1": "203.0.113.7"}"#);
        assert!(err.contains("node `dev-1` must be an object"), "{err}");
        assert!(err.contains("got a string"), "{err}");

        let err = fails(r#"{"dev-1": null}"#);
        assert!(err.contains("got null"), "{err}");
    }

    /// Absent, explicitly `null`, and empty all fail: none reaches a node.
    ///
    /// Each is reported as the mistake it is, against the node it is in.
    /// Asserting on the distinguishing phrase, not just on `"public_ip"`, is
    /// the point — the key name alone appears in all three, so a test that
    /// checks only that would pass even if every case reported "missing",
    /// which is what an operator staring at a `public_ip` line in their own
    /// file must not be told.
    #[test]
    fn reports_an_absent_null_and_empty_key_apart() {
        let cases = [
            (
                r#"{"dev-1": {"fqdn": "n1.seismicdev.net"}}"#,
                "is missing required key `public_ip`",
            ),
            (
                r#"{"dev-1": {"public_ip": null, "fqdn": "n1.seismicdev.net"}}"#,
                "has `public_ip` set to null",
            ),
            (
                r#"{"dev-1": {"public_ip": "", "fqdn": "n1.seismicdev.net"}}"#,
                "has an empty `public_ip`",
            ),
        ];

        for (json, expected) in cases {
            let err = fails(json);
            assert!(err.contains(expected), "{err}");
            assert!(err.contains("node `dev-1`"), "{err}");
        }
    }

    /// A key that is present but unusable is never called "missing", and the
    /// key list — which exists to catch a typo — never contradicts the
    /// sentence in front of it by listing the key it just called absent.
    #[test]
    fn a_present_but_unusable_key_is_not_called_missing() {
        for json in [
            r#"{"dev-1": {"public_ip": null, "fqdn": "n1.seismicdev.net"}}"#,
            r#"{"dev-1": {"public_ip": "", "fqdn": "n1.seismicdev.net"}}"#,
        ] {
            let err = fails(json);
            assert!(!err.contains("missing"), "{err}");
            assert!(!err.contains("got keys"), "{err}");
        }
    }

    /// The failure names the keys that *were* there, because the mistake is
    /// nearly always a neighbouring key rather than an empty entry.
    #[test]
    fn a_missing_key_reports_the_keys_the_entry_had() {
        let err = fails(r#"{"dev-1": {"publicIp": "203.0.113.7", "fqdn": "n1.seismicdev.net"}}"#);

        assert!(err.contains("missing required key `public_ip`"), "{err}");
        assert!(err.contains(r#"["fqdn", "publicIp"]"#), "{err}");
    }

    /// One entry needs no name; several do; an unknown name lists the choices.
    #[test]
    fn selects_the_one_node_a_single_node_command_acts_on() {
        let holder = "nodes/nodes.json";
        let one = parse(&format!(r#"{{"dev-1": {N1}}}"#)).unwrap();
        let two = parse(&format!(r#"{{"dev-1": {N1}, "dev-2": {N2}}}"#)).unwrap();

        assert_eq!(select_descriptor(&one, None, &holder).unwrap().0, "dev-1");
        assert_eq!(
            select_descriptor(&one, Some("dev-1"), &holder).unwrap().0,
            "dev-1"
        );
        assert_eq!(
            select_descriptor(&two, Some("dev-2"), &holder).unwrap().0,
            "dev-2"
        );

        let err = select_descriptor(&two, None, &holder)
            .unwrap_err()
            .to_string();
        assert!(err.contains("pass --name"), "{err}");
        assert!(err.contains("dev-1, dev-2"), "{err}");

        let err = select_descriptor(&two, Some("dev-9"), &holder)
            .unwrap_err()
            .to_string();
        assert!(err.contains("no node `dev-9`"), "{err}");
        assert!(err.contains("dev-1, dev-2"), "{err}");
    }

    /// `select_descriptor` names whatever `holder` it is given — a
    /// descriptor file's path for `--node FILE`, or the context's own
    /// `network <name> in <config path>` phrasing — with no "descriptor
    /// file " prefix baked in.
    #[test]
    fn select_descriptors_messages_name_the_holder_they_were_given() {
        let two = parse(&format!(r#"{{"dev-1": {N1}, "dev-2": {N2}}}"#)).unwrap();
        let holder = "network `devnet-1` in /config.toml";

        let err = select_descriptor(&two, None, &holder)
            .unwrap_err()
            .to_string();
        assert!(err.starts_with(holder), "{err}");

        let err = select_descriptor(&two, Some("dev-9"), &holder)
            .unwrap_err()
            .to_string();
        assert!(err.starts_with(holder), "{err}");
    }

    /// The context crate stores a node table as `[networks.<name>.nodes]`
    /// entries, so a [`NodeDescriptor`] must round-trip through TOML with no
    /// second definition of what a node is.
    #[test]
    fn node_descriptor_round_trips_through_toml() {
        let toml = r#"public_ip = "203.0.113.7"
fqdn = "n1.seismicdev.net"
"#;
        let parsed: NodeDescriptor = toml::from_str(toml).unwrap();
        assert_eq!(parsed, n1());
        assert_eq!(toml::to_string(&parsed).unwrap(), toml);

        let err = toml::from_str::<NodeDescriptor>("public_ip = \"1\"\nbogus = 1")
            .unwrap_err()
            .to_string();
        assert!(err.contains("bogus"), "{err}");
    }

    #[test]
    fn names_the_file_on_every_failure_to_read_it() {
        let dir = tempfile::tempdir().unwrap();

        let good = dir.path().join("nodes.json");
        std::fs::write(&good, format!(r#"{{"dev-2": {N2}}}"#)).unwrap();
        assert_eq!(
            load_descriptors(&good).unwrap().keys().collect::<Vec<_>>(),
            ["dev-2"]
        );

        let malformed = dir.path().join("malformed.json");
        std::fs::write(&malformed, b"not json").unwrap();
        let err = load_descriptors(&malformed).unwrap_err().to_string();
        assert!(err.contains("malformed.json"), "{err}");

        let err = load_descriptors(&dir.path().join("absent.json"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("absent.json"), "{err}");
    }
}
