//! `verify-founding`: re-verify a network's founding — the genesis validator
//! set's TEE provenance — from its committed directory, offline.
//!
//! ```text
//! seismic-tee verify-founding tee/networks/devnet-3
//! seismic-tee verify-founding tee/networks/devnet-3 --record devnet-3-2
//! ```
//!
//! The auditor's command, and the binary's one top-level command. The auditor
//! takes no trust-sensitive action of their own — they verify the other
//! parties' after the fact — and their subject is the network's record as a
//! whole, so the command sits beside the groups rather than in any of them.
//! "Founding" is the docs' word for exactly this event and names what is
//! verified; "harvest", the founder's own name for the step that produced
//! the archive, is not something an auditor has to know.
//!
//! A committed network directory is everything an auditor needs to ask the
//! founding's question again: *these* keys were minted in measured boxes,
//! under *this* policy, and they are the set the network seats. Given the
//! directory, the command
//!
//! - reads `network-manifest.json` through the strict schema and takes
//!   `measurement-policy-bootstrap.json` beside it, refusing a policy whose
//!   hash is not the manifest's `bootstrap_policy_hash` — the same check
//!   `seismic-tee node verify` applies before appraising a node;
//! - reads the founding validator set from the completed `summit-genesis.toml`
//!   beside the manifest, and checks that every archived record's keys are
//!   seated there and every seat is vouched for by an archived record;
//! - replays every record under `inputs/harvest/` from the archive it is:
//!   the quote against the DCAP bundle archived with it, at the instant that
//!   verification was held to, reaching no collateral service. A record that
//!   is not a whole archive fails, as `assemble` fails it: live collateral
//!   would answer today and stop answering in a month.
//!
//! One line per record; the first failure ends the run with a nonzero exit,
//! naming the record. `--record <node>` narrows to one record for an auditor
//! holding part of an archive; the pinned set is then only asked to seat that
//! record. There is no `--pccs-url`: nothing here reaches a network.
//!
//! This is `network assemble --check`'s sibling: `--check` re-derives the
//! artifact set from its inputs and holds the set on disk to the result,
//! this audits the archive the set was pinned from. It is the offline half
//! of `assemble`'s gate on that archive — the two run one function,
//! [`verify_harvest_records`] — with a directory in front of it. Neither
//! needs the founder's shell-outs, so an auditor needs this binary and
//! nothing else.
//!
//! One pin is not recomputed here: `summit.genesis_config_digest` is summit's
//! own digest over the genesis and needs a summit build, which `assemble
//! --check` shells out to. The audit reads the seated set as the genesis file
//! carries it, after the manifest's own structural check of that file; a
//! genesis swapped under its manifest is `--check`'s catch.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context as _, bail};
use clap::Args;
use seismic_tee_common::network_dir::{HARVEST_DIRNAME, INPUTS_DIRNAME};
use seismic_tee_common::{Manifest, NetworkDir};
use seismic_tee_context::{Context, ContextArgs};

use crate::args::DirArgs;
use crate::assemble::verify_harvest_records;
use crate::founding::{FoundingRecords, is_bare_hex, load_harvest_records};
use crate::init::absolute;

#[derive(Debug, Args)]
pub struct VerifyFoundingArgs {
    /// Committed network directory: the artifact set `assemble` wrote at its
    /// top level (network-manifest.json, measurement-policy-bootstrap.json,
    /// summit-genesis.toml) and the founding archive under inputs/harvest/.
    /// Omit it to use the current context's network.
    #[command(flatten)]
    pub dir: DirArgs,

    /// Audit one archived record — its node name, the file stem under
    /// inputs/harvest/ — for a partial archive. The seated set is then only
    /// checked to carry that record's keys.
    #[arg(long, value_name = "NODE")]
    pub record: Option<String>,
}

/// A founding validator's identity as the completed summit genesis pins it:
/// its node pubkey and consensus pubkey, in summit's bare-hex spelling.
pub type ValidatorKeys = (String, String);

/// The founding validator set a completed summit genesis carries, as
/// `(node_public_key, consensus_public_key)` pairs.
///
/// The genesis is `assemble`'s artifact — the authored parameters with the
/// harvested set filled in through summit's own emitter — so this is the set
/// the manifest's `summit.genesis_config_digest` transitively commits to.
pub fn pinned_validator_keys(summit_genesis: &[u8]) -> anyhow::Result<Vec<ValidatorKeys>> {
    let text = std::str::from_utf8(summit_genesis).context("summit genesis is not valid TOML")?;
    let genesis: toml::Table = toml::from_str(text).context("summit genesis is not valid TOML")?;
    let Some(validators) = genesis.get("validators").and_then(toml::Value::as_array) else {
        bail!(
            "summit genesis has no validators array — an assembled genesis carries the founding \
             set; an authored input does not"
        );
    };
    let mut keys = Vec::with_capacity(validators.len());
    for (index, validator) in validators.iter().enumerate() {
        let field = |name: &str, nbytes: usize| -> anyhow::Result<String> {
            match validator.get(name).and_then(toml::Value::as_str) {
                Some(value) if is_bare_hex(value, nbytes) => Ok(value.to_string()),
                other => bail!(
                    "summit genesis validators[{index}].{name}: expected {nbytes}-byte lowercase \
                     bare hex, got {}",
                    other.map_or_else(|| "nothing".to_string(), |v| format!("{v:?}")),
                ),
            }
        };
        keys.push((
            field("node_public_key", 32)?,
            field("consensus_public_key", 48)?,
        ));
    }
    Ok(keys)
}

/// The archived records and the pinned validator set are the same set of
/// keys.
///
/// This is the auditor's actual question: *these* keys — the ones the
/// archived quotes prove were minted in measured boxes — are the ones the
/// genesis seats, and nothing else is seated. Both directions are checked:
/// a record whose keys the genesis does not carry, and a validator no
/// archived quote vouches for, each fail by name. With `partial` set (an
/// auditor holding one record of a larger archive) only the first direction
/// is asked; the archive cannot say whether the rest of the set is vouched
/// for.
pub fn check_records_are_the_pinned_set(
    records: &FoundingRecords,
    pinned: &[ValidatorKeys],
    partial: bool,
) -> anyhow::Result<()> {
    let pinned_set: BTreeSet<&ValidatorKeys> = pinned.iter().collect();
    let mut unpinned = Vec::new();
    let mut vouched = BTreeSet::new();
    for (name, record) in records {
        let keys = (
            record.node_public_key.clone(),
            record.consensus_public_key.clone(),
        );
        if pinned_set.contains(&keys) {
            vouched.insert(keys);
        } else {
            unpinned.push(format!(
                "{name} (node {}…, consensus {}…)",
                &record.node_public_key[..8],
                &record.consensus_public_key[..8]
            ));
        }
    }
    if !unpinned.is_empty() {
        bail!(
            "archived record(s) whose keys the summit genesis does not seat: {} — the archive \
             and the pinned validator set are not the same founding",
            unpinned.join(", ")
        );
    }
    if partial {
        return Ok(());
    }
    let unvouched: Vec<String> = pinned
        .iter()
        .filter(|keys| !vouched.contains(*keys))
        .map(|(node, consensus)| format!("node {}…, consensus {}…", &node[..8], &consensus[..8]))
        .collect();
    if !unvouched.is_empty() {
        bail!(
            "summit genesis seats validator(s) no archived quote vouches for: {} — the pinned set \
             is larger than the founding the archive proves",
            unvouched.join("; ")
        );
    }
    Ok(())
}

/// What a passing audit established.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundingAudit {
    pub manifest: Manifest,
    /// The records re-verified, in node-name order.
    pub verified: Vec<String>,
}

/// Read a file the manifest pins from beside it, naming the artifact set on
/// absence.
fn read_pinned(path: &Path, what: &str) -> anyhow::Result<Vec<u8>> {
    if !path.is_file() {
        bail!(
            "{} not found — the {what} sits beside the manifest in the artifact set `assemble` \
             writes; this directory is not a whole one",
            path.display()
        );
    }
    std::fs::read(path).with_context(|| format!("reading {}", path.display()))
}

/// Re-verify the founding a network directory commits to, offline.
///
/// Every input is checked against the manifest before any quote is
/// replayed, so a directory that is not one artifact set fails by file, not
/// as a verification failure of a quote it never applied to. `record`
/// narrows the archive to one node.
pub fn audit_founding(dir: &NetworkDir, record: Option<&str>) -> anyhow::Result<FoundingAudit> {
    let manifest_path = dir.manifest();
    if !manifest_path.is_file() {
        bail!(
            "{} not found — not an assembled network directory. The audit takes the directory \
             `assemble` wrote its artifact set to: the manifest, the policy and the completed \
             summit genesis at its top level, the founding archive under \
             {INPUTS_DIRNAME}/{HARVEST_DIRNAME}/",
            manifest_path.display()
        );
    }
    let manifest = Manifest::load(&manifest_path)
        .with_context(|| format!("{}: invalid manifest", manifest_path.display()))?;

    let policy_path = dir.policy();
    let policy = read_pinned(&policy_path, "measurement policy")?;
    manifest.check_policy(&policy).with_context(|| {
        format!(
            "{} is not the policy {} commits to — the founding quotes were held to the policy \
             the manifest pins, so this directory is not one artifact set",
            policy_path.display(),
            manifest_path.display()
        )
    })?;

    let summit_path = dir.summit_genesis();
    let summit_genesis = read_pinned(&summit_path, "completed summit genesis")?;
    manifest
        .check_summit_genesis(&summit_genesis)
        .with_context(|| format!("summit genesis {}", summit_path.display()))?;
    let pinned = pinned_validator_keys(&summit_genesis)
        .with_context(|| format!("summit genesis {}", summit_path.display()))?;

    let mut records = load_harvest_records(dir)?;
    if let Some(name) = record {
        let Some(one) = records.remove(name) else {
            bail!(
                "no record {name:?} in {} — archived records: {}",
                dir.harvest().display(),
                records.keys().cloned().collect::<Vec<_>>().join(", ")
            );
        };
        records = FoundingRecords::from([(name.to_string(), one)]);
    }
    check_records_are_the_pinned_set(&records, &pinned, record.is_some()).with_context(|| {
        format!(
            "{} against the validator set {} seats",
            dir.harvest().display(),
            summit_path.display()
        )
    })?;

    verify_harvest_records(dir, &records, &policy)?;
    Ok(FoundingAudit {
        manifest,
        verified: records.keys().cloned().collect(),
    })
}

/// The context's pinned `network_id` for this invocation's network, when it
/// has one.
///
/// Best-effort: a failed selection — no context configured, or `--context`
/// naming a network the file doesn't hold — means there is nothing to pin
/// against, not a hard error. `verify-founding` is the auditor's command, and
/// an auditor need not have run `ctx set-network` at all; a malformed
/// `--config` file still fails loudly, since that is a broken flag rather
/// than an absent one.
fn pinned_network_id(context_args: &ContextArgs) -> anyhow::Result<Option<String>> {
    let context = Context::load(context_args.config.as_deref())?;
    Ok(context
        .select(context_args.context.as_deref())
        .ok()
        .and_then(|selected| selected.network_id().map(str::to_string)))
}

pub async fn run(args: VerifyFoundingArgs) -> anyhow::Result<ExitCode> {
    let root = args.dir.load()?;
    if !root.is_dir() {
        bail!("network directory not found: {}", root.display());
    }
    let dir = NetworkDir::new(absolute(&root)?);

    // Compared before any record is replayed: an artifact set whose
    // network_id disagrees with the context's pin is refused outright,
    // rather than after minutes of quote re-verification.
    if let Some(pinned) = pinned_network_id(&args.dir.context)? {
        let manifest_path = dir.manifest();
        let manifest = Manifest::load(&manifest_path)
            .with_context(|| format!("{}: invalid manifest", manifest_path.display()))?;
        let derived = manifest.network_id();
        let derived_hex = derived.to_string();
        let derived_bare = derived_hex.strip_prefix("0x").unwrap_or(&derived_hex);
        if derived_bare != pinned {
            bail!(
                "network_id mismatch: {} hashes to {derived}, but the context pins 0x{pinned} — \
                 the artifact set is not the network it claims to be",
                manifest_path.display(),
            );
        }
    }

    let audit = audit_founding(&dir, args.record.as_deref())?;
    println!("network_id: {}", audit.manifest.network_id());
    println!(
        "{} founding record(s) re-verified offline against the policy the manifest pins, and \
         seated in its validator set: {}",
        audit.verified.len(),
        audit.verified.join(", ")
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use clap::Parser;
    use seismic_tee_common::network_dir::{
        MANIFEST_FILENAME, POLICY_FILENAME, SUMMIT_GENESIS_FILENAME,
    };
    use seismic_tee_common::test_support::manifest_pinning;

    use super::*;
    use crate::founding::FoundingRecord;
    use crate::founding::tests::{
        NODE_KEY_1, NODE_KEY_2, consensus_key, network_dir, record, write, write_harvest,
    };

    /// A policy the verifier parses: a case that reaches the replay must fail
    /// on the archive, never on the policy.
    const POLICY: &[u8] = br#"[{"attestation_type": "azure-tdx", "measurement_id": "img.vhd",
        "measurements": {"pcr4": {"expected_any":
        ["d57063c0669599b885c43a0683436a3463ad49513ddb3996e6fc96040508fd8e"]}}}]"#;

    /// The fixture manifest's namespace.
    const NAMESPACE: &str = "seismic-devnet-3";

    /// A completed summit genesis seating `validators`, as summit's emitter
    /// renders the array-of-tables.
    fn summit_genesis(namespace: &str, validators: &[(&str, &str)]) -> String {
        let mut text = format!(
            "eth_genesis_hash = \"0x{}\"\nnamespace = {namespace:?}\n",
            "12".repeat(32)
        );
        for (node, consensus_byte) in validators {
            text.push_str(&format!(
                "\n[[validators]]\nnode_public_key = {node:?}\nconsensus_public_key = {:?}\n\
                 ip_address = \"203.0.113.7:18551\"\nwithdrawal_credentials = \"0x{}\"\n",
                consensus_key(consensus_byte),
                "01".repeat(20)
            ));
        }
        text
    }

    fn records(entries: &[(&str, &str, &str)]) -> FoundingRecords {
        entries
            .iter()
            .map(|(name, node, consensus_byte)| {
                (
                    name.to_string(),
                    FoundingRecord {
                        node_public_key: node.to_string(),
                        consensus_public_key: consensus_key(consensus_byte),
                        document: record(node, consensus_byte),
                    },
                )
            })
            .collect()
    }

    /// A committed network directory as `assemble` leaves it, except that
    /// its records are bare harvest records rather than whole founding
    /// archives: every check before the replay passes, and the replay fails
    /// closed on the first record.
    fn committed_dir() -> (tempfile::TempDir, NetworkDir) {
        let (tmp, dir) = network_dir();
        write(
            &dir,
            Path::new(MANIFEST_FILENAME),
            std::str::from_utf8(&manifest_pinning(POLICY)).unwrap(),
        );
        write(
            &dir,
            Path::new(POLICY_FILENAME),
            std::str::from_utf8(POLICY).unwrap(),
        );
        write(
            &dir,
            Path::new(SUMMIT_GENESIS_FILENAME),
            &summit_genesis(NAMESPACE, &[(NODE_KEY_1, "cc"), (NODE_KEY_2, "dd")]),
        );
        write_harvest(&dir, "node-1", &record(NODE_KEY_1, "cc"));
        write_harvest(&dir, "node-2", &record(NODE_KEY_2, "dd"));
        (tmp, dir)
    }

    fn failure(dir: &NetworkDir, record: Option<&str>) -> String {
        format!("{:?}", audit_founding(dir, record).unwrap_err())
    }

    #[derive(Parser)]
    struct Probe {
        #[command(flatten)]
        args: VerifyFoundingArgs,
    }

    fn parse(argv: &[&str]) -> VerifyFoundingArgs {
        Probe::try_parse_from(std::iter::once(&"probe").chain(argv))
            .expect("well-formed argv")
            .args
    }

    #[test]
    fn the_directory_is_positional_and_record_narrows() {
        let args = parse(&["tee/networks/devnet-3"]);
        assert_eq!(
            args.dir.dir.as_deref(),
            Some(Path::new("tee/networks/devnet-3"))
        );
        assert_eq!(args.record, None);

        let args = parse(&["n", "--record", "n-2"]);
        assert_eq!(args.record.as_deref(), Some("n-2"));
    }

    /// The pinned set is read as summit emits it; an authored input, which
    /// carries no set, and a misspelled key are each named.
    #[test]
    fn the_pinned_set_is_read_from_the_completed_genesis() {
        let genesis = summit_genesis("testnet-1", &[(NODE_KEY_1, "cc"), (NODE_KEY_2, "dd")]);
        let keys = pinned_validator_keys(genesis.as_bytes()).unwrap();
        assert_eq!(
            keys,
            [
                (NODE_KEY_1.to_string(), consensus_key("cc")),
                (NODE_KEY_2.to_string(), consensus_key("dd")),
            ]
        );

        let err = pinned_validator_keys(b"namespace = \"testnet-1\"\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("no validators array"), "{err}");

        let uppercase = genesis.replacen(NODE_KEY_1, &NODE_KEY_1.to_uppercase(), 1);
        let err = pinned_validator_keys(uppercase.as_bytes())
            .unwrap_err()
            .to_string();
        assert!(err.contains("validators[0].node_public_key"), "{err}");

        let err = pinned_validator_keys(b"namespace = [")
            .unwrap_err()
            .to_string();
        assert!(err.contains("not valid TOML"), "{err}");
    }

    /// Both directions are gates: a record the genesis does not seat, and a
    /// seat no record vouches for. A consensus key that differs is as much a
    /// mismatch as a node key that does.
    #[test]
    fn the_archive_and_the_pinned_set_must_be_the_same_founding() {
        let pinned = vec![
            (NODE_KEY_1.to_string(), consensus_key("cc")),
            (NODE_KEY_2.to_string(), consensus_key("dd")),
        ];
        let whole = records(&[("node-1", NODE_KEY_1, "cc"), ("node-2", NODE_KEY_2, "dd")]);
        check_records_are_the_pinned_set(&whole, &pinned, false).unwrap();

        let err = check_records_are_the_pinned_set(
            &records(&[("node-1", NODE_KEY_1, "cc")]),
            &pinned,
            false,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("no archived quote vouches for"), "{err}");
        assert!(err.contains(&NODE_KEY_2[..8]), "{err}");

        let err = check_records_are_the_pinned_set(
            &records(&[("node-1", NODE_KEY_1, "cc"), ("node-2", NODE_KEY_2, "ee")]),
            &pinned,
            false,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("does not seat: node-2"), "{err}");
        assert!(err.contains("not the same founding"), "{err}");

        let err = check_records_are_the_pinned_set(
            &records(&[("node-3", "ff".repeat(32).as_str(), "cc")]),
            &pinned,
            true,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("does not seat: node-3"), "{err}");
    }

    /// A partial archive can only ask whether its own records are seated.
    #[test]
    fn a_partial_archive_asks_only_its_own_direction() {
        let pinned = vec![
            (NODE_KEY_1.to_string(), consensus_key("cc")),
            (NODE_KEY_2.to_string(), consensus_key("dd")),
        ];
        let one = records(&[("node-1", NODE_KEY_1, "cc")]);
        check_records_are_the_pinned_set(&one, &pinned, true).unwrap();
        assert!(check_records_are_the_pinned_set(&one, &pinned, false).is_err());
    }

    /// Every artifact is checked against the manifest before a quote is
    /// replayed, and each failure names its file.
    #[test]
    fn the_artifact_set_is_checked_before_any_replay() {
        let (_tmp, dir) = committed_dir();

        // The whole set reaches the replay, which fails closed on the first
        // record, a bare record with no bundle — so everything before it
        // passed.
        let err = failure(&dir, None);
        assert!(err.contains("node-1:"), "{err}");
        assert!(err.contains("is not a founding archive"), "{err}");

        let policy = std::fs::read(dir.policy()).unwrap();
        std::fs::write(dir.policy(), [policy.as_slice(), b"\n"].concat()).unwrap();
        let err = failure(&dir, None);
        assert!(err.contains("bootstrap_policy_hash mismatch"), "{err}");
        assert!(err.contains("not one artifact set"), "{err}");
        std::fs::write(dir.policy(), &policy).unwrap();

        std::fs::remove_file(dir.policy()).unwrap();
        let err = failure(&dir, None);
        assert!(err.contains(POLICY_FILENAME), "{err}");
        assert!(err.contains("not found"), "{err}");
        std::fs::write(dir.policy(), &policy).unwrap();

        let genesis = std::fs::read_to_string(dir.summit_genesis()).unwrap();
        std::fs::write(
            dir.summit_genesis(),
            genesis.replacen(NAMESPACE, "other-net", 1),
        )
        .unwrap();
        let err = failure(&dir, None);
        assert!(err.contains("namespace \"other-net\""), "{err}");
        std::fs::write(dir.summit_genesis(), &genesis).unwrap();

        std::fs::remove_file(dir.manifest()).unwrap();
        let err = failure(&dir, None);
        assert!(err.contains(MANIFEST_FILENAME), "{err}");
        assert!(err.contains("not an assembled network directory"), "{err}");
    }

    /// The archive and the seated set must be one founding, in both
    /// directions.
    #[test]
    fn the_archive_must_be_the_seated_set() {
        let (_tmp, dir) = committed_dir();

        std::fs::write(
            dir.summit_genesis(),
            summit_genesis(NAMESPACE, &[(NODE_KEY_1, "cc")]),
        )
        .unwrap();
        let err = failure(&dir, None);
        assert!(err.contains("does not seat: node-2"), "{err}");
        assert!(err.contains(SUMMIT_GENESIS_FILENAME), "{err}");

        std::fs::write(
            dir.summit_genesis(),
            summit_genesis(
                NAMESPACE,
                &[
                    (NODE_KEY_1, "cc"),
                    (NODE_KEY_2, "dd"),
                    ("ee".repeat(32).as_str(), "ee"),
                ],
            ),
        )
        .unwrap();
        let err = failure(&dir, None);
        assert!(err.contains("no archived quote vouches for"), "{err}");
    }

    /// `--record` narrows the replay to one node and relaxes the set check to
    /// that node's seat; an unknown name lists what the archive holds.
    #[test]
    fn record_narrows_to_a_partial_archive() {
        let (_tmp, dir) = committed_dir();

        let err = failure(&dir, Some("node-2"));
        assert!(err.contains("node-2:"), "{err}");
        assert!(err.contains("is not a founding archive"), "{err}");
        assert!(!err.contains("node-1"), "{err}");

        // The rest of the seated set need not be vouched for by one record.
        std::fs::write(
            dir.summit_genesis(),
            summit_genesis(
                NAMESPACE,
                &[
                    (NODE_KEY_1, "cc"),
                    (NODE_KEY_2, "dd"),
                    ("ee".repeat(32).as_str(), "ee"),
                ],
            ),
        )
        .unwrap();
        let err = failure(&dir, Some("node-2"));
        assert!(err.contains("is not a founding archive"), "{err}");

        let err = failure(&dir, Some("node-9"));
        assert!(err.contains("no record \"node-9\""), "{err}");
        assert!(err.contains("node-1, node-2"), "{err}");
    }

    #[tokio::test]
    async fn a_missing_directory_is_named() {
        let err = format!(
            "{:?}",
            run(parse(&["/absent/network-dir"])).await.unwrap_err()
        );
        assert!(err.contains("network directory not found"), "{err}");
        assert!(err.contains("/absent/network-dir"), "{err}");
    }

    /// A context pointed at by `--config`, pinning `devnet-3` to
    /// `network_id`. Every case below passes `--config` explicitly, so none
    /// of them ever reads a developer's real `~/.config/seismic/config.toml`.
    fn context_pinning(dir: &Path, network_id: &str) -> ContextArgs {
        let config_path = dir.join("config.toml");
        std::fs::write(
            &config_path,
            format!(
                "current = \"devnet-3\"\n\n[networks.devnet-3]\ndir = \"/x\"\nnetwork_id = \
                 {network_id:?}\n"
            ),
        )
        .unwrap();
        ContextArgs {
            context: None,
            config: Some(config_path),
        }
    }

    fn args_with_context(root: PathBuf, context: ContextArgs) -> VerifyFoundingArgs {
        VerifyFoundingArgs {
            dir: DirArgs {
                dir: Some(root),
                context,
            },
            record: None,
        }
    }

    /// A matching pin passes the check and reaches the replay — this
    /// directory's records are bare harvest records rather than whole
    /// archives (see `committed_dir`), so the replay itself fails, but never
    /// on a "network_id mismatch": the pin agreed.
    #[tokio::test]
    async fn a_matching_pin_passes() {
        let (tmp, dir) = committed_dir();
        let derived = Manifest::load(&dir.manifest())
            .unwrap()
            .network_id()
            .to_string();
        let hex = derived.strip_prefix("0x").unwrap();
        let context = context_pinning(tmp.path(), hex);
        let err = format!(
            "{:?}",
            run(args_with_context(dir.root().to_path_buf(), context))
                .await
                .unwrap_err()
        );
        assert!(!err.contains("network_id mismatch"), "{err}");
        assert!(err.contains("is not a founding archive"), "{err}");
    }

    /// A mismatched pin fails before the archive is ever replayed: this
    /// directory's records are bare harvest records rather than whole
    /// archives, so a failure that reached the replay would say so instead.
    #[tokio::test]
    async fn a_mismatched_pin_fails_before_any_record_is_replayed() {
        let (tmp, dir) = committed_dir();
        let context = context_pinning(tmp.path(), &"11".repeat(32));
        let err = format!(
            "{:?}",
            run(args_with_context(dir.root().to_path_buf(), context))
                .await
                .unwrap_err()
        );
        assert!(err.contains("network_id mismatch"), "{err}");
        assert!(err.contains(&format!("0x{}", "11".repeat(32))), "{err}");
        assert!(!err.contains("is not a founding archive"), "{err}");
    }

    /// No context selected at all: the audit proceeds exactly as it does
    /// with no pin configured.
    #[tokio::test]
    async fn no_pin_behaves_exactly_as_today() {
        let (tmp, dir) = committed_dir();
        let args = VerifyFoundingArgs {
            dir: DirArgs {
                dir: Some(dir.root().to_path_buf()),
                // Names no file: Context::load treats an absent file as an
                // empty config, with no `current` to select.
                context: ContextArgs {
                    context: None,
                    config: Some(tmp.path().join("absent-config.toml")),
                },
            },
            record: None,
        };
        let err = format!("{:?}", run(args).await.unwrap_err());
        assert!(err.contains("is not a founding archive"), "{err}");
    }
}
