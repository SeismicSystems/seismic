---
name: audit-commits
description: Wrap audit fix work into two commits (tests + fix)
disable-model-invocation: true
---

# Audit Commits Workflow

The work for an audit fix has been completed. Now we need to wrap it into two well-structured commits following the test-first approach.

## Workflow

1. **Review the current changes**
  - Run `git status` and `git diff` to understand all modifications
  - Identify which changes are test additions vs. fix implementations

2. **Stage and commit the tests first**
  - Stage only the test files/changes that expose the faulty behavior
  - Commit as `test[optional scope]: <description>` (Conventional Commits)

3. **Stage and commit the fix**
  - Stage all remaining changes (the actual fix)
  - Commit as `fix[optional scope]: <description>`, with an optional body explaining the approach and any trade-offs

Do not mention that commits were authored or co-authored by Claude.

## Invariants

- The test commit comes FIRST chronologically
- Tests must fail when checked out independently (before the fix)
- The fix commit must make all tests pass — run the full test suite before committing it
- If you need to adjust anything, maintain the two-commit structure
