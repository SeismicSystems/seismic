//! A file's bytes, kept with the path they came from.
//!
//! The artifacts both command groups deliver — a reth genesis, a summit genesis, a
//! measurement policy — are checked against the manifest and then shipped
//! byte-verbatim, and a check that fails on their *contents* still has to name
//! the *file*, which by then is several calls away from the flag that named
//! it. Carrying the two together is what lets a library call report
//! "`--reth-genesis` /path: chainId mismatch" without being handed the flag.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// A file read whole, and where it was read from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Artifact {
    path: PathBuf,
    bytes: Vec<u8>,
}

impl Artifact {
    /// Read the file at `path`, naming it on failure.
    pub fn read(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(|e| Error::read(path, e))?;
        Ok(Self::new(path, bytes))
    }

    /// Bytes that came from somewhere other than a plain read — a delivered
    /// summit genesis with current IPs spliced in, say — still name the file
    /// they are a version of.
    pub fn new(path: impl Into<PathBuf>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            bytes: bytes.into(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_file_and_keeps_its_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reth-genesis.json");
        std::fs::write(&path, b"{}").unwrap();

        let artifact = Artifact::read(&path).unwrap();
        assert_eq!(artifact.path(), path);
        assert_eq!(artifact.bytes(), b"{}");
    }

    #[test]
    fn a_read_failure_names_the_file() {
        let err = Artifact::read(Path::new("/absent/summit-genesis.toml"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("summit-genesis.toml"), "{err}");
    }
}
