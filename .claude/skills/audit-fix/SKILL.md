---
name: audit-fix
description: Fix security audit findings with a test-first approach
disable-model-invocation: true
---

# Audit Fix Workflow

An auditor has found a problem in this codebase. We are tasked with fixing the issue. The user will paste the auditor's concerns after invoking this command.

## Workflow

Follow these steps for all audit fixes:

1. **Write tests that expose the faulty behaviour**
   - Create comprehensive tests that demonstrate the vulnerability or issue
   - Tests should clearly show the problematic behavior

2. **Run tests against the current code**
   - Execute the tests to confirm they fail as expected
   - This validates that our tests accurately capture the issue
   - The entire test suite is run in .github/workflows, usually called "seismic.yml" but sometimes "test.yml" or "ci.yml"

3. **Commit the changes**
   - All of the tests we have added should be in a single commit
   - Important to not commit changes until you have run the added tests, and they have all failed

4. **Implement the fixes**
   - If the auditor clearly suggested a fix, follow that approach
   - Otherwise, think critically about the best solution
   - Apply the fix to resolve the issue

5. **Run tests again**
   - Execute the tests to verify the fix works
   - Repeat steps 4 & 5 until all tests pass

6. **Commit the fix**
   - It's important that the full test suite passes before you commit your changes
   - If you realized you needed to add more changes, make sure your finished product still contains two commits: (1) the test commit and (2) the actual fix

## Communication

As you make progress, explain:
- What we will do to fix the issue
- Why we're taking this approach
- Trade-offs or considerations in the solution

## Commit message format
It's important that your commit messages are good, and accurately describe the changes. We also want the formats of these commits to match the "Conventional Commits". A summary of these rules:
- Structure: Every commit must follow the format: <type>[optional scope]: <description> followed by an optional body and footer(s).
- Core Types for this task are:
  - fix: we made the actual fix
  - test: we added tests that exposed the faulty behaviour
- Breaking Changes: Indicated by a ! after the type/scope (e.g., feat!:) or by including BREAKING CHANGE: as a footer. This correlates to a MAJOR in SemVer.
- Optional Scopes: A noun in parentheses providing additional context (e.g., feat(api):).
- Other Types: Allows for types like feat:, build:, chore:, ci:, docs:, style:, refactor:, perf:, and test:.
- Format Rules: * A blank line must separate the description from the body and the body from the footer.
- Footers must use a hyphenated token (e.g., Reviewed-by:) followed by a separator.
- Don't mention this was authored or co-authored by Claude

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
If unsure whether the current repo is a fork, check: `~/code/seismic/internal/crates/commit-tracker/repos.toml`
