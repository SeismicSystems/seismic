# seismic-tee CLI

Interface design follows [clig.dev](https://clig.dev). The rules below are
the ones this CLI holds itself to most, and where they meet code here; for
anything they don't cover, follow clig.dev.

## Interface

- **The CLI teaches itself.** The README covers only what the CLI cannot
  show.
  - Every command's help ends in an `Examples:` block (clap `after_help`, as
    `ctx list` has) of real invocations, the common case first. Each one is
    also added to `the_documented_invocations_parse` in
    `seismic-tee/src/main.rs`, so an example cannot drift from the parser.
  - Output says what to run next, through `seismic_tee_common::next_step`:
    a refusal names the command to run instead, a success the next one.
- **Ask for what is missing.** A required value left out is prompted for when
  stdin is a terminal, rather than refused. Without a terminal, fail naming
  the flag that supplies it: a prompt never blocks a script, and every
  prompt has a flag that answers it.
- **Colour carries meaning.** Through `anstream`/`anstyle` only, which decide
  once whether to render (`NO_COLOR`, a non-terminal stream, `TERM=dumb`):
  red for what is dangerous or failed (`confirm::danger`), dimmed for
  narration beside the command's own report (`seismic_tee_context::NOTE`).
  Never the only signal; the text reads the same without it.
- **Derive what can be derived.** A value that follows from another input is
  optional and defaults to it, rather than asked for twice.
