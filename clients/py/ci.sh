#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

# ---------------------------------------------------------------------------
# Flags
# ---------------------------------------------------------------------------
run_anvil=true
run_reth=true

usage() {
  cat <<EOF
Usage: $0 [OPTIONS]

Run CI checks: lint, typecheck, unit tests, and integration tests.

Options:
  --no-anvil, -A    Skip anvil integration tests
  --no-reth,  -R    Skip reth integration tests
  --no-integration, -I  Skip all integration tests
  -h, --help        Show this help
EOF
  exit 0
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -A|--no-anvil)  run_anvil=false ;;
    -R|--no-reth)   run_reth=false ;;
    -I|--no-integration) run_anvil=false; run_reth=false ;;
    -h|--help)      usage ;;
    *) echo "Unknown option: $1" >&2; usage ;;
  esac
  shift
done

# ---------------------------------------------------------------------------
# Static checks + unit tests
# ---------------------------------------------------------------------------

echo "==> Installing dependencies"
uv sync --locked --dev

echo "==> Checking formatting"
uv run ruff format --check src/ tests/

echo "==> Running linter"
uv run ruff check src/ tests/

echo "==> Running type checker"
uv run ty check src/

echo "==> Running unit tests"
uv run pytest tests/ -v --ignore=tests/integration

# ---------------------------------------------------------------------------
# Integration tests
# ---------------------------------------------------------------------------

if $run_anvil; then
  echo "==> Running integration tests (anvil)"
  CHAIN=anvil uv run pytest tests/integration/ -v --timeout=120
fi

if $run_reth; then
  echo "==> Running integration tests (reth)"
  CHAIN=reth uv run pytest tests/integration/ -v --timeout=120
fi

echo "==> All checks passed"
