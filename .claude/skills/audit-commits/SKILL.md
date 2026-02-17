---
name: audit-commits
description: Wrap audit fix work into two commits (tests + fix)
---

# Audit Commits Workflow

The work for an audit fix has been completed. Now we need to wrap it into two well-structured commits following the test-first approach.

## Workflow

Follow these steps to create the two commits:

1. **Review the current changes**
  - Run `git status` and `git diff` to understand all modifications
  - Identify which changes are test additions vs. fix implementations

2. **Stage and commit the tests first**
  - Stage only the test files/changes that expose the faulty behavior
  - Create the first commit with the tests

3. **Stage and commit the fix**
  - Stage all remaining changes (the actual fix)
  - Create the second commit with the implementation

## Commit message format

Follow "Conventional Commits" format for both commits:

**Test commit:**
test[optional scope]:

Fix commit:
fix[optional scope]: <description of the fix>

<optional body explaining the approach and any trade-offs>

Rules:

- Structure: type[optional scope]: description followed by optional body and footer(s)
- Core types for this task: test: (for tests) and fix: (for the fix)
- Breaking changes: Add an exclamation mark after type/scope (like fix(api)!:) or use BREAKING CHANGE: footer
- Optional scopes: A noun in parentheses for context, like fix(api):
- A blank line must separate description from body
- The first line (title) of the commit message must be 72 characters or less, per standard Git convention
- Do not mention this was authored or co-authored by Claude

Important Notes

- The test commit should come FIRST chronologically
- Tests should fail when checked out independently (before the fix)
- The fix commit should make all tests pass
- Ensure the full test suite passes before committing the fix
- If you need to adjust anything, maintain the two-commit structure
