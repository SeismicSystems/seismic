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

    // The gates over the artifacts a manifest pins. Each is a cheap structural
    // check the manifest duplicates a field for, run deploy-side so a mistake
    // fails before a POST rather than as tdx-init's 400 (or, for the policy,
    // before a node is appraised against a document this network never
    // committed to). The *hash* commitments — `eth.genesis_hash`,
    // `summit.genesis_config_digest` — need reth's and summit's own code and
    // are enforced by the founder's `assemble` (and `assemble --check`); these are the
    // fields tdx-init itself checks at POST time.

    /// The reth genesis is valid JSON whose `config.chainId` is this
    /// manifest's `eth.chain_id` — the mirror of tdx-init's POST-time check,
    /// so a genesis other than the one the manifest was assembled from fails
    /// the POST build rather than booting a forked node.
    pub fn check_reth_genesis(&self, genesis: &[u8]) -> Result<()> {
        let genesis: serde_json::Value = serde_json::from_slice(genesis)
            .map_err(|e| Error::gate(format!("reth genesis is not valid JSON: {e}")))?;
        let chain_id = genesis.get("config").and_then(|c| c.get("chainId"));
        let Some(chain_id) = chain_id.and_then(serde_json::Value::as_u64) else {
            return Err(Error::gate(format!(
                "reth genesis config.chainId is {}, not an int",
                chain_id.map_or_else(|| "absent".to_string(), ToString::to_string),
            )));
        };
        if chain_id != self.eth.chain_id {
            return Err(Error::gate(format!(
                "reth genesis config.chainId {chain_id} does not match the manifest's \
                 eth.chain_id {}",
                self.eth.chain_id,
            )));
        }
        Ok(())
    }

    /// The summit genesis is valid TOML whose `namespace` is this manifest's
    /// `summit.namespace` — the mirror of tdx-init's POST-time check. A
    /// delivered copy may differ from the artifact-set file in its validator
    /// IPs (topology, not identity), which is why the check is on the field
    /// and not on the bytes.
    pub fn check_summit_genesis(&self, genesis: &[u8]) -> Result<()> {
        let text = std::str::from_utf8(genesis)
            .map_err(|e| Error::gate(format!("summit genesis is not valid TOML: {e}")))?;
        let genesis: toml::Table = toml::from_str(text)
            .map_err(|e| Error::gate(format!("summit genesis is not valid TOML: {e}")))?;
        let namespace = genesis.get("namespace");
        let Some(namespace) = namespace.and_then(toml::Value::as_str) else {
            return Err(Error::gate(format!(
                "summit genesis namespace is {}, not a string",
                namespace.map_or_else(|| "absent".to_string(), ToString::to_string),
            )));
        };
        if namespace != self.summit.namespace {
            return Err(Error::gate(format!(
                "summit genesis namespace {namespace:?} does not match the manifest's \
                 summit.namespace {:?}",
                self.summit.namespace,
            )));
        }
        Ok(())
    }

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

    /// Verbatim `tee/networks/fixture-devnet/network-manifest.json`. The
    /// fixture is a real founding, assembled under the throwaway name its
    /// cohort was founded as; the name is part of the hashed bytes, so it
    /// stays (see the fixture's README).
    const FIXTURE_DEVNET: &[u8] =
        include_bytes!("../../../networks/fixture-devnet/network-manifest.json");

    #[test]
    fn parses_the_committed_fixture_network() {
        let manifest = Manifest::from_json_bytes(FIXTURE_DEVNET).unwrap();

        assert_eq!(manifest.name, "tmp-devnet-1");
        assert_eq!(manifest.eth.chain_id, 5124);
        assert_eq!(manifest.summit.namespace, "tmp-devnet-1");
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
            "0x6dc6adff3fe0aa9278dfbb3a1236af1ff855e32f6ceb754b8be8390879e06595"
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

    /// The reth genesis check is on `config.chainId` alone: the hash
    /// commitment is `assemble`'s to enforce, and tdx-init checks this field.
    #[test]
    fn the_reth_genesis_must_carry_the_manifests_chain_id() {
        let manifest = Manifest::from_json_bytes(FIXTURE_DEVNET).unwrap();

        manifest
            .check_reth_genesis(br#"{"config": {"chainId": 5124}, "alloc": {}}"#)
            .unwrap();

        let err = manifest
            .check_reth_genesis(br#"{"config": {"chainId": 9999}}"#)
            .unwrap_err()
            .to_string();
        assert!(err.contains("chainId 9999"), "{err}");
        assert!(err.contains("eth.chain_id 5124"), "{err}");

        let err = manifest
            .check_reth_genesis(br#"{"config": {"chainId": "5124"}}"#)
            .unwrap_err()
            .to_string();
        assert!(err.contains("not an int"), "{err}");

        let err = manifest.check_reth_genesis(b"{").unwrap_err().to_string();
        assert!(err.contains("not valid JSON"), "{err}");
    }

    /// The summit genesis check is on `namespace` alone, so a delivered copy
    /// with current validator IPs spliced in still passes.
    #[test]
    fn the_summit_genesis_must_carry_the_manifests_namespace() {
        let manifest = Manifest::from_json_bytes(FIXTURE_DEVNET).unwrap();

        manifest
            .check_summit_genesis(b"namespace = \"tmp-devnet-1\"\nvalidators = []\n")
            .unwrap();

        let err = manifest
            .check_summit_genesis(b"namespace = \"other-net\"\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("\"other-net\""), "{err}");
        assert!(err.contains("summit.namespace \"tmp-devnet-1\""), "{err}");

        let err = manifest
            .check_summit_genesis(b"validators = []\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("absent, not a string"), "{err}");

        let err = manifest
            .check_summit_genesis(b"namespace = [")
            .unwrap_err()
            .to_string();
        assert!(err.contains("not valid TOML"), "{err}");
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
