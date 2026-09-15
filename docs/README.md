# Seismic Docs

Developer docs for the Seismic protocol and workspace:

- [glossary.md](glossary.md) — key concepts: FlaggedStorage, TxSeismic, Mercury Spec, SeismicHost
- [key-schedule.md](key-schedule.md) — every key derivation and its domain-separation label, by layer
- [tee/](tee/README.md) — how Seismic runs in a TEE: what identifies a network, how one is founded, and how a node is let in. Its README is the one-pass orientation, and links onward to the deep docs and to the normative specs held in other repos
- [gitbook/](gitbook/) — source for the user-facing docs (docs.seismic.systems); [gitbook/reference/repos.md](gitbook/reference/repos.md) covers all repos, fork management, and dependency flow
- [claude-code-setup.md](claude-code-setup.md) — Claude Code skills setup and symlink instructions

Conventions:

- [tee/diagrams/README.md](tee/diagrams/README.md) — diagram conventions for the whole repo: Mermaid in markdown for structured diagrams, hand-authored structured SVG next to their docs for spatial ones; follow this when creating or editing diagrams
