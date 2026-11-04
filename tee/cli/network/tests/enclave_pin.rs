//! The pinned enclave crates agree with this repo's `MeasurementRegistry`
//! contract and its policy fixtures.
//!
//! The contract, its artifact and the Solidity fixture tests live here; the
//! admission compiler, the registry runtime-code pin and the Rust registry
//! interfaces live in the enclave repo, which this workspace pins by rev. The
//! dependency runs one way: these checks run only here, against the pinned
//! rev, and enclave CI never reads this repo. Each one fires on either side
//! moving — a PR that regenerates `contracts/artifacts/` or the fixtures, or
//! one that bumps the enclave pin to a rev built against another contract.
//! A contract change therefore lands as: this repo's PR goes red here, the
//! enclave PR re-pins the hash (and any interface or slot change), this PR
//! pins that enclave PR's head until green, the enclave PR merges, and this
//! PR re-pins to the merged commit and merges.
//!
//! Hermetic — every file is in this checkout — so it runs in the default
//! `cargo nextest run` under the required CI job.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use alloy_primitives::keccak256;
use seismic_measurement_admission::{
    genesis::REGISTRY_RUNTIME_CODE_HASH,
    registry_abi::IMeasurementRegistry::IMeasurementRegistryCalls,
};
use seismic_measurement_registry_client::MeasurementRegistry::MeasurementRegistryCalls;
use seismic_tee_network::gates;
use serde_json::Value;

/// A path from the repo root. This crate is tee/cli/network/.
fn repo_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(relative)
}

fn read(relative: &str) -> Vec<u8> {
    let path = repo_path(relative);
    fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn registry_artifact() -> Value {
    serde_json::from_slice(&read("contracts/artifacts/MeasurementRegistry.json"))
        .expect("MeasurementRegistry.json")
}

/// The artifact's `methodIdentifiers`, as selector hex -> signature. Each
/// selector is recomputed from its signature rather than trusting the
/// artifact's own consistency.
fn artifact_methods(artifact: &Value) -> BTreeMap<String, String> {
    let methods: BTreeMap<String, String> =
        serde_json::from_value(artifact["methodIdentifiers"].clone()).expect("methodIdentifiers");
    methods
        .into_iter()
        .map(|(signature, selector)| {
            assert_eq!(
                selector,
                hex::encode(&keccak256(&signature)[..4]),
                "artifact selector for `{signature}` is not the keccak of its signature"
            );
            (selector, signature)
        })
        .collect()
}

/// A `sol!` interface's function selectors, as hex.
fn rust_selectors(selectors: &[[u8; 4]]) -> BTreeSet<String> {
    selectors.iter().map(hex::encode).collect()
}

/// The ABI entry of the function with this signature.
fn abi_function<'a>(artifact: &'a Value, signature: &str) -> &'a Value {
    let (name, _) = signature.split_once('(').expect("signature");
    artifact["abi"]
        .as_array()
        .expect("abi")
        .iter()
        .filter(|item| item["type"] == "function" && item["name"] == name)
        .find(|item| {
            let inputs: Vec<&str> = item["inputs"]
                .as_array()
                .expect("inputs")
                .iter()
                .map(|input| input["type"].as_str().expect("input type"))
                .collect();
            format!("{name}({})", inputs.join(",")) == signature
        })
        .unwrap_or_else(|| panic!("artifact ABI has no `{signature}`"))
}

/// The admission crate's registry runtime-code pin — enforced against the
/// genesis alloc by the validation gates, and embedded in every compiled
/// report — is keccak256 of this repo's deployed bytecode.
#[test]
fn admission_crate_pins_this_repos_registry_runtime() {
    let artifact = registry_artifact();
    let runtime = artifact["deployedBytecode"]["object"]
        .as_str()
        .expect("deployedBytecode.object");
    let runtime = hex::decode(runtime.strip_prefix("0x").unwrap_or(runtime)).unwrap();
    assert_eq!(
        keccak256(&runtime),
        REGISTRY_RUNTIME_CODE_HASH,
        "the admission crate pins a different registry runtime than \
         contracts/artifacts/MeasurementRegistry.json"
    );
}

/// The admission crate's `IMeasurementRegistry` declares exactly the
/// contract's external functions: none missing, none extra.
#[test]
fn admission_crate_registry_interface_matches_this_repos_abi() {
    let declared = rust_selectors(IMeasurementRegistryCalls::SELECTORS);
    let methods = artifact_methods(&registry_artifact());
    let missing: Vec<&String> = methods
        .iter()
        .filter(|(selector, _)| !declared.contains(*selector))
        .map(|(_, signature)| signature)
        .collect();
    let extra: Vec<&String> = declared
        .iter()
        .filter(|selector| !methods.contains_key(*selector))
        .collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "the admission crate's IMeasurementRegistry differs from \
         contracts/artifacts/MeasurementRegistry.json: missing {missing:?}, \
         extra selectors {extra:?}"
    );
}

/// Every function of the enclave runtime's read binding exists in the
/// contract, as a read: the attestation service reaches it by `eth_call`.
#[test]
fn registry_client_binding_is_a_read_of_this_repos_abi() {
    let artifact = registry_artifact();
    let methods = artifact_methods(&artifact);
    for selector in rust_selectors(MeasurementRegistryCalls::SELECTORS) {
        let signature = methods.get(&selector).unwrap_or_else(|| {
            panic!(
                "the registry client calls selector {selector}, which is not in \
                 contracts/artifacts/MeasurementRegistry.json"
            )
        });
        assert_eq!(
            abi_function(&artifact, signature)["stateMutability"],
            "view",
            "the registry client calls `{signature}`, which is not a view"
        );
    }
    // The binding decodes `isAccepted`'s return as one bool.
    let outputs: Vec<&Value> = abi_function(&artifact, "isAccepted(bytes32)")["outputs"]
        .as_array()
        .expect("outputs")
        .iter()
        .map(|output| &output["type"])
        .collect();
    assert_eq!(outputs, ["bool"], "`isAccepted` no longer returns one bool");
}

/// The Solidity fixture tests load a policy document and its compiled report
/// (contracts/test/MeasurementRegistry.t.sol). The report must be what the
/// pinned compiler makes of the document, byte for byte, so the Solidity
/// tests exercise the storage a real genesis would carry.
#[test]
fn contract_fixture_report_is_the_pinned_compilers_output() {
    let compiled = gates::compile(&read("contracts/test/fixtures/measurement-policy-v1.json"))
        .expect("the fixture policy compiles");
    assert_eq!(
        compiled.to_json(),
        String::from_utf8(read(
            "contracts/test/fixtures/measurement-policy-v1.compiled.json"
        ))
        .unwrap(),
        "contracts/test/fixtures/measurement-policy-v1.compiled.json is not the \
         pinned compiler's output; regenerate it with `seismic-tee admission \
         compile contracts/test/fixtures/measurement-policy-v1.json`"
    );
}
