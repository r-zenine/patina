# Context Document: VCS History Metrics

## What We're Building

Three VCS history metrics that surface as context during diff review — not as separate reports.
Inspired by code-maat (Adam Tornhill's "Your Code as a Crime Scene"), adapted for agent-built codebases where social/team signals are irrelevant but structural signals are critical.

## Behavioral Specification

### Metric 1: Revision Hotspots
**What**: Count how many commits touched each file in the current diff, over a configurable time window (default: 90 days).

**Signal value**: High revision count = the file is actively churning. Combined with large size/complexity, it's the highest-risk file to review. In agent-built codebases, hotspots often indicate a file where agents kept iterating rather than converging — a sign the design wasn't right.

**Output**: Per file in the diff: `{ commit_count: u32, window_days: u32 }`. Displayed in the TUI file header during review.

### Metric 2: Logical Coupling
**What**: Detect pairs of files that frequently change in the same commit (co-change). Coupling degree = `shared_commits / avg(total_commits_A, total_commits_B)` as a percentage.

**Signal value**: Files that always change together either belong in the same module (and should be merged) or have a hidden dependency that the architecture doesn't make explicit. In agent-built codebases this is especially common — agents don't always respect module boundaries.

**Output**: For each file in the diff: list of strongly coupled partners with degree `{ coupled_file: String, degree: u8, shared_commits: u32 }`. Only report pairs above a threshold (default: 30% coupling). Displayed in TUI alongside the file being reviewed.

### Metric 3: Non-uniform Modules
**What**: Within a single file, identify which named AST nodes (functions, structs, impls) are stable vs. churning. A file where some functions haven't changed in 60 commits but others change constantly has a coherence problem.

**Signal value**: Stable code co-located with churning code is at risk — a change to the churning part can accidentally break the stable part. This is the canonical split candidate signal. Particularly relevant for agent code where files grow by accretion.

**Output**: Per file: `{ stable_nodes: Vec<String>, churning_nodes: Vec<String> }` with a coherence flag when variance is high. Displayed as an advisory note in the TUI ("Consider splitting: stable fns [A, B] co-located with churning fns [C, D]").

## Architecture Summary

### Data flow
```
git history (git2::Revwalk)
    → CommitRecord { hash, date, files_changed, line_diffs? }
    → HistoryAnalyzer (metric computation)
    → HistorySignals { hotspot, coupling, coherence }
    → Review TUI (per-file annotation)
```

### Layer placement
- **`patina-analysis`** (new crate): `HistoryProvider` trait, `CommitRecord`, `HistorySignals`, all metric types, and metric computation logic. Owns the analysis domain entirely.
- **`git-provider`** (renamed from `diffviz-git`): Exposes raw git data — implements `HistoryProvider` from `patina-analysis`. No analysis types leak into it.
- **`diffviz-core`**: Used in Phase 2 only, directly from `patina-analysis` to map line ranges → AST node names.
- **`patina-cli`** (new): dedicated CLI for history analysis, depends on `patina-analysis` and `git-provider`.
- **`diffviz-cli`** / **`diffviz-review-tui`**: unchanged, continue depending on `git-provider` and `diffviz-review`.

### Dependency graph
```
patina-cli (new)              diffviz-cli / diffviz-review-tui (unchanged)
    └── patina-analysis (new)      ├── diffviz-review
            ├── git-provider  ─────┤       └── diffviz-core
            └── diffviz-core       └── git-provider (renamed from diffviz-git)
```

### New crates
- `patina-analysis`: `HistoryProvider` trait, metric computation, `HistorySignals` types. Dependencies: `git-provider`, `diffviz-core`, `chrono`, `thiserror`.
- `patina-cli`: entry point for the analysis tool. Dependencies: `patina-analysis`, `git-provider`.
- `git-provider` is the rename of `diffviz-git` — a workspace rename only, no functional change in Phase 1.

## Constraints

- `diffviz-core` ZERO WARNING RULE — any work touching diffviz-core must fix all clippy/compiler warnings
- `diffviz-core` no string/regex for code analysis — tree-sitter only (Phase 2 must use SemanticTree, not line scanning)
- `diffviz-core` fail-fast — no defensive fallbacks
- History walk is bounded: configurable `--history-days` window (default 90) to keep it fast on large repos
- Phase 2 (non-uniform modules) only runs on files with supported tree-sitter languages; silently skips others (not a fallback — it's a scope filter)
