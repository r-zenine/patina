# Phase 1 Implementation: Context Handoff

## What Was Done

Implemented the `diffviz debug` subcommand skeleton that:
- Registers Debug variant in Commands enum with all 8 CLI flags
- Parses arguments: `--file` (required), `--from`, `--to`, `--phase`, `--explain-folding`, `--export-fixture`, `--human`, `--line-range`
- Validates inputs: file exists, language supported, phase in 1-7, line-range format valid
- Outputs JSON structure matching design doc schema with all 7 phase fields (currently empty)
- Outputs human-readable summary with `--human` flag

## Files Modified

- `diffviz-cli/src/commands/debug.rs` (new) — 350 lines with DebugCommand struct, validation, JSON output
- `diffviz-cli/src/commands/mod.rs` — Added `pub mod debug;`
- `diffviz-cli/src/main.rs` — Added Debug variant to Commands enum, added match handler, added DebugCommand import

## Architecture Decisions

1. **Wrapper Structs Strategy**: All JSON serialization uses custom Serialize-derived structs (DebugOutput, Metadata, Phases, DiffQueryOutput). This keeps domain types clean and lets us add JSON-only fields without touching core crates.

2. **Phase Field as serde_json::Value**: Each phase (1-7) is `Option<serde_json::Value>` in the Phases struct. This lets Phase 2-6 populate phases independently without restructuring output format.

3. **Input Validation Scope**: Validates file existence, language support, and argument format. Does NOT validate git refs (that's Phase 2's job when talking to ReviewEngineBuilder).

4. **Human Output Strategy**: Minimal placeholder in Phase 1. Defer full tree visualization to Phase 6. Current output shows file header and phase summary only.

## Testing

- ✅ Command compiles with zero warnings
- ✅ All workspace tests pass (185 passed, 7 ignored)
- ✅ Valid JSON output for supported languages (Rust, Python, Go, Java, C, C++, TypeScript, JavaScript)
- ✅ File not found rejection
- ✅ Unsupported extension rejection
- ✅ Invalid phase number rejection (must be 1-7)
- ✅ Invalid line-range format rejection
- ✅ --human flag produces human-readable output

## Known Constraints & Gotchas

1. **Language Detection**: Only file extension-based. Phase 2 won't change this—ReviewEngineBuilder already has language detection, we just extract it.

2. **Git Refs**: Phase 1 accepts `--from` and `--to` as optional strings with default display values. Phase 2 must validate these are actual git refs when calling ReviewEngineBuilder.

3. **Phase Output Granularity**: Each phase outputs a single `serde_json::Value`. If fine-grained filtering per phase is needed later, restructure the Phases struct—but likely not needed since `--phase` flag filters client-side.

4. **Line Range Parsing**: Currently parses as `start-end` format but doesn't apply the filter. Phase 3 applies the actual filtering logic post-ReviewEngineBuilder.

## Next Phase (Phase 2)

Phase 2 will:
1. Instantiate ReviewEngineBuilder with minimal Decision/CodeImpact
2. Extract ReviewState after build
3. Populate all 7 phase fields with reviewable diffs, semantic trees, renderable diffs, etc.
4. Record elapsed time and file stats in metadata

No changes needed to Phase 1 code for Phase 2 to work. Phase 2 only needs to fill in the phase fields in DebugOutput.

## Handoff Checklist

- [x] Command registered and callable
- [x] All CLI arguments parsed
- [x] Input validation complete
- [x] JSON output schema complete
- [x] Human output skeleton created
- [x] Zero compiler warnings
- [x] All tests passing
- [x] No changes to domain crates or other commands
- [x] Architecture constraints respected (CommandExecutor, Environment pattern, no core changes)
