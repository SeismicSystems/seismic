//! `DIR`: which network directory a founding command acts on.
//!
//! Shared by `harvest`, `assemble` and `verify-founding`. An
//! explicit `DIR` wins outright and the context is never read, so a command
//! line stays a complete record of what it acted on; otherwise the current
//! context's network supplies it, and the resolved directory is echoed on
//! stderr before anything is read or written — including for `--force`, so a
//! stale context plus `--force` names the directory it is about to
//! overwrite.

use std::path::{Path, PathBuf};

use crate::{Context, ContextArgs, echo};
use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct DirArgs {
    #[arg(value_name = "DIR")]
    pub dir: Option<PathBuf>,

    #[command(flatten)]
    pub context: ContextArgs,
}

impl DirArgs {
    /// Resolve to the one network directory this invocation acts on, reading
    /// the context file at `config` when `DIR` is not given.
    pub fn load(&self, config: Option<&Path>) -> anyhow::Result<PathBuf> {
        if let Some(dir) = &self.dir {
            return Ok(dir.clone());
        }
        let context = Context::load(config)?;
        if self.context.context.is_none() && context.config().current.is_none() {
            anyhow::bail!("no context selected — pass DIR, or run `seismic-tee ctx use <network>`");
        }
        let selected = context.select(self.context.context.as_deref())?;
        let dir = selected.dir()?;
        echo(&selected.selection, &dir.display());
        Ok(dir)
    }

    /// `DIR` as it was given, for a suggested follow-up command that must act
    /// on the same network. Empty when the context supplied it — the
    /// persisted selection is still in force for the next command, so
    /// nothing needs repeating; an explicit `--context` is not persisted
    /// anywhere, so it is repeated like `DIR` is. Leading space included, so
    /// it splices into a command line.
    pub fn as_args(&self) -> String {
        if let Some(dir) = &self.dir {
            return format!(" {}", dir.display());
        }
        match &self.context.context {
            Some(context) => format!(" --context {context}"),
            None => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NODES_ONLY_CONFIG: &str = r#"
current = "partner-net"

[networks.partner-net.nodes]
my-node = { public_ip = "198.51.100.4", fqdn = "my-node.example.com" }
"#;

    const DIR_NETWORK_CONFIG: &str = r#"
current = "devnet-1"

[networks.devnet-1]
dir = "/nets/devnet-1"
"#;

    fn args(dir: Option<&str>) -> DirArgs {
        DirArgs {
            dir: dir.map(PathBuf::from),
            context: ContextArgs { context: None },
        }
    }

    #[test]
    fn an_explicit_dir_wins() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::fs::write(&config_path, DIR_NETWORK_CONFIG).unwrap();

        let dir = args(Some("/elsewhere")).load(Some(&config_path)).unwrap();
        assert_eq!(dir, PathBuf::from("/elsewhere"));
    }

    #[test]
    fn a_missing_dir_resolves_the_contexts_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::fs::write(&config_path, DIR_NETWORK_CONFIG).unwrap();

        let dir = args(None).load(Some(&config_path)).unwrap();
        assert_eq!(dir, PathBuf::from("/nets/devnet-1"));
    }

    #[test]
    fn a_nodes_only_network_errors_naming_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::fs::write(&config_path, NODES_ONLY_CONFIG).unwrap();

        let err = args(None).load(Some(&config_path)).unwrap_err().to_string();
        assert!(err.contains("is nodes only"), "{err}");
        assert!(err.contains("pass DIR"), "{err}");
    }

    #[test]
    fn no_context_names_dir_and_ctx_use() {
        let tmp = tempfile::tempdir().unwrap();
        // A config path that names no file: an empty config, no `current`.
        let config_path = tmp.path().join("config.toml");

        let err = args(None).load(Some(&config_path)).unwrap_err().to_string();
        assert!(err.contains("DIR"), "{err}");
        assert!(err.contains("ctx use"), "{err}");
    }

    #[test]
    fn an_explicit_dir_is_repeated_and_a_persisted_selection_is_not() {
        let explicit = args(Some("/nets/devnet-1"));
        assert_eq!(explicit.as_args(), " /nets/devnet-1");
        let from_selection = args(None);
        assert_eq!(from_selection.as_args(), "");
        let from_flag = DirArgs {
            dir: None,
            context: ContextArgs {
                context: Some("devnet-1".to_string()),
            },
        };
        assert_eq!(from_flag.as_args(), " --context devnet-1");
    }
}
