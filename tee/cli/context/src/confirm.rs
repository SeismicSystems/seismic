//! The typed-name confirmation `network rm` and `ctx rm` share, as `pulumi
//! stack rm` asks it: the warnings in red, then the name, in blue, to type
//! back. `--yes` skips the question; with no terminal and no `--yes` there is
//! nobody to ask, and that is an error rather than a redirected stdin
//! deciding.

use std::io::{BufRead, Write as _};

use anyhow::{Context as _, bail};

/// Red, as `pulumi stack rm` warns. Rendered only when stderr is a terminal
/// that wants colour (see `anstream` in the workspace manifest).
const DANGER: anstyle::Style = anstyle::AnsiColor::Red.on_default();

/// The name to type back: blue, as `pulumi stack rm` shows it.
const NAME: anstyle::Style = anstyle::AnsiColor::Blue.on_default();

/// A line on stderr in [`DANGER`] style.
pub fn danger(line: &str) {
    anstream::eprintln!("{DANGER}{line}{DANGER:#}");
}

/// Refuse when nobody can answer for `name`: not `yes`, and not
/// `interactive`. `verb` is what the command would do to it, unattended.
pub fn require_answerable(
    name: &str,
    verb: &str,
    yes: bool,
    interactive: bool,
) -> anyhow::Result<()> {
    if !yes && !interactive {
        bail!(
            "stdin is not a terminal, so nobody is here to type `{name}` back; pass --yes to \
             {verb} it unattended"
        );
    }
    Ok(())
}

/// Ask for `name` to be typed back on `input`. A wrong answer is an error,
/// so nothing after it runs; `undone` says what that leaves as it was.
pub fn type_back(name: &str, undone: &str, input: &mut impl BufRead) -> anyhow::Result<()> {
    anstream::eprint!("Type `{NAME}{name}{NAME:#}` to confirm: ");
    std::io::stderr().flush().context("flushing the prompt")?;
    let mut answer = String::new();
    input
        .read_line(&mut answer)
        .context("reading the confirmation")?;
    if answer.trim() != name {
        bail!("`{}` is not `{name}`; {undone}", answer.trim());
    }
    Ok(())
}
