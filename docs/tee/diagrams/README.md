# Architecture Diagrams

Freeform architecture diagrams live as `.excalidraw` files (open JSON), with a rendered `.png` alongside each one for embedding in markdown and hosted docs. This README is the conventions doc for the whole repo; it lives here because the TEE docs are currently the only ones with `.excalidraw` sources. Diagrams sit next to the docs that embed them (this directory serves [`docs/tee/`](../)); the general-architecture PNGs live in [`docs/diagrams/`](../../diagrams/).

## Conventions

- **Structured diagrams** (flowcharts, sequence diagrams, state machines) don't belong here — write them as ` ```mermaid ` blocks directly in the relevant markdown file. GitHub and the VS Code preview (with the recommended `bierner.markdown-mermaid` extension) render them natively.
- **Freeform diagrams** (spatial layouts, trust boundaries, data-flow overviews) are committed as `<name>.excalidraw` + `<name>.png`. Embed the PNG in docs: `![founding reorder](diagrams/founding-reorder-vs-split.png)`.
- **Why both formats**: the `.excalidraw` is the editable source — open JSON, diffable, precise programmatic edits. The `.png` is what humans *and agents* should read: a model analyzes the rendered image (layout, arrows, color-coding, at a glance) far better than the source, which is thousands of lines of geometry JSON that may not even fit in context. When asking Claude to review or discuss a diagram, point it at the PNG; it only needs the JSON to make an edit.
- The `.excalidraw` file is the source of truth. If you edit it, re-render the PNG in the same commit so they never drift.
- The general-architecture PNGs in [`docs/diagrams/`](../../diagrams/) (`seismic-node.png`, `rpc-evm-storage.png`, `tries.png`) predate the convention and have no `.excalidraw` source; recreate the source if one ever needs editing.

## Ways to view & edit

All of these read/write the same format — pick whichever is closest. No account or subscription needed.

1. **VS Code** — the recommended `pomdtr.excalidraw-editor` extension opens `.excalidraw` files as a live editor on click. Export PNG from the editor menu, or use the headless renderer below.
2. **Browser** — drag the file into [excalidraw.com](https://excalidraw.com), edit, then export the scene back over the file (and re-render the PNG).
3. **Local live canvas server** — `npx -y mcp-excalidraw-server start`, open `http://127.0.0.1:3000`, then `npx -y mcp-excalidraw-server import <file>` to load and `export --out <file>` to save back. Useful for watching an agent edit in real time; note the canvas is in-memory, so export before stopping the server.
4. **Headless render (PNG)** — the vendored skill ships a Playwright renderer; this is how the committed PNGs are produced:

   ```bash
   cd .claude/skills/excalidraw-diagram/references
   uv sync && uv run playwright install chromium   # first time only
   uv run python render_excalidraw.py ../../../docs/tee/diagrams/<name>.excalidraw
   ```

You can also just ask Claude Code to create or revise a diagram — the vendored [excalidraw-diagram skill](../../../.claude/skills/README.md) handles the methodology and rendering.
