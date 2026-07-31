# Claude Code Skills

Skills in this directory are loaded automatically by Claude Code for anyone working in this repo. Most are seismic-authored single `SKILL.md` files.

## excalidraw-diagram (vendored)

Vendored from [coleam00/excalidraw-diagram-skill](https://github.com/coleam00/excalidraw-diagram-skill) so agents can create and revise the repo's `.excalidraw` diagrams (conventions in [docs/tee/diagrams/README.md](../../docs/tee/diagrams/README.md)) and render/validate their own output headlessly.

**Local patch vs upstream:** `references/render_template.html` pins `deps=@braintree/sanitize-url@7.1.1` on the esm.sh import — the CDN 404s on the default v6 build, which breaks rendering. Re-apply (or drop, if fixed upstream) when re-vendoring; everything else is a byte-for-byte upstream copy, so refreshing is a plain `rsync` (exclude `.git`, `.venv`, `__pycache__`).

**Renderer setup** (first use only; requires [uv](https://docs.astral.sh/uv/)):

```bash
cd .claude/skills/excalidraw-diagram/references
uv sync && uv run playwright install chromium
```
