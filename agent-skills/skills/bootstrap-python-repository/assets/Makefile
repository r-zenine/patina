.PHONY: lint fmt typecheck test sanity-check

lint:
	uv run ruff check .

fmt:
	uv run ruff format .

typecheck:
	uv run mypy .

test:
	uv run pytest

sanity-check: lint typecheck test
