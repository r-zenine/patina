# Code Context: VCS History Metrics

## diffviz-git — Git Infrastructure

### GitRepository (`diffviz-git/src/lib.rs`)
- `GitRepository::open()` — entry point, wraps `git2::Repository`
- `repo: Repository` field — has `repo.revwalk()` for commit iteration (not yet used)
- `resolve_commit_trees()` L113 — resolves commit refs to tree objects
- `create_git_diff()` L176 — produces `git2::Diff` between two trees
- `get_file_content_at_commit()` L200 — fetches file content at a commit
- **Missing**: no commit history walking; `revwalk()` is available via `git2` but not called anywhere

### Dependencies available in diffviz-git
- `git2 = "0.20"` — `Revwalk`, `Commit`, `Diff`, `Patch` all available
- `diffviz-core` — tree-sitter parsers directly available (can call `SemanticTree`)
- `diffviz-review` — can produce `DiffProvider`-typed results
- `chrono` — in workspace deps, not yet in diffviz-git Cargo.toml

## diffviz-review — Review Entities & Provider Trait

### DiffProvider trait (`diffviz-review/src/providers/mod.rs` L68)
- The dependency-inversion interface `diffviz-git` implements
- New `HistoryProvider` trait should follow this same pattern

### Decision & CodeImpact (`diffviz-review/src/entities/decision.rs`)
- `CodeImpact` L30 — `{ reasoning, file, line_ranges }` — model for attaching context to files
- `DecisionLineRange` L21 — `{ start, end }` — line range type already exists

### Entities mod (`diffviz-review/src/entities/mod.rs`)
- New `HistorySignals` entity should live here

## diffviz-core — AST / Tree-sitter

### SemanticTree (`diffviz-core/src/semantic_ast.rs`)
- `SemanticNode` — tree node with `source_range: SourceRange`
- `SourceRange` — `{ start_byte, end_byte, start_row, end_row }` — maps to line numbers
- Language parsers in `diffviz-core/src/parsers/` — 11 languages, all return `SemanticTree`

### LanguageParser (`diffviz-core/src/parsers/mod.rs`)
- The trait all language parsers implement
- Can parse file content string → `SemanticTree` — needed for Phase 2 node-to-line mapping

### create_reviewable_diff_from_range (`diffviz-core/src/decision_based_diff.rs`)
- Existing pipeline: line range → `SemanticTree` → `ReviewableDiff`
- Phase 2 needs a simpler path: file content → `SemanticTree` → node line ranges

## diffviz-review-tui

### Where file-level annotations surface
- TUI already shows per-file diffs with title annotations (cited ranges on ReviewableDiff)
- History signal display will hook into whatever renders the per-file review header
