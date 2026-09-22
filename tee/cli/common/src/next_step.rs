//! The line a command ends on: the step after it, with this run's arguments
//! filled in.
//!
//! `network init` ends by naming what comes next, and that is why the
//! founding reads as a sequence rather than a list of commands. Every command
//! in the chain ends the same way, through this one printer, so the shape is
//! the same everywhere: a `Next:` line, an optional lead saying what the step
//! is for, then one command per line, indented, spelled with the arguments
//! the finished command already knows (the network directory, a node name
//! from the table) so it pastes. On stderr with the rest of the narration:
//! stdout stays what scripts consume.
//!
//! Every argv spelled here must parse — `the_documented_invocations_parse`
//! in the binary's `main.rs` is the guard against a printed invocation
//! drifting from the command it names.

/// A headed block: `heading` (with `lead` on the same line when there is
/// one), then each line indented four spaces. Empty when there are no lines.
fn block(heading: &str, lead: &str, lines: &[String]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let mut out = format!("\n{heading}");
    if !lead.is_empty() {
        out.push(' ');
        out.push_str(lead);
    }
    for line in lines {
        out.push_str("\n    ");
        out.push_str(line);
    }
    out.push('\n');
    out
}

/// The block as printed: `Next:` (with `lead` on the same line when there is
/// one), then each command indented four spaces. Empty when there is nothing
/// to print.
pub fn render(lead: &str, commands: &[String]) -> String {
    block("Next:", lead, commands)
}

/// `Next:` over a numbered list, for the one command whose next step is
/// several in order (`init`, whose successor is reached through the
/// operator's own edits and a provisioning tool). A step's later lines are
/// indented under its text, so a command sits beneath the sentence that
/// introduces it.
pub fn render_steps(steps: &[String]) -> String {
    let numbered: Vec<String> = steps
        .iter()
        .enumerate()
        .map(|(i, step)| {
            let mut lines = step.lines();
            let mut out = format!("{}. {}", i + 1, lines.next().unwrap_or_default());
            for line in lines {
                out.push_str("\n       ");
                out.push_str(line);
            }
            out
        })
        .collect();
    block("Next:", "", &numbered)
}

/// `ctx:` over commands: what the context file holds for the commands that
/// follow, and the `ctx` command that would add to it. The lead may span
/// lines; they are printed as given.
pub fn render_ctx(lead: &str, commands: &[String]) -> String {
    block("ctx:", lead, commands)
}

/// Print [`render`] on stderr. Nothing is printed when `commands` is empty:
/// a command with no definite next step ends on its own report.
pub fn print(lead: &str, commands: &[String]) {
    eprint!("{}", render(lead, commands));
}

/// Print [`render_steps`] on stderr.
pub fn print_steps(steps: &[String]) {
    eprint!("{}", render_steps(steps));
}

/// Print [`render_ctx`] on stderr.
pub fn print_ctx(lead: &str, commands: &[String]) {
    eprint!("{}", render_ctx(lead, commands));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_next_is_the_commands_alone() {
        assert_eq!(
            render("", &["seismic-tee network assemble".to_string()]),
            "\nNext:\n    seismic-tee network assemble\n"
        );
    }

    #[test]
    fn the_lead_shares_the_next_line_and_each_command_gets_its_own() {
        assert_eq!(
            render(
                "appraise it before relying on it:",
                &["a".to_string(), "b".to_string()]
            ),
            "\nNext: appraise it before relying on it:\n    a\n    b\n"
        );
    }

    #[test]
    fn no_commands_prints_nothing() {
        assert_eq!(render("lead", &[]), "");
        assert_eq!(render_steps(&[]), "");
        assert_eq!(render_ctx("lead", &[]), "");
    }

    #[test]
    fn steps_are_numbered_and_a_steps_later_lines_sit_under_its_first() {
        assert_eq!(
            render_steps(&[
                "review the inputs".to_string(),
                "harvest them:\nseismic-tee network harvest".to_string(),
            ]),
            "\nNext:\n    1. review the inputs\n    2. harvest them:\n       seismic-tee network \
             harvest\n"
        );
    }

    #[test]
    fn a_ctx_block_keeps_a_multi_line_lead_as_given() {
        assert_eq!(
            render_ctx("x is selected,\nso DIR is optional:", &["a".to_string()]),
            "\nctx: x is selected,\nso DIR is optional:\n    a\n"
        );
    }
}
