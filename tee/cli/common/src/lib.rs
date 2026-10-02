//! Shared foundation of the deploy CLI's two command-bearing crates.
//!
//! Everything here is common to `seismic-tee-node` (the `node` group: reach
//! the running machines) and `seismic-tee-network` (the `network` group: the
//! network directory): the node descriptor map, the network-directory layout
//! and the founding inputs it holds, which both read, the manifest they both
//! trust, and the HTTP, JSON-RPC and error types they both speak — and the
//! terminal conventions every group shares: narration ([`note`]), the
//! closing next step ([`next_step`]), prompts for what was left out
//! ([`prompt`]).
//!
//! This crate depends on neither side, so nothing only one side needs
//! belongs in it: each crate must stay buildable without the other's
//! dependencies, whichever binary mounts it. `seismic-tee-context` is a peer
//! rather than a dependent here: it depends on this crate (for the
//! descriptor type and the network-directory layout it resolves a selection
//! to), but nothing in this crate depends on it.

pub mod artifact;
pub mod descriptor;
pub mod error;
pub mod founding;
pub mod home;
pub mod http;
pub mod manifest;
pub mod network_dir;
pub mod next_step;
pub mod prompt;
pub mod rpc;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

/// The style of narration beside a command's own report — what the context
/// resolved to, what a prompt is listing: dimmed, so it reads as background
/// rather than as part of the report. Dimmed rather than a fixed grey because it follows the terminal's
/// foreground, so it stays legible on light and dark themes alike. Rendered
/// only when stderr is a terminal that wants colour (see `anstream` in the
/// workspace manifest).
pub const NOTE: anstyle::Style = anstyle::Style::new().dimmed();

/// A line of narration on stderr, in [`NOTE`] style.
pub fn note(line: &dyn std::fmt::Display) {
    anstream::eprintln!("{NOTE}{line}{NOTE:#}");
}

pub use artifact::Artifact;
pub use descriptor::{Descriptors, NodeDescriptor, load_descriptors, select_descriptor};
pub use error::{Error, Result};
pub use manifest::{Manifest, hex_0x};
pub use network_dir::NetworkDir;
