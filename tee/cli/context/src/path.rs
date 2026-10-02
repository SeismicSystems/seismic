//! Where the context file lives, where fetched networks go, and how a `~`
//! inside the file is expanded.
//!
//! One place spells the layout, the way [`seismic_tee_common::NetworkDir`]
//! spells a network directory's: relocating the file is a change to this file
//! and nothing else.
//!
//! The env reads are split out of [`default_path`], [`networks_root`] and
//! [`expand_tilde`] so
//! the resolution logic is unit-testable without touching process env, which
//! is process-global and racy under a threaded test runner.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};

pub const CONFIG_DIRNAME: &str = "seismic";
pub const CONFIG_FILENAME: &str = "config.toml";
pub const NETWORKS_DIRNAME: &str = "networks";

/// `$XDG_DATA_HOME/seismic/networks`, else `~/.local/share/seismic/networks`:
/// where `ctx set-network --dir <URL>` fetches a network directory to, as
/// `<name>/`. The CLI's data, so XDG's data home rather than beside the
/// context file.
pub fn networks_root() -> anyhow::Result<PathBuf> {
    resolve_networks(std::env::var_os("XDG_DATA_HOME"), std::env::var_os("HOME"))
}

/// `$XDG_CONFIG_HOME/seismic/config.toml`, else `~/.config/seismic/config.toml`.
pub fn default_path() -> anyhow::Result<PathBuf> {
    resolve(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

/// [`default_path`]'s rule, with the environment passed in so it is testable.
fn resolve(xdg: Option<OsString>, home: Option<OsString>) -> anyhow::Result<PathBuf> {
    let Some(base) = xdg_base(xdg, home, ".config") else {
        bail!(
            "neither XDG_CONFIG_HOME nor HOME is set, so there is no config directory to read; \
             pass --config <FILE>"
        );
    };
    Ok(base.join(CONFIG_DIRNAME).join(CONFIG_FILENAME))
}

/// [`networks_root`]'s rule, with the environment passed in so it is
/// testable.
fn resolve_networks(xdg: Option<OsString>, home: Option<OsString>) -> anyhow::Result<PathBuf> {
    let Some(base) = xdg_base(xdg, home, ".local/share") else {
        bail!("neither XDG_DATA_HOME nor HOME is set, so there is nowhere to fetch a network to");
    };
    Ok(base.join(CONFIG_DIRNAME).join(NETWORKS_DIRNAME))
}

/// An XDG base directory: the variable's value when set and non-empty, else
/// `$HOME/<fallback>`, else nothing.
fn xdg_base(xdg: Option<OsString>, home: Option<OsString>, fallback: &str) -> Option<PathBuf> {
    match xdg.filter(|v| !v.is_empty()) {
        Some(xdg) => Some(PathBuf::from(xdg)),
        None => home
            .filter(|v| !v.is_empty())
            .map(|home| PathBuf::from(home).join(fallback)),
    }
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

/// `path` spelled from `~` when it is under `HOME`: [`expand_tilde`]'s
/// inverse, for a path shown to be read or edited.
pub fn abbreviate(path: &Path) -> String {
    abbreviated(path, std::env::var_os("HOME"))
}

/// [`abbreviate`]'s rule, with `HOME` passed in so it is testable.
fn abbreviated(path: &Path, home: Option<OsString>) -> String {
    let rest = home
        .filter(|v| !v.is_empty())
        .and_then(|home| path.strip_prefix(home).ok().map(Path::to_path_buf));
    match rest {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
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
    fn abbreviate_spells_a_path_under_home_from_tilde() {
        let home = Some(OsString::from("/home/alice"));
        let path = Path::new("/home/alice/.local/share/seismic/networks/x");
        assert_eq!(
            abbreviated(path, home.clone()),
            "~/.local/share/seismic/networks/x"
        );
        assert_eq!(
            expand(Path::new(&abbreviated(path, home.clone())), home.clone()).unwrap(),
            path
        );
        assert_eq!(abbreviated(Path::new("/home/alice"), home.clone()), "~");
        assert_eq!(
            abbreviated(Path::new("/home/alicex/y"), home.clone()),
            "/home/alicex/y"
        );
        assert_eq!(abbreviated(path, None), path.display().to_string());
    }

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
    fn networks_go_to_the_xdg_data_home_else_local_share() {
        assert_eq!(
            resolve_networks(Some(os("/custom/data")), Some(os("/home/sl"))).unwrap(),
            Path::new("/custom/data/seismic/networks")
        );
        assert_eq!(
            resolve_networks(Some(os("")), Some(os("/home/sl"))).unwrap(),
            Path::new("/home/sl/.local/share/seismic/networks")
        );
        let err = resolve_networks(None, None).unwrap_err().to_string();
        assert!(err.contains("XDG_DATA_HOME"), "{err}");
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
