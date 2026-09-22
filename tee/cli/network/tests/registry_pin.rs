//! The admission crate's registry runtime-code pin matches this repo's
//! contract artifact.
//!
//! The pinned enclave crate `seismic-measurement-admission` carries keccak256
//! of the canonical MeasurementRegistry deployed bytecode; the validation
//! gates enforce that pin against the genesis alloc, so a stale pin already
//! fails assembly loudly. This is the early warning, and it fires on either
//! side moving: a PR that regenerates `contracts/artifacts/` without a pin
//! bump, or one that bumps the enclave pin to a rev built against other
//! bytecode. Hermetic — the artifact is in this checkout — so it runs in the
//! default `cargo nextest run` under the required CI job.

use std::path::Path;

use alloy_primitives::keccak256;
use seismic_measurement_admission::genesis::REGISTRY_RUNTIME_CODE_HASH;

#[test]
fn admission_crate_pins_this_repos_registry_runtime() {
    // This crate is tee/cli/network/; the artifact is at the repo root.
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../contracts/artifacts/MeasurementRegistry.json");
    let artifact: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let runtime = artifact["deployedBytecode"]["object"]
        .as_str()
        .expect("deployedBytecode.object");
    let runtime = hex::decode(runtime.strip_prefix("0x").unwrap_or(runtime)).unwrap();
    assert_eq!(
        keccak256(&runtime),
        REGISTRY_RUNTIME_CODE_HASH,
        "{}: the admission crate pins a different registry runtime",
        path.display()
    );
}
