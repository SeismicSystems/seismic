# Architecture Diagrams

This README is the diagram-conventions doc for the whole repo; it lives here because the TEE docs are currently the heaviest diagram users. Diagrams sit next to the docs that embed them (this directory serves [`docs/tee/`](../)).

## Conventions

- **Structured diagrams** (flowcharts, sequence diagrams, state machines, data-flow graphs) — write them as ` ```mermaid ` blocks directly in the relevant markdown file. GitHub and the VS Code preview (with the recommended `bierner.markdown-mermaid` extension) render them natively. This is the default: reach for mermaid first, and only leave it when auto-layout can't express the diagram.
- **Spatial diagrams** (layouts where position carries meaning: trust boundaries, side-by-side comparisons, zoned data-flow overviews, embedded code artifacts) — hand-authored **structured SVG**, committed as `<name>.svg` next to the doc and embedded directly: `![founding reorder](diagrams/founding-reorder-vs-split.svg)`. GitHub renders SVG images natively, so the source is also the render — there is no exported copy to keep in sync.
- The general-architecture PNGs in the gitbook assets (`seismic-node.png`, `rpc-evm-storage.png`, `tries.png`, embedded by [`overview/how-seismic-works.md`](../../gitbook/overview/how-seismic-works.md)) predate the convention and have no editable source; recreate as mermaid or SVG if one ever needs editing.

## Writing structured SVG

"Structured" means the file is organized for editing, not exported geometry:

- All styling in one `<style>` block — CSS classes name the visual vocabulary (`.node.authored`, `.pin`, `.flow`) so a color or stroke change is one edit.
- Each node is a `<g class="node …" transform="translate(x,y)">` group containing a rect/ellipse and its text — moving a node is one transform edit.
- Solid vs dashed strokes carry semantics (declare them in a legend); arrowheads via `<marker>` defs.
- XML comments delimit sections; descriptive class names over repeated inline attributes.
- Gotcha: XML collapses leading whitespace in `<tspan>`s — indent code-block lines with explicit per-line `x` offsets.

## Edit → render → view

You cannot judge an SVG from its source: text overflow, label collisions, and arrow misses are only visible rendered. After every edit, render and actually look at it:

```bash
rsvg-convert -w 2000 -o /tmp/check.png diagrams/<name>.svg   # brew install librsvg
```

or open the file in any browser, or use the recommended `jock.svg` VS Code extension for a live source+preview split. Agents editing diagrams must render and view the result before considering the edit done — expect 2–3 fix cycles on non-trivial changes.
