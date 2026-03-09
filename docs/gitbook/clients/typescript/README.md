---
description: TypeScript client libraries for Seismic
icon: js
---

# TypeScript

Seismic provides two TypeScript packages:

| Package | Purpose | Use when |
|---------|---------|----------|
| [seismic-viem](viem/README.md) | Low-level SDK built on viem 2.x | Server-side, scripts, non-React apps |
| [seismic-react](react/README.md) | React hooks layer built on seismic-viem + wagmi | React applications |

`seismic-react` depends on `seismic-viem` — install both if you're building a React app, or just `seismic-viem` for everything else.
