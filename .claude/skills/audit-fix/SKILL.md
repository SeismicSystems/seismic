---
name: audit-fix
description: Fix security audit findings with a test-first approach
disable-model-invocation: true
---

# Audit Fix Workflow

An auditor has found a problem in this codebase. We are tasked with fixing the issue. The user will paste the auditor's concerns after invoking this command.

## Workflow

1. **Write tests that expose the faulty behaviour**
   - Create comprehensive tests that demonstrate the vulnerability or issue

2. **Run tests against the current code**
   - Confirm they fail as expected — this validates that the tests accurately capture the issue
   - The entire test suite is run in .github/workflows, usually called "seismic.yml" but sometimes "test.yml" or "ci.yml"

3. **Commit the tests**
   - All added tests go in a single `test[optional scope]:` commit (Conventional Commits)
   - Do not commit until the added tests have run and all failed

4. **Implement the fix**
   - If the auditor clearly suggested a fix, follow that approach
   - Otherwise, think critically about the best solution

5. **Run tests again**
   - Repeat steps 4 & 5 until all tests pass

6. **Commit the fix**
   - The full test suite must pass before committing
   - Commit as `fix[optional scope]: <description>` with an optional body explaining approach and trade-offs
   - The finished product must be exactly two commits: (1) the tests, (2) the fix

Do not mention that commits were authored or co-authored by Claude.

## Priorities (in order)

### #1: Correctness
The fix must be correct and fully address the audit finding. Security and correctness cannot be compromised.

### #2: Conciseness
The fix should be as concise as possible. Avoid over-engineering or unnecessary changes.

### #3: Minimize Upstream Modifications
Many codebases are forks of upstream repositories. We want our diffs against the upstream code to perform well when we merge in upstream, which we do somewhat regularly.

**Upstream branch names:**
- Most repos: `main`
- seismic-foundry: `master`
- seismic-solidity: `develop`

**Checking if a repo is forked:**
If unsure whether the current repo is a fork, check for an upstream remote (`git remote -v`) or check the repo list in `<workspace-root>/seismic/internal/crates/commit-tracker/repos.toml` (where `<workspace-root>` is the parent directory containing all seismic repos as siblings).
