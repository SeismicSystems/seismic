//! `DIR`: which network directory a founding command acts on.
//!
//! Shared by `harvest`, `assemble` and `verify-founding`. An
//! explicit `DIR` wins outright and the context is never read, so a command
//! line stays a complete record of what it acted on; otherwise the current
//! context's network supplies it, and the resolved directory is echoed on
//! stderr before anything is read or written — including for `--force`, so a
//! stale context plus `--force` names the directory it is about to
//! overwrite.

use std::path::PathBuf;

use clap::Args;
use seismic_tee_context::{Context, ContextArgs, echo};

#[derive(Debug, Clone, Args)]
pub struct DirArgs {
    #[arg(value_name = "DIR")]
    pub dir: Option<PathBuf>,

    #[command(flatten)]
    pub context: ContextArgs,
}

impl DirArgs {
    /// Resolve to the one network directory this invocation acts on.
    pub fn load(&self) -> anyhow::Result<PathBuf> {
        if let Some(dir) = &self.dir {
            return Ok(dir.clone());
        }
        let context = Context::load(self.context.config.as_deref())?;
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

    const LOOSE_NETWORK_CONFIG: &str = r#"
current = "partner-net"

[networks.partner-net]
manifest = "/m/network-manifest.json"
"#;

    const DIR_NETWORK_CONFIG: &str = r#"
current = "devnet-1"

[networks.devnet-1]
dir = "/nets/devnet-1"
"#;

    fn args(dir: Option<&str>, config_path: PathBuf) -> DirArgs {
        DirArgs {
            dir: dir.map(PathBuf::from),
            context: ContextArgs {
                context: None,
                config: Some(config_path),
            },
        }
    }

    #[test]
    fn an_explicit_dir_wins() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::fs::write(&config_path, DIR_NETWORK_CONFIG).unwrap();

        let dir = args(Some("/elsewhere"), config_path).load().unwrap();
        assert_eq!(dir, PathBuf::from("/elsewhere"));
    }

    #[test]
    fn a_missing_dir_resolves_the_contexts_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::fs::write(&config_path, DIR_NETWORK_CONFIG).unwrap();

        let dir = args(None, config_path).load().unwrap();
        assert_eq!(dir, PathBuf::from("/nets/devnet-1"));
    }

    #[test]
    fn a_loose_files_network_errors_with_has_no_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::fs::write(&config_path, LOOSE_NETWORK_CONFIG).unwrap();

        let err = args(None, config_path).load().unwrap_err().to_string();
        assert!(err.contains("has no dir"), "{err}");
        assert!(err.contains("pass DIR"), "{err}");
    }

    #[test]
    fn no_context_names_dir_and_ctx_use() {
        let tmp = tempfile::tempdir().unwrap();
        // A config path that names no file: an empty config, no `current`.
        let config_path = tmp.path().join("config.toml");

        let err = args(None, config_path).load().unwrap_err().to_string();
        assert!(err.contains("DIR"), "{err}");
        assert!(err.contains("ctx use"), "{err}");
    }

    #[test]
    fn an_explicit_dir_is_repeated_and_a_persisted_selection_is_not() {
        let explicit = args(Some("/nets/devnet-1"), PathBuf::from("/nowhere.toml"));
        assert_eq!(explicit.as_args(), " /nets/devnet-1");
        let from_selection = args(None, PathBuf::from("/nowhere.toml"));
        assert_eq!(from_selection.as_args(), "");
        let from_flag = DirArgs {
            dir: None,
            context: ContextArgs {
                context: Some("devnet-1".to_string()),
                config: None,
            },
        };
        assert_eq!(from_flag.as_args(), " --context devnet-1");
    }
}
