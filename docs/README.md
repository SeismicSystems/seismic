# Seismic Docs

Developer docs for the Seismic protocol and workspace:

- [architecture.md](architecture.md) — diagrams: Seismic node, RPC/EVM/storage interactions, tries + SeismicTx (rendered PNGs in [diagrams/](diagrams/))
- [glossary.md](glossary.md) — key concepts: FlaggedStorage, TxSeismic, Mercury Spec, SeismicHost
- [key-schedule.md](key-schedule.md) — every key derivation and its domain-separation label, by layer
- [language-and-vm.md](language-and-vm.md) — Mercury EVM spec: shielded types, CLOAD/CSTORE, FlaggedStorage, arrays, casting
- [tee/network-founding.md](tee/network-founding.md) — network founding: boot-chain sequencing, the summit key holder, what `network_id` pins
- [tee/diagrams/README.md](tee/diagrams/README.md) — diagram conventions: Mermaid in markdown for structured diagrams, hand-authored structured SVG next to their docs for spatial ones; follow this when creating or editing diagrams
- [claude-code-setup.md](claude-code-setup.md) — Claude Code skills setup and symlink instructions
- [gitbook/](gitbook/) — source for the user-facing docs (docs.seismic.systems); [gitbook/reference/repos.md](gitbook/reference/repos.md) covers all repos, fork management, and dependency flow
