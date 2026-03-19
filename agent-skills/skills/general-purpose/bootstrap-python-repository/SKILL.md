---
name: bootstrap-python-repository
description: Bootstrap a modern Python repository with production-grade DevX. Use when the user wants to create a new Python project from scratch and needs tooling setup (linting, type checking, testing, CI, optional CD). Triggers on requests like "bootstrap a Python repo", "set up a new Python project", "create a Python repository with good DevX", "scaffold a Python project".
---

# Bootstrap Python Repository

## Step 1: Ask the user (all at once)

1. **Layout**: flat or `src/`?
   - **Flat**: code at root level — idiomatic for applications/services you run
   - **`src/`**: code under `src/<package>/` — idiomatic for libraries you publish to PyPI
2. **CD**: Raspberry Pi via Tailscale + systemd, or CI only?
3. **direnv**: Do you use direnv for automatic venv activation and `.env` loading?
4. **Project name** and **initial runtime dependencies** (can be none)

## Step 2: Create folder structure

- If flat layout → read `references/flat-layout.md`
- If src/ layout → read `references/src-layout.md`

Create all directories and empty `__init__.py` files. Always create `tests/__init__.py`.

## Step 3: Generate files

### Always generate

**`pyproject.toml`** — generate with:
```toml
[project]
name = "<project-name>"
version = "0.1.0"
requires-python = ">=3.12"
dependencies = [<user deps>]

[dependency-groups]
dev = ["ruff>=0.9.0", "mypy>=1.13.0", "pre-commit>=4.0.0", "pytest>=8.0.0", "pytest-cov>=6.0.0"]

[tool.ruff]
line-length = 100
target-version = "py312"

[tool.ruff.lint]
select = ["E", "F", "I", "UP", "B", "SIM"]

[tool.mypy]
python_version = "3.12"
strict = true
ignore_missing_imports = true

[tool.pytest.ini_options]
testpaths = ["tests"]
```

**`Makefile`** — copy from `assets/Makefile`

**`.pre-commit-config.yaml`** — copy from `assets/.pre-commit-config.yaml`, add user's runtime deps to mypy's `additional_dependencies`

**`.editorconfig`** — copy from `assets/.editorconfig`

**`.gitignore`** — copy from `assets/.gitignore`

**`.github/workflows/ci.yml`** — copy from `assets/ci.yml`

### If direnv chosen

**`.envrc`**:
```
dotenv_if_exists .env
source .venv/bin/activate
```

### If CD chosen

**`docs/devx/cd.md`** — read `references/cd-raspberry-pi.md`, substitute actual project name

**`deploy/<project-name>.service`** — copy from `assets/service.template`, substitute project name

## Step 4: Install

```bash
uv sync --all-groups
uv run pre-commit install
# if direnv:
direnv allow
```

## Step 5: Show summary

Print the final folder tree and available `make` commands.

## Key decisions encoded in this skill

- **uv** is the package manager — not pip, poetry, or pdm
- **ruff** replaces black + flake8 + isort — one tool for lint and format
- **mypy strict** — full type safety from the start
- **Makefile** for task running — `[tool.uv.scripts]` does not exist in uv; avoid poethepoet/taskipy
- **`uv run` kept in Makefile** even when direnv is active — safety net, works either way
- **`.editorconfig`** — natively supported by Neovim 0.9+, no plugin needed
- **direnv `.envrc`** — sources venv + loads `.env` so tools work without `uv run` in the shell
- **CD via Tailscale** — no port forwarding needed; runner joins home network via Tailscale GitHub Action; deploy = `git pull && uv sync && systemctl restart`; no PyPI publishing
