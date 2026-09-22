//! Where the context file lives, and how a `~` inside it is expanded.
//!
//! One place spells the layout, the way [`seismic_tee_common::NetworkDir`]
//! spells a network directory's: relocating the file is a change to this file
//! and nothing else.
//!
//! The env reads are split out of [`default_path`] and [`expand_tilde`] so
//! the resolution logic is unit-testable without touching process env, which
//! is process-global and racy under a threaded test runner.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};

pub const CONFIG_DIRNAME: &str = "seismic";
pub const CONFIG_FILENAME: &str = "config.toml";

/// `$XDG_CONFIG_HOME/seismic/config.toml`, else `~/.config/seismic/config.toml`.
pub fn default_path() -> anyhow::Result<PathBuf> {
    resolve(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

/// [`default_path`]'s rule, with the environment passed in so it is testable.
fn resolve(xdg: Option<OsString>, home: Option<OsString>) -> anyhow::Result<PathBuf> {
    let base = match xdg.filter(|v| !v.is_empty()) {
        Some(xdg) => PathBuf::from(xdg),
        None => match home.filter(|v| !v.is_empty()) {
            Some(home) => PathBuf::from(home).join(".config"),
            None => bail!(
                "neither XDG_CONFIG_HOME nor HOME is set, so there is no config directory to \
                 read; pass --config <FILE>"
            ),
        },
    };
    Ok(base.join(CONFIG_DIRNAME).join(CONFIG_FILENAME))
}

/// `path` made absolute against the current directory, with `.` and `..`
/// collapsed lexically.
///
/// For a path that is about to be stored in the context file: the file is
/// read from whatever directory the next command runs in, so a relative path
/// would point nowhere, and `cli/../networks/x` is nobody's idea of a name.
/// Lexical, not [`std::fs::canonicalize`]: the target need not exist yet
/// and a symlink stays a symlink, so the stored path is the one the operator
/// typed, just spelled from the root.
pub fn absolute(path: &Path) -> anyhow::Result<PathBuf> {
    use std::path::Component;

    let path = std::path::absolute(path)
        .with_context(|| format!("resolving {} against the current directory", path.display()))?;
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

/// Expand a leading `~` against `HOME`.
///
/// Paths in `[networks]` are written by hand, so they carry tildes; Rust has
/// no `Path::expanduser`.
pub fn expand_tilde(path: &Path) -> anyhow::Result<PathBuf> {
    expand(path, std::env::var_os("HOME"))
}

/// [`expand_tilde`]'s rule, with `HOME` passed in so it is testable.
fn expand(path: &Path, home: Option<OsString>) -> anyhow::Result<PathBuf> {
    let Ok(rest) = path.strip_prefix("~") else {
        return Ok(path.to_path_buf());
    };
    let home = home
        .filter(|v| !v.is_empty())
        .ok_or_else(|| anyhow::anyhow!("{} starts with ~, but HOME is not set", path.display()))?;
    let home = PathBuf::from(home);
    if rest.as_os_str().is_empty() {
        return Ok(home);
    }
    Ok(home.join(rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_collapses_dot_and_dotdot() {
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(
            absolute(Path::new("/a/b/../c/./d")).unwrap(),
            Path::new("/a/c/d")
        );
        assert_eq!(absolute(Path::new("/..")).unwrap(), Path::new("/"));
        assert_eq!(absolute(Path::new("./x/../y")).unwrap(), cwd.join("y"),);
        assert!(absolute(Path::new("x")).unwrap().is_absolute());
    }

    fn os(s: &str) -> OsString {
        OsString::from(s)
    }

    #[test]
    fn xdg_config_home_wins() {
        let path = resolve(Some(os("/custom/xdg")), Some(os("/home/sl"))).unwrap();
        assert_eq!(path, Path::new("/custom/xdg/seismic/config.toml"));
    }

    #[test]
    fn an_empty_xdg_config_home_falls_through_to_home() {
        let path = resolve(Some(os("")), Some(os("/home/sl"))).unwrap();
        assert_eq!(path, Path::new("/home/sl/.config/seismic/config.toml"));
    }

    #[test]
    fn neither_set_names_the_flag() {
        let err = resolve(None, None).unwrap_err().to_string();
        assert!(err.contains("--config"), "{err}");

        let err = resolve(Some(os("")), Some(os(""))).unwrap_err().to_string();
        assert!(err.contains("--config"), "{err}");
    }

    #[test]
    fn expand_tilde_on_the_bare_tilde() {
        let path = expand(Path::new("~"), Some(os("/home/sl"))).unwrap();
        assert_eq!(path, Path::new("/home/sl"));
    }

    #[test]
    fn expand_tilde_on_a_tilde_prefixed_path() {
        let path = expand(Path::new("~/x"), Some(os("/home/sl"))).unwrap();
        assert_eq!(path, Path::new("/home/sl/x"));
    }

    #[test]
    fn an_absolute_path_is_returned_unchanged_and_reads_no_home() {
        let path = expand(Path::new("/abs"), None).unwrap();
        assert_eq!(path, Path::new("/abs"));
    }

    #[test]
    fn a_bare_relative_path_is_returned_unchanged_and_reads_no_home() {
        let path = expand(Path::new("relative/path"), None).unwrap();
        assert_eq!(path, Path::new("relative/path"));
    }

    #[test]
    fn a_tilde_path_with_no_home_errors() {
        let err = expand(Path::new("~/x"), None).unwrap_err().to_string();
        assert!(err.contains("HOME"), "{err}");
    }
}
