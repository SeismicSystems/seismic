//! The layout of a network directory.
//!
//! A network directory holds the authored inputs under `inputs/` and the
//! derived artifact set at the top level. Everything top-level is hash-pinned
//! by the manifest; everything under `inputs/` is provenance; `nodes/` holds
//! the records `configure` writes as it runs, regenerated per deploy and
//! gitignored. The cohort's node table itself is not one of these files: it
//! lives in the context file's `[networks.<name>.nodes]`
//! ([`crate::descriptor`]), imported by `ctx set-nodes` or passed as
//! `--nodes FILE`.
//!
//! ```text
//! inputs/image.json                            the image's release record (seismic-images'
//!                                              `image.json`): which image, where its bytes are
//! inputs/reth-genesis.json                     policy-free genesis
//! inputs/summit-genesis.toml                   summit parameter choices
//! inputs/measurements.json                     raw PCR map from `make measure`
//! inputs/founder-withdrawal-credentials.json   one address per founder
//! inputs/harvest/<node>.json                   the founding archive: harvested pubkeys +
//!                                              quote, and the bundle that quote verified against
//!
//! network-manifest.json                        the network's identity; SHA-256 = network_id
//! reth-genesis.json                            the input genesis with compiled
//!                                              registry storage injected
//! summit-genesis.toml                          the completed summit genesis
//! measurement-policy-bootstrap.json            the founding accepted measurement set
//!
//! nodes/bootnodes.json                         the founding enode set
//! nodes/<node>.init-config.toml                the config `configure` POSTed
//!                                              to that node, byte-exact
//! ```
//!
//! These are layout facts, not configuration: both command groups name them, so they are
//! spelled once here.

use std::path::{Path, PathBuf};

/// The network's identity document. Its exact bytes hash to `network_id`.
pub const MANIFEST_FILENAME: &str = "network-manifest.json";
/// "bootstrap" because this is only the *founding* allowlist — what the
/// manifest's `bootstrap_policy_hash` pins and the registry's genesis storage
/// is compiled from. The live policy is the registry contract's state, which
/// the authority can mutate after genesis.
pub const POLICY_FILENAME: &str = "measurement-policy-bootstrap.json";
pub const RETH_GENESIS_FILENAME: &str = "reth-genesis.json";
/// Both the authored input and the shipped artifact use this basename: same
/// format, the artifact being the input with the derived fields filled in.
pub const SUMMIT_GENESIS_FILENAME: &str = "summit-genesis.toml";
pub const MEASUREMENTS_FILENAME: &str = "measurements.json";
pub const FOUNDERS_FILENAME: &str = "founder-withdrawal-credentials.json";
/// seismic-images' record of the image, copied in by `init --image` under
/// the name the release gives it: which image (its tag), the commits it was
/// built from, and per cloud target where its bytes are. `assemble` reads
/// the tag from it to fetch the image's own binaries; the provisioner reads
/// the blob location from it. Absent from a directory `init` scaffolded
/// from loose files.
pub const IMAGE_FILENAME: &str = "image.json";

pub const INPUTS_DIRNAME: &str = "inputs";
pub const HARVEST_DIRNAME: &str = "harvest";
pub const NODES_DIRNAME: &str = "nodes";
pub const BOOTNODES_FILENAME: &str = "bootnodes.json";
/// `<node>` + this: the tdx-init config as it was POSTed to that node, kept
/// as the record of what the node booted with and as a body `curl` can
/// replay. Under `nodes/` because it is per-deploy output — it carries live
/// IPs and the bootnode set of the moment.
pub const INIT_CONFIG_SUFFIX: &str = ".init-config.toml";

/// One network directory, addressed by the layout above.
///
/// Constructing it asserts nothing about what exists on disk: `init` builds a
/// directory that only has inputs, `assemble` fills in the artifact set, and
/// each command reports what it needs and can't find.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkDir {
    root: PathBuf,
}

impl NetworkDir {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The directory a manifest sits in, read as a network directory.
    ///
    /// This is the artifact-set convention the single-node commands rely on:
    /// an operator is handed `--manifest`, and the reth genesis, summit
    /// genesis and measurement policy that manifest pins are the files
    /// `assemble` wrote beside it — so a command that defaults to "beside
    /// `--manifest`" is defaulting to the very files the manifest's hashes
    /// were computed from. A manifest named with no directory component
    /// resolves its siblings relative to the working directory, as the flag
    /// itself did.
    pub fn of_manifest(manifest: &Path) -> Self {
        Self::new(manifest.parent().unwrap_or_else(|| Path::new("")))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    // The derived artifact set.

    pub fn manifest(&self) -> PathBuf {
        self.root.join(MANIFEST_FILENAME)
    }

    pub fn policy(&self) -> PathBuf {
        self.root.join(POLICY_FILENAME)
    }

    pub fn reth_genesis(&self) -> PathBuf {
        self.root.join(RETH_GENESIS_FILENAME)
    }

    pub fn summit_genesis(&self) -> PathBuf {
        self.root.join(SUMMIT_GENESIS_FILENAME)
    }

    // The authored inputs.

    pub fn inputs(&self) -> PathBuf {
        self.root.join(INPUTS_DIRNAME)
    }

    pub fn input_reth_genesis(&self) -> PathBuf {
        self.inputs().join(RETH_GENESIS_FILENAME)
    }

    pub fn input_summit_genesis(&self) -> PathBuf {
        self.inputs().join(SUMMIT_GENESIS_FILENAME)
    }

    pub fn input_measurements(&self) -> PathBuf {
        self.inputs().join(MEASUREMENTS_FILENAME)
    }

    pub fn input_image(&self) -> PathBuf {
        self.inputs().join(IMAGE_FILENAME)
    }

    pub fn founders(&self) -> PathBuf {
        self.inputs().join(FOUNDERS_FILENAME)
    }

    pub fn harvest(&self) -> PathBuf {
        self.inputs().join(HARVEST_DIRNAME)
    }

    /// One node's founding archive: its harvested record and everything the
    /// verification of it rested on, in one document (the verify-quote
    /// library's archive format).
    pub fn harvest_record(&self, node: &str) -> PathBuf {
        self.harvest().join(format!("{node}.json"))
    }

    // Infra state.

    pub fn nodes(&self) -> PathBuf {
        self.root.join(NODES_DIRNAME)
    }

    /// Every top-level entry the layout owns, whether or not it exists: the
    /// authored inputs and the harvest (`inputs/`), the derived artifact set,
    /// and the infra state (`nodes/`). What `init --force` starts over and
    /// the most `network rm` finds in a directory it deletes.
    pub fn top_level(&self) -> [PathBuf; 6] {
        [
            self.inputs(),
            self.manifest(),
            self.policy(),
            self.reth_genesis(),
            self.summit_genesis(),
            self.nodes(),
        ]
    }

    pub fn bootnodes(&self) -> PathBuf {
        self.nodes().join(BOOTNODES_FILENAME)
    }

    /// The record of the config POSTed to `node`.
    pub fn init_config(&self, node: &str) -> PathBuf {
        self.nodes().join(format!("{node}{INIT_CONFIG_SUFFIX}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The committed `tee/networks/fixture-devnet/` is the shape these paths
    /// describe; spot-check one path per tier against it.
    #[test]
    fn paths_hang_off_the_root() {
        let dir = NetworkDir::new("tee/networks/devnet-3");

        assert_eq!(
            dir.manifest(),
            Path::new("tee/networks/devnet-3/network-manifest.json")
        );
        assert_eq!(
            dir.input_measurements(),
            Path::new("tee/networks/devnet-3/inputs/measurements.json")
        );
        assert_eq!(
            dir.harvest_record("dev-bootstrap-node-1"),
            Path::new("tee/networks/devnet-3/inputs/harvest/dev-bootstrap-node-1.json")
        );
        assert_eq!(
            dir.bootnodes(),
            Path::new("tee/networks/devnet-3/nodes/bootnodes.json")
        );
        assert_eq!(
            dir.init_config("dev-bootstrap-node-1"),
            Path::new("tee/networks/devnet-3/nodes/dev-bootstrap-node-1.init-config.toml")
        );
    }

    /// The single-node commands resolve the files a manifest pins from its
    /// own directory — including when the flag names a bare filename.
    #[test]
    fn the_artifact_set_is_found_beside_the_manifest() {
        let dir = NetworkDir::of_manifest(Path::new("tee/networks/devnet-3/network-manifest.json"));
        assert_eq!(
            dir.policy(),
            Path::new("tee/networks/devnet-3/measurement-policy-bootstrap.json")
        );
        assert_eq!(
            dir.reth_genesis(),
            Path::new("tee/networks/devnet-3/reth-genesis.json")
        );
        assert_eq!(
            dir.summit_genesis(),
            Path::new("tee/networks/devnet-3/summit-genesis.toml")
        );

        let bare = NetworkDir::of_manifest(Path::new("network-manifest.json"));
        assert_eq!(bare.reth_genesis(), Path::new("reth-genesis.json"));
    }

    /// The artifact set and the inputs it was derived from share basenames and
    /// must never collide.
    #[test]
    fn the_artifact_set_never_collides_with_its_inputs() {
        let dir = NetworkDir::new("n");
        assert_ne!(dir.reth_genesis(), dir.input_reth_genesis());
        assert_ne!(dir.summit_genesis(), dir.input_summit_genesis());
    }
}
