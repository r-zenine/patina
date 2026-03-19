# Implementation Roadmap for Debug Subcommand

## Execution Strategy

**Strategy**: Core-then-Integrate
**Approach**: Build phases sequentially, each completing independently with tests passing before next begins. Reuse ReviewEngineBuilder to avoid pipeline duplication.

---

## Phase 1: Command Skeleton & Basic JSON Output

**Description**: Register debug command, scaffold JSON structure, establish serialization patterns for all 7 phases.

**Objectives**:
- **Implementation**: Create DebugCommand struct in `diffviz-cli/src/commands/debug.rs`, implement CommandExecutor trait
- **Implementation**: Parse CLI args: `--file` (required), `--from` (default: HEAD), `--to` (default: working_tree), `--phase`, `--explain-folding`, `--export-fixture`, `--human`
- **Implementation**: Define JSON output root struct with serializable fields: file_path, language, query, metadata, phases
- **Implementation**: Validate inputs: file exists, git refs valid, language supported via existing `get_language_parser_for_file()` helper
- **Implementation**: Register Debug variant in Commands enum in `main.rs` and add match handler

**Testing Criteria**:
- `diffviz debug --file src/main.rs` outputs valid JSON with empty phases
- Input validation rejects missing/invalid file paths and unsupported languages
- Help text correct: `diffviz debug --help`

**Dependencies**: None — fresh implementation

**Relevant Local Skills**: None

**Files to Modify**:
- `diffviz-cli/src/commands/debug.rs` (new) — DebugCommand struct and trait impl
- `diffviz-cli/src/main.rs` — Add Commands::Debug variant; add handler in match
- `diffviz-cli/src/commands/mod.rs` — Export DebugCommand (if public module)

---

## Phase 2: ReviewEngineBuilder Integration & Phase Output

**Description**: Populate all 7 phase outputs by reusing ReviewEngineBuilder and serializing domain types.

**Objectives**:
- **Implementation**: Modify DebugCommand to accept file_path + DiffQuery (from/to refs)
- **Implementation**: Create minimal Decision/CodeImpact to seed ReviewEngineBuilder::build_from_decisions()
- **Implementation**: Extract ReviewState from ReviewEngine; measure elapsed time
- **Implementation**: For each phase, create wrapper structs (SerializableNode, SerializableDiffNode, etc.) and serialize:
  - Phase 2 (SemanticTree): Walk semantic tree, output nodes with metadata
  - Phase 3 (SemanticPairs): Count matched/added/deleted pairs
  - Phase 4 (ReviewableDiffs): Serialize reviewable_diffs from ReviewState
  - Phase 5 (DiffNode hierarchy): Walk tree, output structure + relevance scores
  - Phase 6 (RenderableDiff): Serialize Myers diff lines
  - Phase 7 (Final output): Same as Phase 6 for now
- **Implementation**: Populate metadata: analysis_duration_ms, file size, diff stats

**Testing Criteria**:
- `diffviz debug --file src/main.rs --from HEAD` outputs complete JSON with all 7 phases
- Phase outputs match design-doc schema structure
- JSON parses and validates without errors
- No performance regression vs ReviewEngineBuilder baseline

**Dependencies**: Must complete Phase 1 first

**Relevant Local Skills**: None

**Files to Modify**:
- `diffviz-cli/src/commands/debug.rs` — ReviewEngineBuilder integration, wrapper structs, serialization logic

---

## Phase 3: Line-Range Filtering

**Description**: Implement `--line-range <start>-<end>` to filter ReviewableDiffs by boundary overlap.

**Objectives**:
- **Implementation**: Parse `--line-range` CLI argument
- **Implementation**: After ReviewEngineBuilder::build, filter reviewable_diffs using overlap logic: `start <= range_end && end >= range_start` (mirrors TUI code-impact)
- **Implementation**: Update Phase 4-7 JSON to reflect filtered diffs only
- **Implementation**: Add filtered_count and total_count to metadata; add line_range_filter object to JSON root

**Testing Criteria**:
- `diffviz debug --file src/main.rs --line-range 50-100` outputs only overlapping diffs
- Filtering logic produces identical results to TUI code-impact filtering on same inputs
- Metadata correctly reports filtered vs total counts

**Dependencies**: Must complete Phase 2 first

**Relevant Local Skills**: None

**Files to Modify**:
- `diffviz-cli/src/commands/debug.rs` — Line-range parsing and filtering logic

---

## Phase 4: --explain-folding Implementation

**Description**: Add reasoning for each DiffNode's relevance score assignment when flag passed.

**Objectives**:
- **Implementation**: Add optional `explanations` field to serialized DiffNode outputs
- **Implementation**: For each DiffNode, inspect semantic_kind, unit_type, relevance_score; generate human-readable explanation
- **Implementation**: Examples: "High relevance: Modified function signature", "Low relevance: Whitespace change"
- **Design**: Determine how to map relevance scores to explanation strings (inspect DiffNode fields + metadata)

**Testing Criteria**:
- `diffviz debug --file src/main.rs --explain-folding` includes explanations for each node
- Explanations are human-readable and informative
- Baseline output (without flag) unchanged
- No performance impact on non-flagged runs

**Dependencies**: Must complete Phase 2 first

**Relevant Local Skills**: None

**Files to Modify**:
- `diffviz-cli/src/commands/debug.rs` — Explanation generation logic

---

## Phase 5: --export-fixture Implementation

**Description**: Export minimal ReviewFixture JSON (old_code, new_code, file_path, language) for test data creation.

**Objectives**:
- **Implementation**: Create ReviewFixture struct with fields: old_code, new_code, file_path, language
- **Implementation**: After ReviewEngineBuilder::build, extract full source code via DiffProvider::get_source_code()
- **Implementation**: Serialize ReviewFixture to JSON and write to file specified in `--export-fixture <path>`

**Testing Criteria**:
- `diffviz debug --file src/main.rs --export-fixture /tmp/fixture.json` creates valid JSON fixture
- Fixture contains only 4 fields (no phase data)
- Fixture is JSON-parseable and matches ReviewFixture schema

**Dependencies**: Must complete Phase 2 first (needs DiffProvider access)

**Relevant Local Skills**: None

**Files to Modify**:
- `diffviz-cli/src/commands/debug.rs` — ReviewFixture struct, export logic, file writing

---

## Phase 6: --human Flag Implementation

**Description**: Provide human-readable text output alternative to JSON.

**Objectives**:
- **Implementation**: Add `--human` CLI flag parsing
- **Implementation**: If flag set, format JSON to readable text: file header (path, language, refs), per-phase summaries (node counts, change types), tree visualization of DiffNode hierarchy
- **Design**: Determine text format and tree visualization style (indentation, symbols, colors)

**Testing Criteria**:
- `diffviz debug --file src/main.rs --human` outputs readable text (not JSON)
- Key information visible: file path, language, git query, phase summaries
- Output pipeable and encodable without issues

**Dependencies**: Must complete Phase 2 first

**Relevant Local Skills**: None

**Files to Modify**:
- `diffviz-cli/src/commands/debug.rs` — Human format output logic

---

## Phase 7: Integration & Polish

**Description**: Register command fully, add comprehensive tests, ensure zero compiler warnings.

**Objectives**:
- **Implementation**: Run `cargo check --workspace` and `cargo clippy --workspace`; fix all compiler/clippy warnings (ZERO WARNING RULE)
- **Implementation**: Add integration tests in `diffviz-cli/tests/debug/`:
  - Test basic invocation with valid file
  - Test invalid file path rejection
  - Test invalid git ref rejection
  - Test JSON output structure validation
  - Test line-range filtering accuracy
  - Test --explain-folding output presence
  - Test --export-fixture file creation
  - Test --human output formatting
- **Implementation**: Run `cargo test --workspace` and verify all tests pass
- **Design**: Update CLAUDE.md if command documentation needed

**Testing Criteria**:
- `diffviz debug` fully registered and callable
- `cargo check --workspace` and `cargo clippy --workspace` produce zero warnings
- All new tests pass
- All existing tests still pass (no regressions)
- Help text accurate and complete

**Dependencies**: Must complete all Phases 1-6 first

**Relevant Local Skills**: None

**Files to Modify**:
- `diffviz-cli/tests/debug/` (new) — Integration test suite
- `diffviz-cli/src/commands/debug.rs` — Final polish
- `CLAUDE.md` (optional) — Command documentation

---

## Architecture Alignment

- ✅ CommandExecutor pattern (Phase 1)
- ✅ Reuses ReviewEngineBuilder (Phase 2) — no pipeline duplication
- ✅ Works through DiffProvider abstraction (Phase 2)
- ✅ ReviewableDiff as domain contract (Phase 2)
- ✅ Serde for serialization (Phase 2) — wrapper structs for non-Serialize types
- ✅ Environment for dependency injection (Phase 1)
- ✅ Zero compiler warnings (Phase 7) — ZERO WARNING RULE
- ✅ Isolated command — no changes to core crates or other commands

## Success Criteria

- ✅ All 7 phases output in JSON (decision-log D1)
- ✅ Line-range filtering via overlap logic (decision-log D2)
- ✅ Git-only input with DiffProvider reuse (decision-log D3)
- ✅ JSON primary output, --human optional (decision-log D4)
- ✅ ReviewFixture export minimal (decision-log D5)
- ✅ --explain-folding optional (decision-log D6)
- ✅ Phase 1 AST as outline (decision-log D7)
- ✅ Zero compiler/clippy warnings
- ✅ All tests passing
- ✅ Architecture constraints respected
