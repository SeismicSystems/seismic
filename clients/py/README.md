# seismic-web3

Python SDK for the [Seismic](https://seismic.systems) privacy-enabled EVM, built as extensions on top of [web3.py](https://github.com/ethereum/web3.py).

## Prerequisites

- **Python 3.10+**
- **[uv](https://docs.astral.sh/uv/)** — install with `curl -LsSf https://astral.sh/uv/install.sh | sh`

## Local Development

```bash
# Clone the repo and navigate to the Python client
cd clients/py

# Install all dependencies (creates .venv automatically)
uv sync --dev

# Run the test suite
make test

# Run all CI checks locally (format, lint, typecheck, test)
make ci
```

## Running CI Locally

```bash
./ci.sh
```

This single script installs deps, checks formatting, lints, type-checks, and runs tests. It exits on the first failure.

## Available Commands

| Command | Description |
|---------|-------------|
| `make install` | Install all dependencies into venv |
| `make fmt` | Format code with ruff |
| `make fmt-check` | Check formatting without changes |
| `make lint` | Run ruff linter |
| `make typecheck` | Run ty type checker |
| `make test` | Run test suite with pytest |
| `make ci` | Run all checks (fmt-check + lint + typecheck + test) |

## Running Tools Directly

```bash
# Linter
uv run ruff check src/ tests/

# Formatter
uv run ruff format src/ tests/

# Type checker
uv run ty check src/

# Tests
uv run pytest tests/ -v
```

## Publishing to PyPI

### Building

```bash
# Build source distribution and wheel
uv build

# Verify it builds without custom sources (mimics how others will build it)
uv build --no-sources
```

This produces artifacts in `dist/`:
- `seismic_web3-<version>.tar.gz` (source distribution)
- `seismic_web3-<version>-py3-none-any.whl` (wheel)

### Publishing

```bash
# Publish to PyPI (requires authentication)
uv publish
```

**Authentication options:**
- **Token:** `uv publish --token <PYPI_TOKEN>` or set `UV_PUBLISH_TOKEN`
- **Trusted publishing (recommended for CI):** Configure a [trusted publisher](https://docs.pypi.org/trusted-publishers/) on PyPI for the GitHub repository, then `uv publish` works without credentials in GitHub Actions

### Installing

Once published, users install with:

```bash
pip install seismic-web3
```

Or with uv:

```bash
uv add seismic-web3
```

## CI

The GitHub Actions workflow (`.github/workflows/ci-py-client.yml`) runs on pushes to `main` and on pull requests that touch `clients/py/`:

- **lint** — ruff check + format verification (Python 3.13)
- **typecheck** — ty strict type checking (Python 3.13)
- **test** — pytest across Python 3.10, 3.11, 3.12, 3.13

## Project Structure

```
clients/py/
├── pyproject.toml          # Project config, dependencies, tool settings
├── ci.sh                   # Run full CI locally
├── Makefile                # Dev commands
├── .python-version         # Local Python version (3.13) for uv
├── src/
│   └── seismic_web3/       # Main package (import as seismic_web3)
│       ├── __init__.py
│       └── py.typed        # PEP 561 type marker
└── tests/
    └── test_placeholder.py
```
