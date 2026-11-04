//! The network manifest, held as the bytes it is identified by.
//!
//! `network_id = SHA-256(file bytes)`, so the manifest travels as opaque bytes
//! through every hop: deploy artifact → the config POST → tdx-init →
//! `/run/seismic/conf/`. Any byte change, even a reformat, names a different
//! network and fails loudly at the first binding check.
//!
//! That makes "hash the same bytes you parse" the rule the whole scheme rests
//! on, so this type holds both and derives the id itself — a caller can't pair
//! a parsed manifest with an id from somewhere else. The schema is the enclave
//! repo's [`seismic_network_manifest`], the same code every node parses the
//! manifest with, linked rather than mirrored.

use std::ops::Deref;
use std::path::Path;

use seismic_network_manifest::{NetworkId, NetworkManifestV1};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

/// `0x`-prefixed lowercase hex, as the manifest and the genesis spell words.
pub fn hex_0x(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// A manifest and the bytes it was parsed from, with the id those bytes derive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    bytes: Vec<u8>,
    network_id: NetworkId,
    parsed: NetworkManifestV1,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(|e| Error::read(path, e))?;
        Self::from_json_bytes(bytes).map_err(|source| Error::ManifestSchema {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn from_json_bytes(
        bytes: impl Into<Vec<u8>>,
    ) -> std::result::Result<Self, seismic_network_manifest::ManifestError> {
        let bytes = bytes.into();
        Ok(Self {
            network_id: NetworkId::from_manifest_bytes(&bytes),
            parsed: NetworkManifestV1::from_json_bytes(&bytes)?,
            bytes,
        })
    }

    /// The exact bytes to deliver onward. Never re-serialize the parsed value:
    /// that is a different document and a different network.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn network_id(&self) -> NetworkId {
        self.network_id
    }

    // The genesis gates are tdx-init-config's (`check_reth_genesis`,
    // `check_summit_genesis`, and `InitConfig::validate` over a whole config):
    // the same code tdx-init runs on the POST, linked rather than mirrored.

    /// The policy document is the one this manifest commits to:
    /// `measurements.bootstrap_policy_hash == SHA-256(policy bytes)`.
    ///
    /// A byte hash, so it holds for the exact file — the same document the
    /// registry's genesis-seeded admission IDs were compiled from. Every
    /// consumer of a network's policy artifact runs this before use, because
    /// appraising a node against a policy this network never committed to
    /// proves nothing about joining it.
    pub fn check_policy(&self, policy: &[u8]) -> Result<()> {
        let computed: [u8; 32] = Sha256::digest(policy).into();
        if computed != self.measurements.bootstrap_policy_hash {
            // `0x`-prefixed lowercase, as the manifest spells its hashes.
            return Err(Error::gate(format!(
                "measurements.bootstrap_policy_hash mismatch: manifest has 0x{}, computed 0x{}",
                hex::encode(self.measurements.bootstrap_policy_hash),
                hex::encode(computed),
            )));
        }
        Ok(())
    }
}

/// Read the manifest's fields straight off the value.
impl Deref for Manifest {
    type Target = NetworkManifestV1;

    fn deref(&self) -> &Self::Target {
        &self.parsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verbatim `tee/networks/fixture-devnet/network-manifest.json`, a real
    /// founding's manifest (see tee/networks/README.md).
    const FIXTURE_DEVNET: &[u8] =
        include_bytes!("../../../networks/fixture-devnet/network-manifest.json");

    #[test]
    fn parses_the_committed_fixture_network() {
        let manifest = Manifest::from_json_bytes(FIXTURE_DEVNET).unwrap();

        assert_eq!(manifest.name, "fixture-devnet");
        assert_eq!(manifest.eth.chain_id, 5124);
        assert_eq!(manifest.summit.namespace, "fixture-devnet");
    }

    /// The digest is pinned rather than recomputed the way the constructor
    /// does it, so this catches the enclave crate changing how `network_id` is
    /// derived — the one drift that would silently rename every network.
    ///
    /// Recompute with:
    /// `sha256sum tee/networks/fixture-devnet/network-manifest.json`
    #[test]
    fn the_id_is_the_hash_of_the_file_bytes() {
        let manifest = Manifest::from_json_bytes(FIXTURE_DEVNET).unwrap();

        assert_eq!(manifest.bytes(), FIXTURE_DEVNET);
        assert_eq!(
            manifest.network_id().to_string(),
            "0x31f2ba7ac91a4a9a560c2a4bcd9cee348a76d980537448d85924a1507cc989fc"
        );
    }

    /// One trailing newline is a different network — the property the whole
    /// deliver-verbatim rule exists to protect.
    #[test]
    fn a_reformat_is_a_different_network() {
        let reformatted = [FIXTURE_DEVNET, b"\n"].concat();

        assert_ne!(
            Manifest::from_json_bytes(FIXTURE_DEVNET)
                .unwrap()
                .network_id(),
            Manifest::from_json_bytes(reformatted).unwrap().network_id(),
        );
    }

    /// The policy check is a byte hash: the committed fixture's policy passes
    /// as it is on disk, and one trailing newline fails it.
    #[test]
    fn the_policy_must_hash_to_the_manifests_bootstrap_policy_hash() {
        let manifest = Manifest::from_json_bytes(FIXTURE_DEVNET).unwrap();
        let policy =
            include_bytes!("../../../networks/fixture-devnet/measurement-policy-bootstrap.json");

        manifest.check_policy(policy).unwrap();

        let err = manifest
            .check_policy(&[policy.as_slice(), b"\n"].concat())
            .unwrap_err()
            .to_string();
        assert!(err.contains("bootstrap_policy_hash mismatch"), "{err}");
        assert!(err.contains("manifest has 0x"), "{err}");
        assert!(err.contains("computed 0x"), "{err}");
    }

    #[test]
    fn a_load_failure_names_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("network-manifest.json");
        std::fs::write(&path, br#"{"manifest_version": 1}"#).unwrap();

        let err = Manifest::load(&path).unwrap_err().to_string();
        assert!(err.contains("network-manifest.json"), "{err}");
    }
}
