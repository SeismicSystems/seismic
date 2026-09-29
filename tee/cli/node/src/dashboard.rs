//! Live multi-node status: one line per node, repainted in place on a TTY,
//! printed on change elsewhere (so CI logs stay readable).

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

    /// The terminal's column count, for truncating a row that would wrap.
    fn columns() -> usize {
        terminal_size::terminal_size().map_or(100, |(w, _)| usize::from(w.0))
    }

    pub fn render(&mut self, states: &BTreeMap<String, String>) {
        let mut stdout = std::io::stdout().lock();
        if self.isatty {
            let cols = Self::columns();
            if self.painted {
                let _ = write!(stdout, "\x1b[{}A", self.rows.len());
            }
            for (name, label) in &self.rows {
                let line = one_line(states.get(name).map_or("…", String::as_str));
                let mut text = format!("{label:>width$}  {line}", width = self.width);
                if text.chars().count() >= cols {
                    text = text
                        .chars()
                        .take(cols.saturating_sub(1))
                        .collect::<String>()
                        + "…";
                }
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
