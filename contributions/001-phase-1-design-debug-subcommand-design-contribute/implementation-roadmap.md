# Implementation Roadmap: Debug Subcommand

## Overview

Implement the `diffviz debug` subcommand that exposes all 7 pipeline phases in JSON format for transparency into how DiffViz groups and renders code changes.

**Scope**: New command taking file path + git refs, running full ReviewEngineBuilder pipeline, outputting structured JSON with per-phase results.

**Reuses**: ReviewEngineBuilder, DiffProvider (GitRepository), CommandExecutor pattern, serde for serialization.

---

## Implementation Strategy: Core-then-Integrate

**Phases proceed sequentially** — each phase builds on the previous and leaves the code compiling/passing tests.

### Phase 1: Command Skeleton & Basic JSON Output
**Objective**: Register debug command, scaffold JSON structure, establish serialization patterns.

**Key Tasks**:
1. Create `diffviz-cli/src/commands/debug.rs` with `DebugCommand` struct
2. Implement `CommandExecutor` trait for DebugCommand
3. Parse CLI args: `--file`, `--from` (default: HEAD), `--to` (default: working_tree), `--phase`, `--explain-folding`, `--export-fixture`, `--human`
4. Register command in `main.rs` (add to Commands enum)
5. Define JSON output root struct with `#[derive(Serialize)]`:
   - file_path, language, query, metadata (analysis_duration_ms)
   - phases: HashMap<String, serde_json::Value> (placeholder)
6. Implement input validation: file exists, git refs valid, language supported
7. Output basic JSON structure (all phases empty for now)

**Completion Criteria**:
- `diffviz debug --file src/main.rs` outputs valid JSON with empty phases
- No warnings from cargo check/clippy
- All tests in other crates still pass

---

### Phase 2: ReviewEngineBuilder Integration & Phase Output
**Objective**: Populate Phase 2-7 JSON output by reusing ReviewEngineBuilder.

**Key Tasks**:
1. Modify DebugCommand to accept file_path + DiffQuery (from, to refs)
2. Get file language via existing `get_language_parser_for_file()` helper
3. Create minimal Decision/CodeImpact to feed ReviewEngineBuilder
4. Call `ReviewEngineBuilder::build_from_decisions()` to construct ReviewEngine
5. Extract ReviewState from ReviewEngine
6. For each phase, serialize:
   - **Phase 2 (SemanticTree)**: Walk semantic tree, output nodes with metadata
   - **Phase 3 (SemanticPairs)**: Extract matched/added/deleted pairs
   - **Phase 4 (ReviewableDiffs)**: Serialize reviewable_diffs from ReviewState (all diffs, no filtering yet)
   - **Phase 5 (DiffNode hierarchy)**: Walk diff_node tree, output structure + relevance scores
   - **Phase 6 (RenderableDiff)**: Serialize Myers diff lines
   - **Phase 7 (Final output)**: Same as Phase 6 for now
7. Populate metadata: analysis_duration_ms

**Serialization Pattern** (for domain types without built-in Serialize):
- Create wrapper structs with `#[derive(Serialize)]` for complex types
- Example: `SerializableNode { kind: String, start: usize, end: usize, ... }`
- Map domain types → wrapper structs before JSON serialization

**Completion Criteria**:
- `diffviz debug --file src/main.rs --from HEAD` outputs complete JSON with all phases
- JSON validates against design-doc schema
- No clippy warnings
- Phase output matches design expectations

---

### Phase 3: Line-Range Filtering
**Objective**: Implement `--line-range` flag to filter ReviewableDiffs by boundary overlap.

**Key Tasks**:
1. Parse `--line-range <start>-<end>` CLI argument
2. After ReviewEngineBuilder::build, filter reviewable_diffs:
   - Keep diffs where `diff.line_range.start <= range_end && diff.line_range.end >= range_start`
   - This mirrors TUI code-impact logic (shows contextually-related diffs)
3. Update Phase 4-7 JSON to reflect filtered diffs only
4. Add filtered_count and total_count to metadata
5. Add line_range_filter to JSON root (with start/end)

**Completion Criteria**:
- `diffviz debug --file src/main.rs --line-range 50-100` outputs only overlapping diffs
- Filtering logic matches TUI behavior
- Metadata correctly reports counts
- All tests pass

---

### Phase 4: --explain-folding Implementation
**Objective**: Add reasoning for each DiffNode's relevance score assignment.

**Key Tasks**:
1. Add optional `explain_folding` field to JSON phases (populated only if --explain-folding passed)
2. For each DiffNode, inspect:
   - `semantic_kind` (FunctionDef, TypeDecl, etc.)
   - `unit_type` (Boundary, Semantic, etc.)
   - `relevance_score` (normalized 0-1)
3. Generate explanation string:
   - Examples: "High relevance: Modified function signature", "Low relevance: Whitespace change"
   - Reference semantic_kind + change_type + impact
4. Add explanation to each node's JSON representation
5. Optionally add summary at phase level

**Complexity Note**: This requires understanding why folding assigned each score. If the scoring logic is opaque, add TODO for future refinement and output best-guess explanation based on metadata.

**Completion Criteria**:
- `diffviz debug --file src/main.rs --explain-folding` includes reasoning for each node
- Explanations are human-readable and informative
- Baseline output (without --explain-folding) unchanged

---

### Phase 5: --export-fixture Implementation
**Objective**: Export minimal ReviewFixture JSON for test data creation.

**Key Tasks**:
1. Define ReviewFixture struct (if not exists):
   ```rust
   #[derive(Serialize)]
   struct ReviewFixture {
       old_code: String,
       new_code: String,
       file_path: String,
       language: String,
   }
   ```
2. After ReviewEngineBuilder::build, extract old_code + new_code for file
3. Use DiffProvider::get_source_code() to fetch full content
4. Serialize ReviewFixture to JSON
5. Write to path specified in `--export-fixture <path>`

**Completion Criteria**:
- `diffviz debug --file src/main.rs --export-fixture /tmp/fixture.json` creates valid fixture
- Fixture contains only essentials (old_code, new_code, file_path, language)
- File is JSON-parseable

---

### Phase 6: --human Flag Implementation
**Objective**: Provide human-readable output alternative to JSON.

**Key Tasks**:
1. Add `--human` CLI flag parsing
2. If flag present, convert JSON output to formatted text:
   - File info header (path, language, query refs)
   - Per-phase summary (count of nodes, key changes)
   - Tree visualization for DiffNode hierarchy
   - Example: Indented tree with relevance scores visible
3. Use ANSI colors for readability (optional, based on terminal support)
4. Keep baseline JSON as primary output

**Complexity Note**: Text formatting is secondary; keep it simple. Focus on readability over visual flair.

**Completion Criteria**:
- `diffviz debug --file src/main.rs --human` outputs readable text (not JSON)
- Key information (file, language, query, phase summaries) visible
- Can be piped or saved to file without encoding issues

---

### Phase 7: Integration & Polish
**Objective**: Register command, add tests, ensure zero warnings.

**Key Tasks**:
1. Register DebugCommand in `diffviz-cli/src/main.rs`:
   - Add variant to Commands enum: `Debug { ... }`
   - Add handler in match statement
2. Run `cargo check --workspace` and `cargo clippy --workspace`
3. Fix all compiler/clippy warnings (ZERO WARNING RULE)
4. Add integration tests:
   - Test basic command invocation with valid file
   - Test invalid file path handling
   - Test invalid git ref handling
   - Test JSON output structure
   - Test line-range filtering
   - Test --export-fixture output
5. Run full test suite: `cargo test --workspace`
6. Update CLAUDE.md if needed (command documentation)

**Completion Criteria**:
- `diffviz debug` registered and callable from CLI
- Zero compiler/clippy warnings
- All new tests pass
- All existing tests still pass
- Help text accurate: `diffviz debug --help`

---

## Dependency Chain & Parallel Work

**Sequential phases** (each depends on previous):
- Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6 → Phase 7

**No parallel work** — features build on prior JSON structure.

---

## Key Technical Decisions (From design-log.md)

These constraints shape implementation:

- **D1: Full Pipeline Exposure** → All 7 phases output by default
- **D2: Line Range as Filter** → Filter after building, not before
- **D3: Git-Only Input** → Reuse DiffProvider, no stdin
- **D4: JSON Output Default** → Primary output is JSON
- **D5: Minimal Fixture Export** → Only old_code, new_code, file_path, language
- **D6: --explain-folding Optional** → Only on demand to keep output lean
- **D7: Phase 1 AST as Summary** → Structure outline, not full dump

---

## Architecture Alignment Checklist

- ✅ CommandExecutor pattern (Phase 1)
- ✅ Reuses ReviewEngineBuilder (Phase 2)
- ✅ Works through DiffProvider abstraction (Phase 2)
- ✅ ReviewableDiff as contract (Phase 3)
- ✅ Serde for serialization (Phase 2)
- ✅ Environment for DI (Phase 7)
- ✅ Zero compiler warnings (Phase 7)
- ✅ Clean git integration (Phase 2)

---

## File Changes Summary

**New Files**:
- `diffviz-cli/src/commands/debug.rs` — Core DebugCommand implementation

**Modified Files**:
- `diffviz-cli/src/main.rs` — Register Commands::Debug variant + handler
- `diffviz-cli/src/commands/mod.rs` — Export DebugCommand (if needed)

**No changes to**:
- Core crates (diffviz-core, diffviz-review, diffviz-git) — reuses existing APIs
- Other commands — debug command is isolated

---

## Testing Strategy

**Unit Tests** (in debug.rs):
- Input validation (file exists, language supported, git refs valid)
- JSON schema validation

**Integration Tests** (diffviz-cli/tests/):
- End-to-end CLI invocation
- Output parsing and correctness
- Line-range filtering accuracy
- Fixture export validity

**Manual Testing**:
- Run on real repositories
- Verify output matches design-doc schema
- Verify line-range filtering matches TUI behavior
- Compare --human output readability

---

## Definition of Done

Each phase is complete when:
1. All code compiles without warnings
2. All clippy lints pass
3. Existing tests still pass
4. New functionality tested
5. Code follows project patterns (Entity-centric, CommandExecutor, serde)
6. Git commits are atomic and clear

---

## Risk Mitigation

**Risk**: Serializing complex domain types (DiffNode, SemanticTree) may require custom serializers.
- **Mitigation**: Create wrapper structs with simple fields; map domain types before serialization.

**Risk**: Filtering logic must match TUI code-impact behavior exactly.
- **Mitigation**: Review TUI filtering code first; add regression tests comparing outputs.

**Risk**: --explain-folding logic may be opaque (relevance scoring is complex).
- **Mitigation**: If scoring logic is unclear, add TODO for future refinement. Output best-guess explanation based on node metadata.

---

## Success Criteria

- ✅ Command fully functional and documented
- ✅ All design decisions honored (7 phases, JSON output, filtering, etc.)
- ✅ Zero compiler/clippy warnings
- ✅ All tests pass
- ✅ Ready for agent integration (JSON output parseable)
- ✅ Architecture constraints respected (no breaking changes to core layers)
