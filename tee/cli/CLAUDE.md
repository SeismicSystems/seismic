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
- **Ask for what is missing, and help answer.** A required value left out is
  asked for on a terminal, rather than refused — offered as a list filtered
  as it is typed when its usual values can be listed (published networks,
  image releases), else typed with paths completed on Tab. A blank question
  is no help: a value that can be neither listed nor completed stays a
  refusal. Without a terminal, fail naming the flag that supplies it: a
  prompt never blocks a script, and every prompt has a flag that answers it.
  Through `seismic_tee_common::prompt::Prompt`: the clap argument is
  optional, and its help says it is required.
- **Colour carries meaning.** Through `anstream`/`anstyle` only, which decide
  once whether to render (`NO_COLOR`, a non-terminal stream, `TERM=dumb`):
  red for what is dangerous or failed (`confirm::danger`), dimmed for
  narration beside the command's own report (`seismic_tee_common::NOTE`).
  Never the only signal; the text reads the same without it.
- **Derive what can be derived.** A value that follows from another input is
  optional and defaults to it, rather than asked for twice. When that input
  was itself asked for, the derived value may be offered pre-filled, to
  accept with Enter (`network init`'s directory after its name); given as a
  flag, it never prompts.
