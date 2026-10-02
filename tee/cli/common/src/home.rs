//! A leading `~` in a path, expanded against `HOME` and spelled back: Rust
//! has no `Path::expanduser`, and a path typed anywhere the shell does not
//! see it — the context file, an answer at a prompt — carries the tilde
//! as typed.
//!
//! The `HOME` read is split out of [`expand_tilde`] and [`abbreviate`] so
//! their rules are unit-testable without touching process env, which is
//! process-global and racy under a threaded test runner.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Expand a leading `~` against `HOME`.
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

    fn os(s: &str) -> OsString {
        OsString::from(s)
    }

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
