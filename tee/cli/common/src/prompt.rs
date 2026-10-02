//! Asking for a value the command line left out. On a terminal a required
//! one is asked for, with help: a list of what the value usually is,
//! filtered as it is typed, or text input that completes paths on Tab. With
//! no terminal there is nobody to ask, so the command fails naming the flag
//! that supplies it. A prompt never blocks a script, and every prompt has a
//! flag that answers it. An optional path is offered pre-filled with its
//! default, to accept or edit, and taken as the default with no terminal.

use std::collections::VecDeque;
use std::fmt;
use std::io::IsTerminal as _;
use std::path::Path;

use anyhow::bail;
use inquire::autocompletion::{Autocomplete, Replacement};
use inquire::{CustomUserError, InquireError, Select, Text};

use crate::{home, note};

/// Where the answers to a command's prompts come from.
pub struct Prompt {
    answers: Answers,
}

enum Answers {
    /// A person at a terminal.
    Terminal,
    /// A test's answers, in order: a choice's label picks it, anything else
    /// is typed.
    Scripted(VecDeque<String>),
    /// Every missing value is refused.
    Nobody,
}

/// One entry of a list to choose from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// What the list shows, and what typing filters on.
    pub label: String,
    /// What picking it answers.
    pub value: String,
}

/// The last entry of every list: the way to a value the list does not hold.
const OTHER: &str = "Other: type a path or URL";

impl Prompt {
    /// A person, when stdin and stderr are both a terminal: the prompt is
    /// drawn on stderr, and a redirected stderr is a log, not a screen.
    pub fn stdin() -> Self {
        let interactive = std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
        Self {
            answers: if interactive {
                Answers::Terminal
            } else {
                Answers::Nobody
            },
        }
    }

    pub fn nobody() -> Self {
        Self {
            answers: Answers::Nobody,
        }
    }

    /// `answers`, in order, for a test.
    pub fn scripted(answers: &[&str]) -> Self {
        Self {
            answers: Answers::Scripted(answers.iter().map(|a| a.to_string()).collect()),
        }
    }

    /// Whether a missing value would be asked for, so a caller can gather a
    /// [`Self::choose`] list ahead — an async one, say — only when it is.
    pub fn can_ask(&self) -> bool {
        !matches!(self.answers, Answers::Nobody)
    }

    /// `value` when the command line gave it; else typed in answer to
    /// `question`; else, with nobody to ask, an error naming `flag`.
    pub fn text<T: From<String>>(
        &mut self,
        value: Option<T>,
        question: &str,
        flag: &str,
    ) -> anyhow::Result<T> {
        self.required(value, question, flag, None)
    }

    /// `value` when the command line gave it; else `default`, offered on a
    /// terminal to accept with Enter or edit, paths completed on Tab.
    pub fn path<T: From<String>>(
        &mut self,
        value: Option<T>,
        question: &str,
        flag: &str,
        default: &str,
    ) -> anyhow::Result<T> {
        if let Some(value) = value {
            return Ok(value);
        }
        let answer = match &mut self.answers {
            Answers::Nobody => String::new(),
            Answers::Scripted(answers) => next(answers, flag)?,
            Answers::Terminal => ask(question, flag, Some(default), Some(PathCompleter))?,
        };
        let answer = if answer.is_empty() {
            default.to_string()
        } else {
            answer
        };
        Ok(expand(answer)?.into())
    }

    /// A required value, typed; paths completed on Tab with `paths`.
    fn required<T: From<String>>(
        &mut self,
        value: Option<T>,
        question: &str,
        flag: &str,
        paths: Option<PathCompleter>,
    ) -> anyhow::Result<T> {
        if let Some(value) = value {
            return Ok(value);
        }
        let answer = match &mut self.answers {
            Answers::Nobody => bail!("{}", unanswered(flag)),
            Answers::Scripted(answers) => next(answers, flag)?,
            Answers::Terminal => loop {
                let answer = ask(question, flag, None, paths.clone())?;
                if !answer.is_empty() {
                    break answer;
                }
            },
        };
        Ok(expand(answer)?.into())
    }

    /// `value` when the command line gave it; else picked from what `list`
    /// returns, or typed, paths completed on Tab; else, with nobody to ask, an
    /// error naming `flag`. `list` runs only when there is someone to ask;
    /// when it fails or finds nothing, that is said and the value is typed.
    pub fn choose<T: From<String>>(
        &mut self,
        value: Option<T>,
        question: &str,
        flag: &str,
        list: impl FnOnce() -> anyhow::Result<Vec<Choice>>,
    ) -> anyhow::Result<T> {
        if let Some(value) = value {
            return Ok(value);
        }
        if !self.can_ask() {
            bail!("{}", unanswered(flag));
        }
        let choices = match list() {
            Ok(choices) => choices,
            Err(error) => {
                note(&format_args!("{error:#}"));
                Vec::new()
            }
        };
        if choices.is_empty() {
            return self.required(None, question, flag, Some(PathCompleter));
        }
        let answer = match &mut self.answers {
            Answers::Nobody => unreachable!("can_ask checked"),
            Answers::Scripted(answers) => {
                let answer = next(answers, flag)?;
                match choices.into_iter().find(|choice| choice.label == answer) {
                    Some(choice) => choice.value,
                    None => answer,
                }
            }
            Answers::Terminal => {
                let mut entries: Vec<Entry> = choices.into_iter().map(Entry::Choice).collect();
                entries.push(Entry::Other);
                match Select::new(question, entries)
                    .with_scorer(&keep_other)
                    .prompt()
                {
                    Ok(Entry::Choice(choice)) => choice.value,
                    Ok(Entry::Other) => {
                        return self.required(None, question, flag, Some(PathCompleter));
                    }
                    Err(error) => return Err(interrupted(error, flag)),
                }
            }
        };
        Ok(expand(answer)?.into())
    }
}

/// A list entry, as [`Select`] shows it.
enum Entry {
    Choice(Choice),
    Other,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Choice(choice) => f.write_str(&choice.label),
            Self::Other => f.write_str(OTHER),
        }
    }
}

/// The list's fuzzy filter, except that [`Entry::Other`] matches anything,
/// last: typing what the list does not hold leaves the way to type it.
fn keep_other(input: &str, entry: &Entry, label: &str, index: usize) -> Option<i64> {
    match entry {
        Entry::Choice(_) => Select::<Entry>::DEFAULT_SCORER(input, entry, label, index),
        Entry::Other => Some(i64::MIN),
    }
}

fn unanswered(flag: &str) -> String {
    format!("{flag} is required, and there is no terminal to ask for it on")
}

fn next(answers: &mut VecDeque<String>, flag: &str) -> anyhow::Result<String> {
    answers
        .pop_front()
        .ok_or_else(|| anyhow::anyhow!("no answer for {flag}"))
}

/// One line typed at the terminal, trimmed: pre-filled with `initial`, and
/// paths completed on Tab with `paths`.
fn ask(
    question: &str,
    flag: &str,
    initial: Option<&str>,
    paths: Option<PathCompleter>,
) -> anyhow::Result<String> {
    let mut text = Text::new(question);
    if let Some(initial) = initial {
        text = text.with_initial_value(initial);
    }
    if let Some(paths) = paths {
        text = text
            .with_autocomplete(paths)
            .with_help_message("Tab completes a path");
    }
    text.prompt()
        .map(|answer| answer.trim().to_string())
        .map_err(|error| interrupted(error, flag))
}

/// Esc or Ctrl-C at a prompt: nothing has been done yet, and the flag is how
/// to answer without one.
fn interrupted(error: InquireError, flag: &str) -> anyhow::Error {
    match error {
        InquireError::OperationCanceled | InquireError::OperationInterrupted => {
            anyhow::anyhow!("cancelled at the prompt for {flag}; nothing was done")
        }
        InquireError::NotTTY => anyhow::anyhow!("{}", unanswered(flag)),
        other => anyhow::Error::new(other).context(format!("asking for {flag}")),
    }
}

/// A leading `~` expanded, as the shell would have on the command line.
fn expand(answer: String) -> anyhow::Result<String> {
    if !answer.starts_with('~') {
        return Ok(answer);
    }
    Ok(home::expand_tilde(Path::new(&answer))?
        .to_string_lossy()
        .into_owned())
}

/// Tab completion of a path, as a shell's: the entries of the directory typed
/// so far that start with what follows its last `/`, directories with a
/// trailing `/`.
#[derive(Clone)]
struct PathCompleter;

impl Autocomplete for PathCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        Ok(path_suggestions(input))
    }

    /// The highlighted suggestion; else the longest prefix every suggestion
    /// shares, when it adds to the input.
    fn get_completion(
        &mut self,
        input: &str,
        highlighted: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        if highlighted.is_some() {
            return Ok(highlighted);
        }
        let suggestions = path_suggestions(input);
        let shared = suggestions.iter().skip(1).fold(
            suggestions.first().cloned().unwrap_or_default(),
            |a, b| {
                a.chars()
                    .zip(b.chars())
                    .take_while(|(x, y)| x == y)
                    .map(|(x, _)| x)
                    .collect()
            },
        );
        Ok((shared.len() > input.len()).then_some(shared))
    }
}

/// [`PathCompleter`]'s suggestions for `input`, spelled as typed (a `~`
/// kept), sorted. A URL has none; hidden entries only once a `.` is typed.
fn path_suggestions(input: &str) -> Vec<String> {
    if input.contains("://") {
        return Vec::new();
    }
    let (typed_dir, prefix) = match input.rfind('/') {
        Some(i) => input.split_at(i + 1),
        None => ("", input),
    };
    let dir = match typed_dir {
        "" => Path::new(".").to_path_buf(),
        typed => match home::expand_tilde(Path::new(typed)) {
            Ok(dir) => dir,
            Err(_) => return Vec::new(),
        },
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut suggestions: Vec<String> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with(prefix) || (name.starts_with('.') && !prefix.starts_with('.')) {
                return None;
            }
            let slash = if entry.path().is_dir() { "/" } else { "" };
            Some(format!("{typed_dir}{name}{slash}"))
        })
        .collect();
    suggestions.sort();
    suggestions
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn listed() -> anyhow::Result<Vec<Choice>> {
        Ok(vec![Choice {
            label: "devnet-1".to_string(),
            value: "https://example.com/devnet-1".to_string(),
        }])
    }

    #[test]
    fn a_given_value_is_neither_asked_for_nor_listed() {
        let mut nobody = Prompt::nobody();
        let value = nobody.text(Some("x".to_string()), "Q", "--flag <X>");
        assert_eq!(value.unwrap(), "x");
        let value: String = nobody
            .choose(Some("x".to_string()), "Q", "--flag <X>", || {
                unreachable!("nothing to list for a given value")
            })
            .unwrap();
        assert_eq!(value, "x");
    }

    #[test]
    fn with_nobody_to_ask_the_flag_is_named_and_nothing_listed() {
        let err = Prompt::nobody()
            .text::<String>(None, "Q", "--flag <X>")
            .unwrap_err()
            .to_string();
        assert!(err.contains("--flag <X> is required"), "{err}");
        let err = Prompt::nobody()
            .choose::<String>(None, "Q", "--flag <X>", || {
                unreachable!("nobody to show it")
            })
            .unwrap_err()
            .to_string();
        assert!(err.contains("--flag <X> is required"), "{err}");
    }

    #[test]
    fn a_choice_answers_its_value_and_anything_else_is_typed() {
        let mut prompt = Prompt::scripted(&["devnet-1", "some/dir"]);
        let picked: String = prompt.choose(None, "Q", "--dir", listed).unwrap();
        assert_eq!(picked, "https://example.com/devnet-1");
        let typed: PathBuf = prompt.choose(None, "Q", "--dir", listed).unwrap();
        assert_eq!(typed, PathBuf::from("some/dir"));
    }

    #[test]
    fn a_failed_listing_falls_back_to_typing() {
        let typed: String = Prompt::scripted(&["some/dir"])
            .choose(None, "Q", "--dir", || bail!("offline"))
            .unwrap();
        assert_eq!(typed, "some/dir");
    }

    #[test]
    fn a_path_is_its_default_unless_given_or_edited() {
        let value: String = Prompt::nobody().path(None, "Q", "DIR", "/d").unwrap();
        assert_eq!(value, "/d");
        let mut prompt = Prompt::scripted(&["", "/edited"]);
        let accepted: String = prompt.path(None, "Q", "DIR", "/d").unwrap();
        assert_eq!(accepted, "/d");
        let edited: String = prompt.path(None, "Q", "DIR", "/d").unwrap();
        assert_eq!(edited, "/edited");
        let given: String = prompt
            .path(Some("/given".to_string()), "Q", "DIR", "/d")
            .unwrap();
        assert_eq!(given, "/given");
    }

    #[test]
    fn a_leading_tilde_is_expanded() {
        let value: String = Prompt::scripted(&["~/networks/devnet-1"])
            .text(None, "Q", "DIR")
            .unwrap();
        assert!(!value.starts_with('~'), "{value}");
        assert!(value.ends_with("/networks/devnet-1"), "{value}");
    }

    #[test]
    fn paths_complete_as_a_shell_completes_them() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("networks/devnet-1")).unwrap();
        std::fs::create_dir_all(tmp.path().join("networks/devnet-2")).unwrap();
        std::fs::create_dir_all(tmp.path().join("networks/.hidden")).unwrap();
        std::fs::write(tmp.path().join("networks/notes.txt"), "").unwrap();
        let root = format!("{}/", tmp.path().display());

        assert_eq!(path_suggestions(&root), [format!("{root}networks/")]);
        assert_eq!(
            path_suggestions(&format!("{root}networks/")),
            [
                format!("{root}networks/devnet-1/"),
                format!("{root}networks/devnet-2/"),
                format!("{root}networks/notes.txt"),
            ]
        );
        assert_eq!(
            path_suggestions(&format!("{root}networks/.")),
            [format!("{root}networks/.hidden/")]
        );
        assert_eq!(
            PathCompleter
                .get_completion(&format!("{root}networks/d"), None)
                .unwrap(),
            Some(format!("{root}networks/devnet-"))
        );
        assert!(path_suggestions("https://github.com/").is_empty());
    }
}
