---
description: Ready-to-use CLAUDE.md templates for each Seismic SDK
icon: file-code
---

# CLAUDE.md Templates

A `CLAUDE.md` file is a markdown file in your project root that Claude Code reads at the start of every session. It tells Claude about your project's APIs, patterns, and constraints — filling the knowledge gap for libraries that aren't in Claude's training data.

## What's in each template

Every template follows the same structure:

| Section              | Purpose                                                         |
| -------------------- | --------------------------------------------------------------- |
| **Seismic Overview** | What Seismic is and how it differs from Ethereum                |
| **Key Concepts**     | Shielded types, TxSeismic, precompiles, signed reads            |
| **SDK**              | Installation, imports, and key exports for the specific library |
| **Core Patterns**    | Code snippets showing the correct way to use the SDK            |
| **Common Mistakes**  | Gotchas that Claude would otherwise get wrong                   |
| **Networks**         | Chain IDs, RPC URLs, and faucet links                           |
| **Links**            | Pointers to the full SDK documentation                          |

## Setup

1. Pick the template that matches your primary SDK
2. Copy the entire code block from the template page
3. Save it as `CLAUDE.md` in your project root (next to `package.json`, `Cargo.toml`, etc.)

That's it. Claude Code detects the file automatically.

## Combining templates

If your project uses multiple SDKs — for example, Solidity contracts with a TypeScript frontend — concatenate the relevant templates into a single `CLAUDE.md` file:

```bash
# Example: Solidity + Viem project
cat claude-solidity.md claude-viem.md > CLAUDE.md
```

The shared sections (Seismic Overview, Key Concepts, Networks) are intentionally short and repeated across templates, so duplicating them is fine. Claude handles redundant context gracefully.

## Customizing

After pasting a template, you should edit a few things:

- **Project name** — Replace the placeholder at the top with your actual project name
- **Contract addresses** — Add your deployed contract addresses
- **ABI paths** — Point to where your ABIs live in the project
- **Custom patterns** — Add any project-specific conventions Claude should follow

## Templates

- [Seismic Solidity](seismic-solidity.md) — Smart contract development with shielded types
- [Seismic Viem](seismic-viem.md) — TypeScript dapp development
- [Seismic React](seismic-react.md) — React frontend development
- [Seismic Alloy (Rust)](seismic-alloy.md) — Rust dapp development
- [Seismic Python](seismic-python.md) — Python scripts and backends
