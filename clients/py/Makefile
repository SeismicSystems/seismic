.PHONY: install lint fmt fmt-check typecheck test test-integration-anvil test-integration-reth test-all ci

install: ## Install all dependencies into venv
	uv sync --all-extras --dev

lint: ## Run ruff linter
	uv run ruff check src/ tests/

fmt: ## Format code with ruff
	uv run ruff format src/ tests/

fmt-check: ## Check formatting without making changes
	uv run ruff format --check src/ tests/

typecheck: ## Run ty type checker
	uv run ty check src/

test: ## Run unit tests (no node required)
	uv run pytest tests/ -v --ignore=tests/integration

test-integration-anvil: ## Run integration tests against sanvil
	CHAIN=anvil uv run pytest tests/integration/ -v --timeout=120

test-integration-reth: ## Run integration tests against seismic-reth
	CHAIN=reth uv run pytest tests/integration/ -v --timeout=120

test-all: ## Run all tests (unit + integration)
	uv run pytest tests/ -v --timeout=120

ci: fmt-check lint typecheck test ## Run all CI checks
