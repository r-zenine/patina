# Flat Layout Structure

Use for applications and services — things you run, not distribute.

```
<project-name>/
├── <module1>/
│   └── __init__.py
├── <module2>/
│   └── __init__.py
├── tests/
│   └── __init__.py
├── .github/
│   └── workflows/
│       └── ci.yml
├── docs/
│   └── devx/
│       └── cd.md        # only if CD chosen
├── deploy/              # only if CD chosen
│   └── <project>.service
├── .editorconfig
├── .envrc               # only if direnv chosen
├── .gitignore
├── .pre-commit-config.yaml
├── Makefile
├── pyproject.toml
└── main.py              # entry point
```

Standard modules for an agents project: `agents/`, `tools/`, `channels/`, `storage/`, `skills/`

Adapt module names to the project's domain.
