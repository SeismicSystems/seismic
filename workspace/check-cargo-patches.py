#!/usr/bin/env python3
"""Check that [patch.crates-io] git revs are consistent across all repos.

For each git URL used in patches, verifies that every repo pins the same
rev/tag/branch. Flags any mismatches so you can fix drift before CI breaks.

Usage:
    python workspace/check-patch-sync.py [--workspace-dir /path/to/seismic-workspace]
"""

import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path


# Repos to skip (upstream mirrors, docs snippets, etc.)
SKIP_PATTERNS = [
    "_UPSTREAM",
    "docs/vocs/docs/snippets",
]


def find_cargo_tomls(workspace_dir: Path) -> list[Path]:
    """Find top-level Cargo.toml files that have [patch.crates-io]."""
    results = []
    for child in sorted(workspace_dir.iterdir()):
        if not child.is_dir():
            continue
        if any(pat in child.name for pat in SKIP_PATTERNS):
            continue
        cargo_toml = child / "Cargo.toml"
        if cargo_toml.exists():
            content = cargo_toml.read_text()
            if "[patch.crates-io]" in content:
                results.append(cargo_toml)
    return results


def parse_patches(cargo_toml: Path) -> dict[str, dict]:
    """Parse [patch.crates-io] section, returning {crate_name: {git, rev/tag/branch}}."""
    content = cargo_toml.read_text()

    # Extract the [patch.crates-io] section
    match = re.search(r"\[patch\.crates-io\]\s*\n(.*?)(\n\[|\Z)", content, re.DOTALL)
    if not match:
        return {}

    section = match.group(1)
    patches = {}

    for line in section.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue

        # Match: crate-name = { git = "...", rev = "..." }
        # or:    crate-name = { git = "...", tag = "..." }
        # or:    crate-name = { git = "...", branch = "..." }
        crate_match = re.match(
            r'^([\w-]+)\s*=\s*\{.*?git\s*=\s*"([^"]+)"', line
        )
        if not crate_match:
            continue

        crate_name = crate_match.group(1)
        git_url = crate_match.group(2)

        pin = {}
        rev_match = re.search(r'rev\s*=\s*"([^"]+)"', line)
        tag_match = re.search(r'tag\s*=\s*"([^"]+)"', line)
        branch_match = re.search(r'branch\s*=\s*"([^"]+)"', line)

        if rev_match:
            pin = {"type": "rev", "value": rev_match.group(1)}
        elif tag_match:
            pin = {"type": "tag", "value": tag_match.group(1)}
        elif branch_match:
            pin = {"type": "branch", "value": branch_match.group(1)}

        if pin:
            patches[crate_name] = {"git": git_url, **pin}

    return patches


def normalize_git_url(url: str) -> str:
    """Normalize git URL for comparison (strip trailing .git, lowercase)."""
    url = url.lower().rstrip("/")
    if url.endswith(".git"):
        url = url[:-4]
    return url


def check_sync(workspace_dir: Path) -> list[str]:
    """Check patch consistency. Returns list of error messages."""
    cargo_tomls = find_cargo_tomls(workspace_dir)

    # Group: normalized_git_url -> {crate_name -> [(repo, pin_type, pin_value)]}
    by_git_url: dict[str, dict[str, list[tuple[str, str, str]]]] = defaultdict(
        lambda: defaultdict(list),
    )

    for cargo_toml in cargo_tomls:
        repo_name = cargo_toml.parent.name
        patches = parse_patches(cargo_toml)
        for crate_name, info in patches.items():
            norm_url = normalize_git_url(info["git"])
            by_git_url[norm_url][crate_name].append(
                (repo_name, info["type"], info["value"])
            )

    errors = []

    # For each git repo, check that all crates from it use the same pin
    for git_url, crates in sorted(by_git_url.items()):
        # Collect all unique pins across all crates from this git URL
        pins_by_repo: dict[str, set[tuple[str, str]]] = defaultdict(set)
        for crate_name, usages in crates.items():
            for repo_name, pin_type, pin_value in usages:
                pins_by_repo[repo_name].add((pin_type, pin_value))

        # Each repo should use exactly one pin for a given git URL
        all_pins: set[tuple[str, str]] = set()
        for pins in pins_by_repo.values():
            all_pins.update(pins)

        if len(all_pins) <= 1:
            continue

        # There's a mismatch — format the error
        short_url = git_url.split("github.com/")[-1]
        lines = [f"MISMATCH for {short_url}:"]
        for repo_name, pins in sorted(pins_by_repo.items()):
            for pin_type, pin_value in sorted(pins):
                short_val = pin_value[:12] if pin_type == "rev" else pin_value
                lines.append(f"  {repo_name:30s} {pin_type}={short_val}")
        errors.append("\n".join(lines))

    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--workspace-dir",
        type=Path,
        default=Path(__file__).resolve().parent.parent.parent,
        help="Path to seismic-workspace root",
    )
    args = parser.parse_args()

    print(f"Checking patch consistency in {args.workspace_dir}\n")

    errors = check_sync(args.workspace_dir)

    if not errors:
        print("All patches are in sync.")
        sys.exit(0)

    print(f"Found {len(errors)} inconsistencies:\n")
    for err in errors:
        print(err)
        print()

    sys.exit(1)


if __name__ == "__main__":
    main()
