//! The deploy-side validation gates over an artifact set, and the registry
//! injection they hold `assemble` to.
//!
//! Everything top-level in a network directory is hash-pinned by the manifest,
//! and every pin is re-derived here — with the sibling binary that owns the
//! derivation ([`Derivations`]), or the linked enclave crate that owns the
//! rule — and compared. A mismatch shows expected vs computed. This fails at
//! deploy, not at boot: tdx-init re-runs the structural half of these checks
//! over the embedded artifacts at POST time, and the nodes enforce the hash
//! commitments by simply not agreeing with each other.
//!
//! The registry gate is the one with teeth. Committed genesis files are
//! policy-free (canonical registry runtime, empty storage — failing closed):
//! the accepted admission IDs are a per-network fact, so `assemble` compiles
//! the network's policy document with `seismic-measurement-admission` and
//! writes the compiled `registry_genesis_storage` into the registry account
//! ([`inject_registry_genesis_storage`]). The gate then requires the account
//! to hold the canonical runtime code and exactly that storage — every
//! expected slot with the expected word, and no unexplained slots — so
//! `eth.genesis_hash` commits to the reviewed policy and nothing else.

use std::collections::BTreeMap;

use alloy_primitives::{Address, B256, U256, keccak256};
use anyhow::{Context as _, bail};
use seismic_measurement_admission::{CompileReport, compile_policy};
use seismic_tee_common::{Artifact, Manifest, NetworkDir};
use serde_json::Value;

use crate::shell_outs::Derivations;

/// Today's hardcoded summit BLS domain separator. Two chains sharing it can
/// cross-replay BLS signatures.
const SUMMIT_DEFAULT_NAMESPACE: &str = "_SUMMIT";

/// Compile a policy document into the admission IDs it admits and the
/// registry genesis storage seeding them. "The compiler accepts it" is the
/// whole document validation.
pub fn compile(policy: &[u8]) -> anyhow::Result<CompileReport> {
    let compiled = compile_policy(policy).context("compiling the measurement policy")?;
    Ok(CompileReport::new(&compiled))
}

/// `0x`-prefixed lowercase hex, as the manifest and the genesis spell words.
pub fn hex_0x(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Write the compile report's `registry_genesis_storage` verbatim into the
/// reth genesis's registry account, replacing any existing storage.
///
/// Replacement, not merge: the gate requires the account to hold exactly the
/// compiled map, and wholesale replacement keeps re-assembly idempotent when
/// the policy changes. Everything else in the document — key order included —
/// is left as authored, so a re-assembly diffs as the injection and nothing
/// else.
pub fn inject_registry_genesis_storage(
    genesis: &[u8],
    registry: Address,
    report: &CompileReport,
) -> anyhow::Result<Vec<u8>> {
    if report.registry_genesis_storage.is_empty() {
        bail!(
            "policy compile report carries no registry_genesis_storage; the linked admission \
             crate is not one that compiles a policy to genesis storage"
        );
    }
    let mut genesis: Value =
        serde_json::from_slice(genesis).context("reth genesis is not valid JSON")?;
    let Some(alloc) = genesis.get_mut("alloc").and_then(Value::as_object_mut) else {
        bail!("reth genesis has no alloc object");
    };
    let wanted = hex_0x(registry.as_slice());
    let matches: Vec<String> = alloc
        .keys()
        .filter(|k| k.to_lowercase() == wanted)
        .cloned()
        .collect();
    let key = match matches.as_slice() {
        [] => bail!("registry {wanted} is not in the reth genesis alloc"),
        [key] => key.clone(),
        _ => {
            bail!("reth genesis alloc lists registry {wanted} twice under different hex spellings")
        }
    };
    let Some(account) = alloc.get_mut(&key).and_then(Value::as_object_mut) else {
        bail!("registry {wanted} alloc entry is not an object");
    };
    let storage: serde_json::Map<String, Value> = report
        .registry_genesis_storage
        .iter()
        .map(|(slot, word)| {
            (
                hex_0x(slot.as_slice()),
                Value::String(hex_0x(word.as_slice())),
            )
        })
        .collect();
    account.insert("storage".to_string(), Value::Object(storage));
    let mut bytes = serde_json::to_vec_pretty(&genesis).context("re-rendering the reth genesis")?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Normalize a 256-bit storage slot/word to its canonical 32 bytes.
///
/// Genesis JSON accepts unpadded and mixed-case hex; the compile report is
/// already canonical. Comparing normalized words keeps the gate about values,
/// not formatting.
fn normalize_word(value: &Value, what: &str) -> anyhow::Result<B256> {
    let Some(text) = value.as_str() else {
        bail!("{what}: expected hex string, got {value}");
    };
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .unwrap_or(text);
    if digits.is_empty() {
        bail!("{what}: expected hex string, got {text:?}");
    }
    let word = U256::from_str_radix(digits, 16)
        .map_err(|_| anyhow::anyhow!("{what}: expected hex string, got {text:?}"))?;
    Ok(B256::from(word))
}

/// Exact registry-account gate: canonical runtime code and precisely the
/// compiled genesis storage — every expected slot present with the expected
/// word, and no unexplained slots.
fn validate_registry_account(
    account: &Value,
    address: &str,
    report: &CompileReport,
) -> anyhow::Result<()> {
    let code = account
        .get("code")
        .and_then(Value::as_str)
        .map(|c| c.strip_prefix("0x").unwrap_or(c))
        .and_then(|c| hex::decode(c).ok())
        .with_context(|| format!("registry {address} code is not valid hex"))?;
    let code_hash = keccak256(&code);
    if code_hash != report.registry_runtime_code_hash {
        bail!(
            "registry {address} code is not the canonical MeasurementRegistry runtime: keccak256 \
             is {}, the policy compiler pins {} (rebuild the genesis from the current contract \
             artifact)",
            hex_0x(code_hash.as_slice()),
            hex_0x(report.registry_runtime_code_hash.as_slice()),
        );
    }

    let expected: &BTreeMap<B256, B256> = &report.registry_genesis_storage;
    let raw_storage = match account.get("storage") {
        None => &serde_json::Map::new(),
        Some(Value::Object(storage)) => storage,
        Some(other) => bail!("registry {address} storage: expected an object, got {other}"),
    };
    let mut actual: BTreeMap<B256, B256> = BTreeMap::new();
    for (slot, word) in raw_storage {
        let slot = normalize_word(
            &Value::String(slot.clone()),
            &format!("registry {address} storage slot"),
        )?;
        let word = normalize_word(word, &format!("registry {address} storage value at {slot}"))?;
        if actual.insert(slot, word).is_some() {
            bail!(
                "registry {address} genesis storage lists the same slot twice under different \
                 hex spellings"
            );
        }
    }
    if actual.is_empty() {
        bail!(
            "registry {address} genesis storage is empty: the admission policy must be \
             genesis-pinned. Seed the account with the compiled registry_genesis_storage \
             (`seismic-tee admission compile measurement-policy-bootstrap.json`)"
        );
    }
    if &actual != expected {
        let mut problems = Vec::new();
        for (slot, word) in expected {
            match actual.get(slot) {
                None => problems.push(format!(
                    "slot {} missing (expected {})",
                    hex_0x(slot.as_slice()),
                    hex_0x(word.as_slice())
                )),
                Some(held) if held != word => problems.push(format!(
                    "slot {} holds {}, expected {}",
                    hex_0x(slot.as_slice()),
                    hex_0x(held.as_slice()),
                    hex_0x(word.as_slice())
                )),
                Some(_) => {}
            }
        }
        for (slot, word) in &actual {
            if !expected.contains_key(slot) {
                problems.push(format!(
                    "slot {} unexplained (value {})",
                    hex_0x(slot.as_slice()),
                    hex_0x(word.as_slice())
                ));
            }
        }
        bail!(
            "registry {address} genesis storage does not match the compiled policy \
             artifact:\n  {}",
            problems.join("\n  ")
        );
    }
    Ok(())
}

/// An artifact set as the gates read it: the manifest and the three files it
/// pins, each with the path it came from.
#[derive(Debug, Clone)]
pub struct ArtifactSet {
    pub manifest: Manifest,
    pub reth_genesis: Artifact,
    pub summit_genesis: Artifact,
    pub policy: Artifact,
}

impl ArtifactSet {
    /// The artifact set `assemble` wrote at the top of a network directory.
    pub fn load(dir: &NetworkDir) -> anyhow::Result<Self> {
        let manifest_path = dir.manifest();
        if !manifest_path.is_file() {
            bail!(
                "{} not found — no artifact set to check; derive one with `seismic-tee network \
                 assemble`",
                manifest_path.display()
            );
        }
        Ok(Self {
            manifest: Manifest::load(&manifest_path)?,
            reth_genesis: Artifact::read(&dir.reth_genesis())?,
            summit_genesis: Artifact::read(&dir.summit_genesis())?,
            policy: Artifact::read(&dir.policy())?,
        })
    }
}

/// Deploy-time cross-artifact gates: every mismatch shows expected vs
/// computed. Returns the warnings a passing set still earns.
pub async fn run_validation_gates(
    set: &ArtifactSet,
    derive: &impl Derivations,
) -> anyhow::Result<Vec<String>> {
    let manifest = &set.manifest;
    let mut warnings = Vec::new();

    // eth.chain_id == reth genesis config.chainId.
    manifest
        .check_reth_genesis(set.reth_genesis.bytes())
        .with_context(|| format!("reth genesis {}", set.reth_genesis.path().display()))?;
    let genesis: Value = serde_json::from_slice(set.reth_genesis.bytes())
        .context("reth genesis is not valid JSON")?;

    // eth.genesis_hash: recomputed offline by seismic-reth's own code.
    let computed = derive.reth_genesis_hash(set.reth_genesis.bytes()).await?;
    if computed != manifest.eth.genesis_hash {
        bail!(
            "eth.genesis_hash mismatch: manifest has {}, recomputed {} from {}",
            hex_0x(&manifest.eth.genesis_hash),
            hex_0x(&computed),
            set.reth_genesis.path().display(),
        );
    }

    // summit.genesis_config_digest: summit's own SSZ-domain digest over the
    // shipped genesis, not a byte hash of the file.
    let computed = derive
        .summit_config_digest(set.summit_genesis.bytes())
        .await?;
    if computed != manifest.summit.genesis_config_digest {
        bail!(
            "summit.genesis_config_digest mismatch: manifest has {}, recomputed {} from {}",
            hex_0x(&manifest.summit.genesis_config_digest),
            hex_0x(&computed),
            set.summit_genesis.path().display(),
        );
    }

    // The genesis's embedded eth_genesis_hash and namespace must match the
    // manifest fields (the namespace is duplicated into the manifest so
    // verifiers don't need to parse TOML).
    let summit_text = std::str::from_utf8(set.summit_genesis.bytes())
        .context("summit genesis is not valid TOML")?;
    let summit: toml::Table =
        toml::from_str(summit_text).context("summit genesis is not valid TOML")?;
    let genesis_eth_hash = summit.get("eth_genesis_hash").and_then(toml::Value::as_str);
    if genesis_eth_hash.map(str::to_lowercase) != Some(hex_0x(&manifest.eth.genesis_hash)) {
        bail!(
            "summit genesis eth_genesis_hash is {}, manifest has {}",
            genesis_eth_hash.map_or_else(|| "absent".to_string(), |h| format!("{h:?}")),
            hex_0x(&manifest.eth.genesis_hash),
        );
    }
    manifest
        .check_summit_genesis(set.summit_genesis.bytes())
        .with_context(|| format!("summit genesis {}", set.summit_genesis.path().display()))?;
    if manifest.summit.namespace == SUMMIT_DEFAULT_NAMESPACE {
        warnings.push(format!(
            "summit namespace is the hardcoded default '{SUMMIT_DEFAULT_NAMESPACE}'; two chains \
             running the same image can cross-replay BLS signatures"
        ));
    }

    manifest
        .check_policy(set.policy.bytes())
        .with_context(|| format!("policy {}", set.policy.path().display()))?;

    // Contract addresses must exist in the genesis alloc (with code).
    let alloc: BTreeMap<String, &Value> = genesis
        .get("alloc")
        .and_then(Value::as_object)
        .map(|alloc| {
            alloc
                .iter()
                .map(|(address, account)| (address.to_lowercase(), account))
                .collect()
        })
        .unwrap_or_default();
    let contracts = &manifest.measurements.contracts;
    let registry_address = hex_0x(&contracts.registry);
    for (role, address) in [
        ("registry", &registry_address),
        ("authority", &hex_0x(&contracts.authority)),
    ] {
        let Some(account) = alloc.get(address) else {
            bail!("measurements.contracts.{role} {address} is not in the reth genesis alloc");
        };
        if account
            .get("code")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            bail!("measurements.contracts.{role} {address} has no code in the reth genesis alloc");
        }
    }

    // Policy artifact <-> registry account consistency: compile the policy
    // with the linked admission crate and require the registry genesis
    // account to hold the canonical runtime code plus exactly the compiled
    // storage, so the genesis hash commits to the reviewed policy and nothing
    // else.
    let report = compile(set.policy.bytes())?;
    validate_registry_account(alloc[&registry_address], &registry_address, &report)?;
    Ok(warnings)
}

#[cfg(test)]
pub(crate) mod tests {
    use seismic_tee_common::network_dir::MANIFEST_FILENAME;
    use serde_json::json;

    use super::*;
    use crate::assemble::tests::{Fake, assemble_with, authored};
    use crate::assemble::write_artifact_set;

    /// The committed fixture network's reth genesis: the registry predeploy
    /// carries the canonical MeasurementRegistry runtime, so it is the one
    /// genesis these tests can hold to the real compiler's code-hash pin.
    pub(crate) const FIXTURE_RETH_GENESIS: &[u8] =
        include_bytes!("../../../networks/fixture-devnet/reth-genesis.json");
    /// The policy that fixture's registry storage was compiled from.
    pub(crate) const FIXTURE_POLICY: &[u8] =
        include_bytes!("../../../networks/fixture-devnet/measurement-policy-bootstrap.json");

    pub(crate) const REGISTRY: Address =
        alloy_primitives::address!("0x1000000000000000000000000000000000000001");

    /// A promoted policy other than the fixture's: same registers, another
    /// image, so it compiles to a different admission ID and policy hash.
    pub(crate) fn other_policy() -> Vec<u8> {
        let records = json!([{
            "attestation_type": "azure-tdx",
            "measurement_id": "other.vhd",
            "measurements": {
                "pcr4": {"expected_any": ["ab".repeat(32)]},
                "pcr9": {"expected_any": ["cd".repeat(32)]},
                "pcr11": {"expected_any": ["ef".repeat(32)]},
            },
        }]);
        let mut bytes = serde_json::to_vec_pretty(&records).unwrap();
        bytes.push(b'\n');
        bytes
    }

    fn registry_account(genesis: &[u8]) -> Value {
        let genesis: Value = serde_json::from_slice(genesis).unwrap();
        genesis["alloc"][&hex_0x(REGISTRY.as_slice())].clone()
    }

    /// Injection writes exactly the report's storage into the registry
    /// account, replaces whatever was there, and leaves the rest of the
    /// document — key order included — as authored.
    #[test]
    fn injection_replaces_the_registry_storage_and_nothing_else() {
        let report = compile(&other_policy()).unwrap();
        let injected =
            inject_registry_genesis_storage(FIXTURE_RETH_GENESIS, REGISTRY, &report).unwrap();

        let account = registry_account(&injected);
        let storage = account["storage"].as_object().unwrap();
        assert_eq!(storage.len(), report.registry_genesis_storage.len());
        for (slot, word) in &report.registry_genesis_storage {
            assert_eq!(storage[&hex_0x(slot.as_slice())], hex_0x(word.as_slice()));
        }
        // The fixture's own storage (compiled from its policy) is gone.
        assert_ne!(
            account["storage"],
            registry_account(FIXTURE_RETH_GENESIS)["storage"]
        );

        // Everything else survives, in the authored order.
        let mut before: Value = serde_json::from_slice(FIXTURE_RETH_GENESIS).unwrap();
        let mut after: Value = serde_json::from_slice(&injected).unwrap();
        for genesis in [&mut before, &mut after] {
            genesis["alloc"][&hex_0x(REGISTRY.as_slice())]
                .as_object_mut()
                .unwrap()
                .remove("storage");
        }
        assert_eq!(before, after);
        let keys = |g: &Value| g.as_object().unwrap().keys().cloned().collect::<Vec<_>>();
        assert_eq!(keys(&before), keys(&after));
        assert!(injected.ends_with(b"}\n"));

        // Re-injecting the fixture's own policy reproduces its committed
        // registry storage.
        let report = compile(FIXTURE_POLICY).unwrap();
        let reinjected =
            inject_registry_genesis_storage(FIXTURE_RETH_GENESIS, REGISTRY, &report).unwrap();
        assert_eq!(
            registry_account(&reinjected)["storage"],
            registry_account(FIXTURE_RETH_GENESIS)["storage"]
        );
    }

    #[test]
    fn injection_needs_the_registry_account_once_in_the_alloc() {
        let report = compile(FIXTURE_POLICY).unwrap();
        let err = inject_registry_genesis_storage(
            br#"{"config": {"chainId": 1}, "alloc": {}}"#,
            REGISTRY,
            &report,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("is not in the reth genesis alloc"), "{err}");

        // The address has no hex letters, so the second spelling differs in
        // its prefix.
        let twice = json!({"alloc": {
            "0x1000000000000000000000000000000000000001": {"code": "0x00"},
            "0X1000000000000000000000000000000000000001": {"code": "0x00"},
        }});
        let err = inject_registry_genesis_storage(twice.to_string().as_bytes(), REGISTRY, &report)
            .unwrap_err()
            .to_string();
        assert!(err.contains("twice under different hex spellings"), "{err}");

        let err = inject_registry_genesis_storage(b"[]", REGISTRY, &report)
            .unwrap_err()
            .to_string();
        assert!(err.contains("no alloc object"), "{err}");
    }

    /// The registry gate compares words, not spellings: unpadded and
    /// mixed-case slots equal to the report's pass, and every way the account
    /// can disagree is named slot by slot.
    #[test]
    fn the_registry_gate_is_exact_over_normalized_words() {
        let report = compile(FIXTURE_POLICY).unwrap();
        let address = hex_0x(REGISTRY.as_slice());
        let canonical = registry_account(FIXTURE_RETH_GENESIS);
        validate_registry_account(&canonical, &address, &report).unwrap();

        // Respelled: uppercase digits, `0x0` shorthand for a zero-padded word.
        let mut respelled = canonical.clone();
        let storage: serde_json::Map<String, Value> = canonical["storage"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(slot, word)| {
                let word = word.as_str().unwrap();
                let short = format!(
                    "0x{}",
                    word.trim_start_matches("0x").trim_start_matches('0')
                );
                (
                    slot.to_uppercase().replace("0X", "0x"),
                    Value::String(short),
                )
            })
            .collect();
        respelled["storage"] = Value::Object(storage);
        validate_registry_account(&respelled, &address, &report).unwrap();

        // Empty storage: the fail-closed genesis, never shippable.
        let mut empty = canonical.clone();
        empty.as_object_mut().unwrap().remove("storage");
        let err = validate_registry_account(&empty, &address, &report)
            .unwrap_err()
            .to_string();
        assert!(err.contains("genesis storage is empty"), "{err}");

        // A missing slot, an unexplained slot, and a wrong word are each named.
        let (first_slot, first_word) = {
            let storage = canonical["storage"].as_object().unwrap();
            let (s, w) = storage.iter().next().unwrap();
            (s.clone(), w.as_str().unwrap().to_string())
        };
        let mut tampered = canonical.clone();
        let storage = tampered["storage"].as_object_mut().unwrap();
        storage.remove(&first_slot);
        storage.insert(format!("0x{}", "77".repeat(32)), json!("0x1"));
        let err = validate_registry_account(&tampered, &address, &report)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(&format!(
                "slot {first_slot} missing (expected {first_word})"
            )),
            "{err}"
        );
        assert!(
            err.contains(&format!("slot 0x{} unexplained", "77".repeat(32))),
            "{err}"
        );

        let mut wrong = canonical.clone();
        wrong["storage"][&first_slot] = json!(format!("0x{}", "99".repeat(32)));
        let err = validate_registry_account(&wrong, &address, &report)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(&format!("slot {first_slot} holds 0x{}", "99".repeat(32))),
            "{err}"
        );

        // Two spellings of one slot.
        let mut doubled = canonical.clone();
        doubled["storage"][first_slot.to_uppercase().replace("0X", "0x")] = json!(first_word);
        let err = validate_registry_account(&doubled, &address, &report)
            .unwrap_err()
            .to_string();
        assert!(err.contains("same slot twice"), "{err}");

        // Code that is not the canonical runtime.
        let mut other_code = canonical.clone();
        other_code["code"] = json!("0x600160005500");
        let err = validate_registry_account(&other_code, &address, &report)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("not the canonical MeasurementRegistry runtime"),
            "{err}"
        );
    }

    /// An assembled network directory on disk, ready to be tampered with:
    /// the gates below run over what `assemble` wrote, as they do again
    /// under `assemble --check` and over the committed directories in the
    /// drift suite.
    async fn assembled_dir() -> (tempfile::TempDir, NetworkDir) {
        let authored = authored();
        let assembled = assemble_with(&authored, FIXTURE_POLICY, &Fake::default())
            .await
            .unwrap();
        let dir = NetworkDir::new(authored.dir.path().join("net"));
        write_artifact_set(&dir, &assembled, false).unwrap();
        (authored.dir, dir)
    }

    async fn gates_over(dir: &NetworkDir) -> anyhow::Result<Vec<String>> {
        run_validation_gates(&ArtifactSet::load(dir).unwrap(), &Fake::default()).await
    }

    fn failure(result: anyhow::Result<Vec<String>>) -> String {
        format!("{:?}", result.unwrap_err())
    }

    fn edit_genesis(dir: &NetworkDir, edit: impl FnOnce(&mut Value)) {
        let mut genesis: Value =
            serde_json::from_slice(&std::fs::read(dir.reth_genesis()).unwrap()).unwrap();
        edit(&mut genesis);
        std::fs::write(dir.reth_genesis(), genesis.to_string()).unwrap();
    }

    #[tokio::test]
    async fn an_assembled_directory_passes_and_a_missing_one_is_named() {
        let (_tmp, dir) = assembled_dir().await;
        assert!(gates_over(&dir).await.unwrap().is_empty());

        let err = ArtifactSet::load(&NetworkDir::new(_tmp.path().join("absent")))
            .unwrap_err()
            .to_string();
        assert!(err.contains(MANIFEST_FILENAME), "{err}");
        assert!(err.contains("seismic-tee network assemble"), "{err}");
    }

    /// Every pin is re-derived: a genesis hash, a digest, a policy hash or a
    /// summit field that disagrees with the manifest fails by name.
    #[tokio::test]
    async fn each_pin_is_a_gate() {
        let (_tmp, dir) = assembled_dir().await;

        let err = failure(
            run_validation_gates(
                &ArtifactSet::load(&dir).unwrap(),
                &Fake {
                    eth_hash: [0x34; 32],
                    digest: None,
                },
            )
            .await,
        );
        assert!(err.contains("eth.genesis_hash mismatch"), "{err}");
        assert!(
            err.contains(&format!("recomputed 0x{}", "34".repeat(32))),
            "{err}"
        );

        // The digest is over the summit genesis bytes: any edit breaks it.
        let summit = std::fs::read_to_string(dir.summit_genesis()).unwrap();
        std::fs::write(dir.summit_genesis(), format!("{summit}# tampered\n")).unwrap();
        let err = failure(gates_over(&dir).await);
        assert!(
            err.contains("summit.genesis_config_digest mismatch"),
            "{err}"
        );
        std::fs::write(dir.summit_genesis(), &summit).unwrap();

        // The policy hash is a byte hash.
        let policy = std::fs::read(dir.policy()).unwrap();
        std::fs::write(dir.policy(), [policy.as_slice(), b"\n"].concat()).unwrap();
        let err = failure(gates_over(&dir).await);
        assert!(err.contains("bootstrap_policy_hash mismatch"), "{err}");
        std::fs::write(dir.policy(), &policy).unwrap();

        edit_genesis(&dir, |g| g["config"]["chainId"] = json!(9999));
        let err = failure(gates_over(&dir).await);
        assert!(err.contains("chainId 9999"), "{err}");
    }

    /// The summit genesis's own copies of the derived values must agree with
    /// the manifest. The digest gate comes first and is content-derived here,
    /// so the fake is pinned to the manifest's digest to reach the field
    /// gates behind it.
    #[tokio::test]
    async fn the_summit_genesis_fields_must_match_the_manifest() {
        let (_tmp, dir) = assembled_dir().await;
        let set = ArtifactSet::load(&dir).unwrap();
        let pinned = Fake {
            eth_hash: [0x12; 32],
            digest: Some(set.manifest.summit.genesis_config_digest),
        };
        let original = std::fs::read_to_string(dir.summit_genesis()).unwrap();

        let wrong_hash = original.replacen(
            &format!("eth_genesis_hash = \"0x{}\"", "12".repeat(32)),
            &format!("eth_genesis_hash = \"0x{}\"", "56".repeat(32)),
            1,
        );
        assert_ne!(wrong_hash, original);
        std::fs::write(dir.summit_genesis(), &wrong_hash).unwrap();
        let err = failure(run_validation_gates(&ArtifactSet::load(&dir).unwrap(), &pinned).await);
        assert!(err.contains("summit genesis eth_genesis_hash is"), "{err}");
        assert!(
            err.contains(&format!("manifest has 0x{}", "12".repeat(32))),
            "{err}"
        );

        let wrong_namespace =
            original.replacen("namespace = \"testnet-1\"", "namespace = \"other\"", 1);
        assert_ne!(wrong_namespace, original);
        std::fs::write(dir.summit_genesis(), &wrong_namespace).unwrap();
        let err = failure(run_validation_gates(&ArtifactSet::load(&dir).unwrap(), &pinned).await);
        assert!(err.contains("namespace \"other\""), "{err}");
        assert!(err.contains("summit.namespace \"testnet-1\""), "{err}");
    }

    /// The gates re-run over the on-disk genesis catch every way the registry
    /// account can disagree with the policy it was compiled from.
    #[tokio::test]
    async fn the_registry_account_is_held_to_the_policy() {
        let (_tmp, dir) = assembled_dir().await;
        let registry = hex_0x(REGISTRY.as_slice());

        edit_genesis(&dir, |g| {
            g["alloc"][&registry]["storage"] = json!({});
        });
        let err = failure(gates_over(&dir).await);
        assert!(err.contains("genesis storage is empty"), "{err}");

        // A genesis assembled around another policy: same slots, other words.
        let other = compile(&other_policy()).unwrap();
        edit_genesis(&dir, |g| {
            let storage: serde_json::Map<String, Value> = other
                .registry_genesis_storage
                .iter()
                .map(|(s, w)| (hex_0x(s.as_slice()), json!(hex_0x(w.as_slice()))))
                .collect();
            g["alloc"][&registry]["storage"] = json!(storage);
        });
        let err = failure(gates_over(&dir).await);
        assert!(
            err.contains("does not match the compiled policy artifact"),
            "{err}"
        );
        assert!(err.contains("missing"), "{err}");
        assert!(err.contains("unexplained"), "{err}");

        edit_genesis(&dir, |g| {
            g["alloc"][&registry]["code"] = json!("0x600160005500");
        });
        let err = failure(gates_over(&dir).await);
        assert!(
            err.contains("not the canonical MeasurementRegistry runtime"),
            "{err}"
        );

        edit_genesis(&dir, |g| {
            g["alloc"].as_object_mut().unwrap().remove(&registry);
        });
        let err = failure(gates_over(&dir).await);
        assert!(err.contains("measurements.contracts.registry"), "{err}");
        assert!(err.contains("not in the reth genesis alloc"), "{err}");
    }

    #[tokio::test]
    async fn a_contract_without_code_is_rejected() {
        let (_tmp, dir) = assembled_dir().await;
        edit_genesis(&dir, |g| {
            g["alloc"]["0x1000000000000000000000000000000000000002"]["code"] = json!("");
        });
        let err = failure(gates_over(&dir).await);
        assert!(err.contains("measurements.contracts.authority"), "{err}");
        assert!(err.contains("has no code"), "{err}");
    }

    #[test]
    fn words_normalize_from_any_hex_spelling() {
        let one = B256::from(U256::from(1));
        assert_eq!(normalize_word(&json!("0x1"), "w").unwrap(), one);
        assert_eq!(normalize_word(&json!("0x01"), "w").unwrap(), one);
        assert_eq!(normalize_word(&json!("1"), "w").unwrap(), one);
        assert_eq!(
            normalize_word(&json!(format!("0x{}01", "00".repeat(31))), "w").unwrap(),
            one
        );
        assert!(normalize_word(&json!("0xzz"), "w").is_err());
        assert!(normalize_word(&json!(""), "w").is_err());
        assert!(normalize_word(&json!(5), "w").is_err());
        // Too wide for a word.
        assert!(normalize_word(&json!(format!("0x1{}", "00".repeat(32))), "w").is_err());
    }
}
