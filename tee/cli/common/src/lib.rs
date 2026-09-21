//! Shared foundation of the deploy CLI's two command-bearing crates.
//!
//! Everything here is common to `seismic-tee-node` (the `node` group: act on
//! your own node) and `seismic-tee-network` (the `network` group and the rest
//! of the founder side): the node descriptor map that is the seam between
//! them, the network-directory layout they both read, the manifest they both
//! trust and the gates it puts the other artifacts through, and the HTTP,
//! JSON-RPC and error types they both speak.
//!
//! This crate depends on neither side, so nothing founder-only belongs in it:
//! the node crate must stay buildable without a single founder-only
//! dependency, whichever binary mounts it. `seismic-tee-context` is a peer
//! rather than a dependent here: it depends on this crate (for the
//! descriptor type and the network-directory layout it resolves a selection
//! to), but nothing in this crate depends on it.

pub mod artifact;
pub mod descriptor;
pub mod error;
pub mod http;
pub mod manifest;
pub mod network_dir;
pub mod next_step;
pub mod rpc;
#[cfg(feature = "test-support")]
pub mod test_support;

pub use artifact::Artifact;
pub use descriptor::{Descriptors, NodeDescriptor, load_descriptors, select_descriptor};
pub use error::{Error, Result};
pub use manifest::Manifest;
pub use network_dir::NetworkDir;
