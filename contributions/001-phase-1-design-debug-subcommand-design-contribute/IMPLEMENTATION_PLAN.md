# Implementation Plan: Debug Subcommand

## Summary

This document is the master reference for implementing the `diffviz debug` subcommand. It ties together design decisions, architectural constraints, and phased implementation steps.

**What's been completed**: Design phase with full specifications and decision rationale.

**What's ready to build**: 7-phase implementation roadmap with clear task breakdown and success criteria.

---

## Design Phase Artifacts (Complete)

All design work is finished and ready for implementation:

### 1. **context-handoff.md**
- Problem statement: Agents need transparency into 7-phase pipeline
- Solution: `diffviz debug` subcommand with JSON output
- Reading guide: Where to find what

### 2. **design-doc.md**
- Full command structure with all flags
- Processing flow (7 phases)
- JSON output schema
- Implementation notes (file locations, serialization approach)

### 3. **decision-log.md**
- D1: Full pipeline exposure (all 7 phases by default)
- D2: Line range as filter (post-processing, not pre-processing)
- D3: Git-only input (no stdin)
- D4: JSON output default (with --human flag)
- D5: Minimal fixture export (old_code, new_code, file_path, language)
- D6: --explain-folding optional (on demand)
- D7: Phase 1 AST as summary (outline, not full dump)

**Key takeaway**: These decisions are intentional and tested. Don't second-guess them during implementation.

---

## Implementation Roadmap (7 Phases)

### Phase 1: Command Skeleton & Basic JSON Output
**Goal**: Register debug command, establish JSON structure

**Deliverables**:
- New file: `diffviz-cli/src/commands/debug.rs`
- Modified file: `diffviz-cli/src/main.rs` (register command)
- JSON root structure with all fields (phases initially empty)
- Input validation (file exists, language supported)

**Success**: `diffviz debug --file src/main.rs` outputs valid JSON with empty phases

---

### Phase 2: ReviewEngineBuilder Integration & Phase Output
**Goal**: Populate all 7 phases with semantic analysis results

**Key tasks**:
- Create minimal Decision to feed ReviewEngineBuilder
- Call ReviewEngineBuilder::build_from_decisions()
- Extract ReviewState
- Serialize each phase:
  - Phase 2: SemanticTree nodes
  - Phase 3: SemanticPairs (matched/added/deleted)
  - Phase 4: ReviewableDiffs
  - Phase 5: DiffNode hierarchy
  - Phase 6: RenderableDiff (Myers diff)
  - Phase 7: Final output
- Add metadata (analysis_duration_ms)

**Serialization approach**:
- Use wrapper structs with `#[derive(Serialize)]` for complex types
- Map domain types → wrappers before JSON serialization
- Examples: SerializableNode, SerializableDiffNode, etc.

**Success**: `diffviz debug --file src/main.rs --from HEAD` outputs complete JSON matching design-doc schema

---

### Phase 3: Line-Range Filtering
**Goal**: Implement `--line-range <start>-<end>` flag

**Key tasks**:
- Parse CLI argument
- After ReviewEngineBuilder, filter reviewable_diffs:
  - Keep: `diff.start <= range_end && diff.end >= range_start`
  - This mirrors TUI code-impact logic
- Update phases 4-7 to reflect filtered diffs
- Add metadata: filtered_count, total_count

**Success**: Filtering logic matches TUI behavior; metadata reports counts correctly

---

### Phase 4: --explain-folding Implementation
**Goal**: Add reasoning for each DiffNode's relevance score

**Key tasks**:
- For each DiffNode, inspect: semantic_kind, unit_type, relevance_score
- Generate explanation (e.g., "High relevance: Modified function signature")
- Add explanation to node JSON (only if --explain-folding passed)
- Baseline output unchanged (without flag)

**Complexity**: If scoring logic is opaque, add TODO and output best-guess explanation based on metadata

**Success**: Explanations are human-readable and informative

---

### Phase 5: --export-fixture Implementation
**Goal**: Export minimal ReviewFixture JSON for test data

**Key tasks**:
- Define ReviewFixture struct (old_code, new_code, file_path, language)
- Fetch old/new code via DiffProvider::get_source_code()
- Serialize ReviewFixture
- Write to path specified in --export-fixture

**Success**: Fixture is valid JSON, contains only essentials

---

### Phase 6: --human Flag Implementation
**Goal**: Provide human-readable text output alternative to JSON

**Key tasks**:
- Parse --human CLI flag
- Convert JSON to formatted text:
  - File info header
  - Per-phase summary
  - Tree visualization for DiffNode hierarchy
- Keep implementation simple (readability over visual flair)

**Success**: Output is readable, key information visible, no encoding issues

---

### Phase 7: Integration & Polish
**Goal**: Register command, test, ensure zero warnings

**Key tasks**:
- Register DebugCommand in main.rs
- Run `cargo check --workspace` and `cargo clippy --workspace`
- Fix ALL compiler/clippy warnings (ZERO WARNING RULE)
- Add integration tests (valid file, invalid inputs, JSON structure, filtering, fixture)
- Run `cargo test --workspace`
- Update CLAUDE.md if needed

**Success**: Command callable, zero warnings, all tests pass

---

## Architecture Context (Key Insights)

### Reusable Patterns
- **ReviewEngineBuilder**: Orchestrates 7-phase pipeline; reuse it
- **DiffProvider**: Git abstraction; use it via Environment
- **CommandExecutor**: Trait for CLI commands; implement it
- **Serde**: Use for JSON serialization; create wrapper structs for complex types
- **Environment**: Dependency injection container; access via execute() parameter

### Critical Constraints (From CLAUDE.md)
1. **No fallbacks**: Fail fast, don't add defensive programming
2. **Zero warnings**: Fix all compiler + clippy warnings immediately
3. **Architecture rules**: Respect layer boundaries, no circular deps
4. **Reuse, don't reinvent**: Don't duplicate ReviewEngineBuilder or DiffProvider logic

### Serialization Pattern
```rust
#[derive(Serialize)]
struct SerializableNode {
    kind: String,
    start: usize,
    end: usize,
    // ...
}

// Map domain → wrapper:
fn serialize_node(node: &DomainNode) -> SerializableNode {
    SerializableNode { ... }
}
```

### File Locations (Ready to Reference)
- ReviewEngineBuilder: `diffviz-review/src/review_engine_builder.rs`
- DiffProvider trait: `diffviz-review/src/providers/mod.rs`
- CommandExecutor: `diffviz-cli/src/commands/mod.rs`
- CLI entry: `diffviz-cli/src/main.rs`

---

## Execution Checklist

### Before Starting
- [ ] Read context-handoff.md (understand problem/solution)
- [ ] Read design-doc.md (understand command structure)
- [ ] Read decision-log.md (understand why each design choice)
- [ ] Read architecture-context.md (understand how to build it)
- [ ] Familiarize with ReviewEngineBuilder, DiffProvider, CommandExecutor patterns

### During Implementation
- [ ] Follow 7-phase roadmap sequentially (no jumping ahead)
- [ ] After each phase, run `cargo check --workspace` and `cargo clippy --workspace`
- [ ] Fix warnings immediately (don't accumulate)
- [ ] Commit atomically after each phase with clear messages
- [ ] Run `cargo test --workspace` to ensure no regressions

### After Completion
- [ ] `cargo check --workspace` — zero warnings
- [ ] `cargo clippy --workspace` — zero warnings
- [ ] `cargo test --workspace` — all tests pass
- [ ] `diffviz debug --help` — help text accurate
- [ ] Manual test on real repo (verify JSON structure, filtering behavior)
- [ ] Code review checklist:
  - [ ] Follows CommandExecutor pattern
  - [ ] Reuses ReviewEngineBuilder, DiffProvider
  - [ ] JSON output matches design-doc schema
  - [ ] All flags implemented (--line-range, --explain-folding, --export-fixture, --human)
  - [ ] Tests cover happy path + error cases

---

## Risk Mitigation

### Risk 1: Serializing Complex Domain Types
**Problem**: DiffNode, SemanticTree may not derive Serialize.
**Mitigation**: Create wrapper structs with simple fields; map domain types before serialization.
**Pattern**: See SerializableNode example in architecture-context.md

### Risk 2: Filtering Logic Differs from TUI
**Problem**: Line-range filtering must match TUI code-impact behavior exactly.
**Mitigation**:
1. Review TUI filtering code first
2. Add regression tests comparing outputs
3. Test on same file with same line ranges

### Risk 3: --explain-folding Logic Unclear
**Problem**: Relevance scoring is complex; may not understand why each score was assigned.
**Mitigation**:
1. Inspect node metadata (semantic_kind, unit_type, change_type)
2. Output best-guess explanation based on metadata
3. Add TODO comment for future refinement
4. Don't block implementation on perfect explanations

### Risk 4: JSON Output Bloat
**Problem**: Full phase output for large files may be memory-intensive.
**Mitigation**:
1. Test on large files early (Phase 2)
2. If memory is issue, implement pagination (Phase 7 polish)
3. Design-doc allows --phase flag for selective output
4. This is future optimization; not MVP requirement

---

## Definition of Done

✅ **Code**:
- All code compiles without warnings
- All clippy lints pass
- Follows project patterns (CommandExecutor, serde, Environment)
- Reuses existing abstractions (no duplication)

✅ **Functionality**:
- All design decisions honored (7 phases, JSON, filtering, etc.)
- All flags implemented (--line-range, --explain-folding, --export-fixture, --human)
- JSON output matches design-doc schema
- Filtering logic verified against TUI behavior

✅ **Testing**:
- Input validation tested (invalid file, bad git ref, unsupported language)
- JSON structure validated (parse output with serde)
- Line-range filtering tested
- Fixture export tested
- End-to-end CLI invocation tested
- All existing tests still pass

✅ **Documentation**:
- Command registered in main.rs
- Help text accurate (`diffviz debug --help`)
- Comments explain non-obvious serialization logic
- Code follows existing style conventions

---

## Next Steps

1. **Start with Phase 1**: Create debug.rs, register command in main.rs
2. **Establish JSON structure**: Define root struct, implement CommandExecutor trait
3. **Verify compiles**: `cargo check --workspace` with zero warnings
4. **Move to Phase 2**: Integrate ReviewEngineBuilder, populate phases
5. **Test JSON output**: Verify it parses and matches design-doc schema
6. **Iterate through remaining phases**: Each phase is self-contained
7. **Final polish**: Integration testing, zero warnings, commit to main

---

## Files in This Contribution

This contribution folder contains the complete design and implementation roadmap:

- **context-handoff.md** — Problem, solution, reading guide
- **design-doc.md** — Command structure, processing flow, JSON schema
- **decision-log.md** — All design decisions with rationale
- **architecture-context.md** — Codebase overview, patterns, file locations
- **implementation-roadmap.md** — Detailed 7-phase implementation plan
- **IMPLEMENTATION_PLAN.md** — This file (master reference)

**Ready to hand off to implementer** ✅

---

## Quick Reference: Key Commands

```bash
# Check for warnings
cargo check --workspace
cargo clippy --workspace

# Run tests
cargo test --workspace
cargo test --package diffviz-cli

# Build and test specific crate
cargo build --package diffviz-cli
cargo test --package diffviz-cli

# Run the debug command (once implemented)
diffviz debug --file src/main.rs
diffviz debug --file src/main.rs --from HEAD --to working_tree
diffviz debug --file src/main.rs --line-range 50-100
diffviz debug --file src/main.rs --explain-folding
diffviz debug --file src/main.rs --export-fixture /tmp/fixture.json
diffviz debug --file src/main.rs --human
```

---

## Questions? Go Back To...

- **Why do we need this?** → context-handoff.md
- **What does it do?** → design-doc.md
- **Why this design?** → decision-log.md
- **How do we build it?** → architecture-context.md + implementation-roadmap.md
- **Where's the code?** → architecture-context.md (file locations)
- **What's the execution plan?** → This document + implementation-roadmap.md

All artifacts are complete and ready for implementation.
