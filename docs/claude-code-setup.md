# Claude Code Setup

This guide covers how to set up the Seismic-specific Claude Code skills on your machine so they're available in all repos.

## Skills

This repo ships the following Claude Code skills in `.claude/skills/`:

| Skill            | Command          | Description                                                               |
|------------------|------------------|---------------------------------------------------------------------------|
| `ssolc-tests`    | `/ssolc-tests`   | Build and test seismic-solidity (unit tests, semantic tests, isoltest).   |
| `audit-fix`      | `/audit-fix`     | Walk through fixing a security audit finding with a test-first approach.  |
| `audit-commits`  | `/audit-commits` | Structure completed audit fix work into two clean commits (tests + fix).  |
| `py-docs`        | `/py-docs`       | Write Python SDK documentation pages in GitBook format.                   |

## Installation

There are two ways to install the skills, depending on where you run Claude Code from. You can do one or both.

### Option A: Workspace-level (recommended)

This makes skills available when running Claude Code from the `seismic-workspace` root directory (the same level where `CLAUDE.md` is already symlinked). From that directory:

```sh
cd /path/to/seismic-workspace

mkdir -p .claude/skills
for skill in seismic/.claude/skills/*/; do
  ln -sf "../seismic/.claude/skills/$(basename "$skill")" .claude/skills/"$(basename "$skill")"
done
```

To symlink a single skill:

```sh
cd /path/to/seismic-workspace

mkdir -p .claude/skills
ln -sf "../seismic/.claude/skills/SKILL_NAME" .claude/skills/SKILL_NAME
```

### Option B: Global (`~/.claude`)

This makes skills available in every Claude Code session, regardless of working directory:

```sh
cd /path/to/seismic-workspace

mkdir -p ~/.claude/skills
for skill in seismic/.claude/skills/*/; do
  ln -sf "$(pwd)/$skill" ~/.claude/skills/"$(basename "$skill")"
done
```

To symlink a single skill:

```sh
cd /path/to/seismic-workspace

mkdir -p ~/.claude/skills
ln -sf "$(pwd)/seismic/.claude/skills/SKILL_NAME" ~/.claude/skills/SKILL_NAME
```

### Verifying

After symlinking, check that the links point back into this repo:

```sh
# workspace-level
ls -la .claude/skills/

# global
ls -la ~/.claude/skills/
```

When you start a new Claude Code session, the skills will appear as available slash commands.

## Usage

In any Claude Code session, type the skill name as a slash command:

- `/ssolc-tests` — get instructions for building and running seismic-solidity tests
- `/audit-fix` — start a guided audit fix workflow (paste the auditor's finding after invoking)
- `/audit-commits` — structure your uncommitted audit fix work into two commits
- `/py-docs` — write Python SDK documentation pages following the established GitBook format
