//! Live multi-node status, repainted in place on a TTY and printed on change
//! elsewhere (so CI logs stay readable): one line per node for a cohort's
//! delivery ([`CohortDashboard`]), or a whole block for `status --watch`
//! ([`Painter`]).

use std::collections::BTreeMap;
use std::io::{IsTerminal as _, Write as _};

/// Flatten a status string onto one line. A failure line can carry a
/// verifier's multi-line reason, and the in-place TTY repaint moves the cursor
/// up one row per node — so a newline in a status would tear the dashboard
/// apart. Only line breaks are folded: the progress bar's column padding is
/// significant.
fn one_line(text: &str) -> String {
    text.lines().collect::<Vec<_>>().join(" ")
}

/// The terminal's column count, for truncating a row that would wrap: the
/// in-place repaint moves the cursor up one row per line, so a wrapped line
/// would tear it.
fn columns() -> usize {
    terminal_size::terminal_size().map_or(100, |(w, _)| usize::from(w.0))
}

/// `text` cut to fit `cols` columns, marked with an ellipsis when cut.
fn fit(text: String, cols: usize) -> String {
    if text.chars().count() < cols {
        return text;
    }
    text.chars()
        .take(cols.saturating_sub(1))
        .collect::<String>()
        + "…"
}

/// A block of lines, repainted whole.
pub struct Painter {
    isatty: bool,
    /// Lines the cursor sits below, from the last paint.
    painted: usize,
    last: Vec<String>,
}

impl Default for Painter {
    fn default() -> Self {
        Self::new()
    }
}

impl Painter {
    pub fn new() -> Self {
        Self {
            isatty: std::io::stdout().is_terminal(),
            painted: 0,
            last: Vec::new(),
        }
    }

    pub fn paint(&mut self, lines: &[String]) {
        let mut stdout = std::io::stdout().lock();
        if self.isatty {
            let cols = columns();
            if self.painted > 0 {
                let _ = write!(stdout, "\x1b[{}A", self.painted);
            }
            for line in lines {
                let _ = writeln!(stdout, "\x1b[2K{}", fit(one_line(line), cols));
            }
            // A shorter block clears what the longer one left below it.
            for _ in lines.len()..self.painted {
                let _ = writeln!(stdout, "\x1b[2K");
            }
            self.painted = self.painted.max(lines.len());
        } else if lines != self.last {
            for line in lines {
                let _ = writeln!(stdout, "{line}");
            }
            let _ = writeln!(stdout);
            self.last = lines.to_vec();
        }
        let _ = stdout.flush();
    }
}

/// Named status lines, rendered in the order they were declared.
pub struct CohortDashboard {
    isatty: bool,
    /// `(name, label)`, in display order.
    rows: Vec<(String, String)>,
    width: usize,
    painted: bool,
    last: BTreeMap<String, String>,
}

impl CohortDashboard {
    /// One row per `(name, label)`; the label is what is printed, the name
    /// keys the states handed to [`render`](Self::render).
    pub fn new(rows: Vec<(String, String)>) -> Self {
        assert!(
            !rows.is_empty(),
            "dashboard requires at least one status row"
        );
        let width = rows
            .iter()
            .map(|(_, label)| label.chars().count())
            .max()
            .unwrap_or(0);
        Self {
            isatty: std::io::stdout().is_terminal(),
            rows,
            width,
            painted: false,
            last: BTreeMap::new(),
        }
    }

    pub fn render(&mut self, states: &BTreeMap<String, String>) {
        let mut stdout = std::io::stdout().lock();
        if self.isatty {
            let cols = columns();
            if self.painted {
                let _ = write!(stdout, "\x1b[{}A", self.rows.len());
            }
            for (name, label) in &self.rows {
                let line = one_line(states.get(name).map_or("…", String::as_str));
                let text = fit(format!("{label:>width$}  {line}", width = self.width), cols);
                let _ = writeln!(stdout, "\x1b[2K{text}");
            }
            let _ = stdout.flush();
            self.painted = true;
            return;
        }
        for (name, label) in &self.rows {
            let line = one_line(states.get(name).map_or("…", String::as_str));
            if self.last.get(name) != Some(&line) {
                let _ = writeln!(stdout, "{label}: {line}");
                let _ = stdout.flush();
                self.last.insert(name.clone(), line);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_are_folded_onto_one_line() {
        // Only the breaks are folded: an empty line still leaves its separator.
        assert_eq!(one_line("a\nb\n\nc"), "a b  c");
        assert_eq!(one_line("bar [###---]  10.0%"), "bar [###---]  10.0%");
    }

    #[test]
    fn a_line_that_would_wrap_is_cut_with_an_ellipsis() {
        assert_eq!(fit("abc".to_string(), 4), "abc");
        assert_eq!(fit("abcd".to_string(), 4), "abc…");
    }

    #[test]
    fn rows_keep_their_order_and_widest_label() {
        let dashboard = CohortDashboard::new(vec![
            ("b".into(), "b (genesis)".into()),
            ("a".into(), "a".into()),
        ]);
        assert_eq!(dashboard.rows[0].0, "b");
        assert_eq!(dashboard.width, "b (genesis)".len());
    }

    #[test]
    #[should_panic(expected = "at least one status row")]
    fn an_empty_dashboard_is_a_bug() {
        CohortDashboard::new(Vec::new());
    }
}
