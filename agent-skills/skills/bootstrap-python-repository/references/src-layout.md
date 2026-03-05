# src/ Layout Structure

Use for libraries — things you publish to PyPI that others import.

```
<project-name>/
├── src/
│   └── <package-name>/
│       ├── __init__.py
│       ├── <module1>/
│       │   └── __init__.py
│       └── <module2>/
│           └── __init__.py
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
└── pyproject.toml
```

With src/ layout, add to `pyproject.toml`:
```toml
[tool.setuptools.packages.find]
where = ["src"]
```

The `src/` wrapper prevents accidental imports before the package is installed,
which is critical for libraries where consumers install via pip/uv.
