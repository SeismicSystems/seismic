# Writing the TEE docs

- **One file per topic, in two parts.**
  - The body: what the thing is and how it behaves today.
  - A single trailing `## Design rationale`: the alternatives weighed and set
    aside.
  - No argument appears in both halves. When a reader who misses a decision
    would undo it, the body states the rule in one sentence and links down to
    the rationale.
- **Decision records in [decisions/](decisions/) are point-in-time.** Never
  update one as the design moves. Link it from the design rationale of the doc
  whose choice it settled.
- **Cross-doc routing lives only in [the README's doc list](README.md#the-docs).**
  A doc opens with its own scope and its normative boundary (which spec owns the
  byte-exact rules). It links a sibling inline, where the reader needs it, and
  never keeps an index of its own: two hand-written summaries of one doc drift.
- **Diagrams** follow [diagrams/README.md](diagrams/README.md).
