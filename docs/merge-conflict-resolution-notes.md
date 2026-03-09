# Merge Conflict Resolution Notes

**Branch:** `docs/restructure-information-architecture` merging `main`
**Date:** 2026-03-09

## Strategy

- **Python client docs (`docs/gitbook/clients/python/`):** Take main's version for ALL files.
  Main has 15 commits of careful auditing/polishing work. The branch only imported these docs
  from main and made no substantive changes to the python content itself.
- **Everything else (SUMMARY.md structure, etc.):** Take the branch's version, since that's
  where the docs restructuring work lives.

## Did the branch author make valuable python doc changes?

**No.** The branch's python doc commits are:
- `8066aa8` — bulk import of python docs from main
- `67caa45` — "update python" — 24K lines added, but this is just the full import, not new content
- `ea285e1` — "clients" — organizational commit

Main's python doc commits (15 total) include:
- Auditing every section (ABIs, SRC20, Namespaces, Contracts, API Reference, Precompiles)
- Merging Examples into Guides (deleting `examples/` folder)
- Fixing misleading AI-generated content
- Adding `hex_to_bytes` function + docs
- Making types/tx-types sections beefier
- Adding icons for SRC20 sections
- Simplifying SRC20 intelligence section

## Conflict Categories

### UU (Both Modified) — 44 files in `clients/python/`
All python content files. Resolution: **Take main's version.**
The branch has stale imported versions; main has audited versions.

### UD (Branch Modified, Main Deleted) — 10 files
- `examples/README.md`, `async-patterns.md`, `basic-wallet-setup.md`,
  `shielded-write-complete.md`, `signed-read-pattern.md`, `src20-workflow.md`
- `src20/directory/README.md`, `check-has-key.md`, `get-viewing-key.md`, `register-viewing-key.md`
- `src20/event-watching/watch-src20-events-with-key.md`

Resolution: **Accept main's deletions.**
- `examples/*` was merged into `guides/*` on main (PR #43)
- `src20/directory/*` was reorganized into `src20/intelligence-providers/*` on main (PR #53)
- `watch-src20-events-with-key.md` moved to intelligence-providers section

### AA (Both Added) — 2 files
- `api-reference/types/hex-to-bytes.md` — both sides added (main via PR #33)
- `namespaces/methods/encode-shielded-calldata.md` — both sides added

Resolution: **Take main's version** (audited quality).

### UU — SUMMARY.md
Resolution: **Take branch's version.**

The branch's SUMMARY.md is actually MORE correct than main's:
- Main's SUMMARY references `client-libraries/seismic-python/...` paths but files live at
  `clients/python/...` — so main's SUMMARY has broken links
- Main's SUMMARY still lists `examples/` section and `src20/directory/` section, but those
  files were deleted/moved on main itself (PRs #43, #53) — main's SUMMARY is outdated
- Branch's SUMMARY has the correct paths AND correctly reflects the file reorganization
  (no `examples/`, has `intelligence-providers/` instead of `directory/`, merged guides)

Note: `encode_shielded_calldata.md` exists as a file but is not in either SUMMARY.md.
Can be added later if needed.
