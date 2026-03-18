# Debug Subcommand - Context Handoff

## Problem
Agents and developers need transparency into why DiffViz grouped/rendered changes a specific way. Currently no way to introspect the 7-phase pipeline.

## Solution Overview
New `diffviz debug` subcommand that exposes all pipeline phases in JSON format:
- Takes: file path, git refs (from/to), optional line range
- Runs: full ReviewEngineBuilder pipeline (phases 1-7)
- Outputs: JSON with per-phase results, optional `--explain-folding` for relevance reasoning
- Exports: minimal ReviewFixture JSON for test data

## Design Highlights
- **Full transparency**: All 7 phases shown (configurable via `--phase` flags)
- **Line range as filter**: Process entire file, then filter ReviewableDiffs by boundary overlap (mirrors TUI code-impact logic)
- **Git-only input**: Real diffs with git context (agents work with real repos)
- **JSON default**: Machine-parseable output; `--human` flag for readability
- **Minimal fixture**: Export only old/new code + metadata

## Reading Guide
- **For implementers**: See `design-doc.md` for command structure and JSON schema
- **For decisions**: See `decision-log.md` for rationale on each choice
- **Key file**: `diffviz-cli/src/commands/debug.rs` (new) + update `main.rs`
