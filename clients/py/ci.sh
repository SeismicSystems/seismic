#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "==> Installing dependencies"
uv sync --dev

echo "==> Checking formatting"
uv run ruff format --check src/ tests/

echo "==> Running linter"
uv run ruff check src/ tests/

echo "==> Running type checker"
uv run ty check src/

echo "==> Running tests"
uv run pytest tests/ -v

echo "==> All checks passed"
