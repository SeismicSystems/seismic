# Architecture Diagrams

Freeform architecture diagrams live here as `.excalidraw` files (open JSON), with a rendered `.png` alongside each one for embedding in markdown and hosted docs.

## Conventions

- **Structured diagrams** (flowcharts, sequence diagrams, state machines) don't belong here — write them as ` ```mermaid ` blocks directly in the relevant markdown file. GitHub and the VS Code preview (with the recommended `bierner.markdown-mermaid` extension) render them natively.
- **Freeform diagrams** (spatial layouts, trust boundaries, data-flow overviews) go here as `<name>.excalidraw` + `<name>.png`. Embed the PNG in docs: `![measurement admission](./diagrams/measurement-admission-pipeline.png)`.
- The `.excalidraw` file is the source of truth. If you edit it, re-render the PNG in the same commit so they never drift.

## Ways to view & edit

All of these read/write the same format — pick whichever is closest. No account or subscription needed.

1. **VS Code** — the recommended `pomdtr.excalidraw-editor` extension opens `.excalidraw` files as a live editor on click. Export PNG from the editor menu, or use the headless renderer below.
2. **Browser** — drag the file into [excalidraw.com](https://excalidraw.com), edit, then export the scene back over the file (and re-render the PNG).
3. **Local live canvas server** — `npx -y mcp-excalidraw-server start`, open `http://127.0.0.1:3000`, then `npx -y mcp-excalidraw-server import <file>` to load and `export --out <file>` to save back. Useful for watching an agent edit in real time; note the canvas is in-memory, so export before stopping the server.
4. **Headless render (PNG)** — the vendored skill ships a Playwright renderer; this is how the committed PNGs are produced:

   ```bash
   cd .claude/skills/excalidraw-diagram/references
   uv sync && uv run playwright install chromium   # first time only
   uv run python render_excalidraw.py ../../../docs/diagrams/<name>.excalidraw
   ```

You can also just ask Claude Code to create or revise a diagram — the vendored [excalidraw-diagram skill](../../.claude/skills/README.md) handles the methodology and rendering.
