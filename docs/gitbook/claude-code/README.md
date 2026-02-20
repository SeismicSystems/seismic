---
description: Use Claude Code effectively with Seismic's privacy-first SDKs
icon: terminal
---

# Claude Code

[Claude Code](https://docs.anthropic.com/en/docs/claude-code/overview) is Anthropic's CLI tool for agentic coding with Claude. It reads your project files, runs commands, and writes code — but it only knows what's in its training data and your project context.

Seismic's SDKs and Solidity extensions are new. Claude doesn't know the correct function names, shielded type syntax, or encryption patterns out of the box. Without guidance, it will hallucinate standard Ethereum patterns that don't work on Seismic.

## How CLAUDE.md templates fix this

A `CLAUDE.md` file in your project root is automatically loaded by Claude Code at the start of every session. It acts as a persistent instruction set — telling Claude about your project's APIs, conventions, and common pitfalls.

The templates on this site give Claude the knowledge it needs to write correct Seismic code:

- Shielded types and their compiler behavior
- SDK-specific imports, client construction, and contract interaction patterns
- Transaction encryption, signed reads, and encrypted events
- Network configuration (chain IDs, RPC URLs)
- Common mistakes that waste debugging time

## Choose your template

| Template                                           | Good for                                       | Language   |
| -------------------------------------------------- | ---------------------------------------------- | ---------- |
| [Seismic Solidity](templates/seismic-solidity.md)  | Smart contract development with shielded types | Solidity   |
| [Seismic Viem](templates/seismic-viem.md)          | TypeScript dapp backends and scripts           | TypeScript |
| [Seismic React](templates/seismic-react.md)        | React frontend development                     | TypeScript |
| [Seismic Alloy (Rust)](templates/seismic-alloy.md) | Rust dapp development                          | Rust       |
| [Seismic Python](templates/seismic-python.md)      | Python scripts and backends                    | Python     |

## Quick setup

1. Pick the template(s) that match your stack
2. Copy the template content
3. Save it as `CLAUDE.md` in your project root
4. If your project spans multiple SDKs (e.g., Solidity contracts + TypeScript frontend), concatenate the relevant templates into one file
5. Start Claude Code — it reads `CLAUDE.md` automatically

{% hint style="info" %}
**For Seismic contributors:** Seismic's internal repositories (seismic-reth, seismic-foundry, seismic-client, etc.) already have their own CLAUDE.md files tailored for contributor workflows. The templates here are for **external dapp developers** building on Seismic.
{% endhint %}

## Optional: Workflow skills

Claude Code also supports [skills](workflow-skills.md) — reusable task instructions you can invoke with slash commands. We provide a few optional skills for common Seismic workflows like deploying contracts and debugging shielded transactions.
