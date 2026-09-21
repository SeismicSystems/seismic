//! The ways deploy-side work fails, with the file or node it failed on.
//!
//! Deploy tooling reads files an operator authored and talks to boxes an
//! operator provisioned, so every error names the thing that was wrong. The
//! two failure *kinds* that matter are the manifest schema (an artifact does
//! not parse) and a gate (artifacts parse but disagree with each other) —
//! both fail at deploy time rather than at boot.

use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("reading {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{path} is not valid JSON")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    /// The strict v1 schema's verdict, from the same crate every node parses
    /// the manifest with.
    #[error("{path} does not satisfy the network-manifest schema")]
    ManifestSchema {
        path: PathBuf,
        #[source]
        source: seismic_network_manifest::ManifestError,
    },

    /// A cross-artifact validation gate failed: each file is well-formed, but
    /// they don't agree on the network they describe.
    #[error("{0}")]
    Gate(String),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    /// A node's JSON-RPC endpoint gave no answer: connection refused, a
    /// timeout, or a reply that was not JSON-RPC. The normal state of a node
    /// still coming up, which is why pollers keep going on it.
    #[error("no answer from {url}")]
    RpcTransport {
        url: String,
        #[source]
        source: jsonrpsee::core::ClientError,
    },

    /// A node's JSON-RPC endpoint answered, with an error object. Unlike
    /// [`Self::RpcTransport`] this is never the normal state of a node coming
    /// up.
    #[error("RPC error from {url}: {message}")]
    Rpc { url: String, message: String },
}

impl Error {
    pub fn read(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Read {
            path: path.into(),
            source,
        }
    }

    pub fn json(path: impl Into<PathBuf>, source: serde_json::Error) -> Self {
        Self::Json {
            path: path.into(),
            source,
        }
    }

    pub fn gate(message: impl Into<String>) -> Self {
        Self::Gate(message.into())
    }
}
