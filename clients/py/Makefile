.PHONY: install lint fmt fmt-check typecheck test ci

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

test: ## Run test suite
	uv run pytest tests/ -v

ci: fmt-check lint typecheck test ## Run all CI checks
