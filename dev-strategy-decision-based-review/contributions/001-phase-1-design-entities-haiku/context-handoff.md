# Context Handoff - Phase 1 Decision-Based Review Implementation

## What Was Accomplished

### Data Model Foundation ✓
- Complete decision entity types (Decision, CodeImpact, Confidence, ChangeType, DecisionLineRange)
- ReviewDecisions collection type with internal decision index
- Proper integration into ReviewState maintaining entity-centric architecture
- 100% test coverage of entity operations

### Business Logic Integration ✓
- ReviewEngine now exposes decision query APIs without breaking existing review operations
- ReviewState properly initialized with decisions in all construction paths
- No changes to core semantic analysis (diffviz-core remains pure)
- Decisions stored only in diffviz-review layer as intended

### TUI Visualization ✓
- Decision badges display on file headers showing which decisions affect each file
- Decision badges display on individual diffs showing which decisions affect specific code blocks
- Display is non-intrusive, uses minimal space format: `D[1,2]` for decisions 1 and 2
- Hardcoded test data provides realistic scenarios for manual validation

### Clean Code & Testing ✓
- All 122 diffviz-review tests pass
- New decision entity tests validate serialization, indexing, lookup
- Code formatted and linted (no warnings in modified code)
- Design documented in decision-log.md with rationale

## Critical Implementation Details for Phase 2

### Decision Index Mechanics
The ReviewDecisions::add_decision() method creates synthetic ReviewableDiffIds for each code impact's line ranges:

```rust
// For each code impact in a decision:
for impact in &decision.code_impacts {
    for line_range in &impact.line_ranges {
        let reviewable_id = self.create_synthetic_reviewable_id(
            &impact.file,
            line_range.start,
            line_range.end,
        );
        // Maps this synthetic ID to decision number
        self.decision_index.entry(reviewable_id).or_default().push(decision_number);
    }
}
```

**Why this matters for Phase 2**:
- When a ReviewableDiffId exists in the review (from git diff analysis), there might be a corresponding synthetic ID from decision mapping
- Lookup via `get_decisions_for_diff(reviewable_id)` works because ReviewableDiffIds are deterministic
- **IMPORTANT**: The synthetic ID must have the same query context (head_to_unstaged) as actual diffs for matching to work

### Decision Index vs Actual Diffs
Critical distinction that Phase 2 must handle:

1. **Actual ReviewableDiffs** from git:
   - Created by diffviz-core semantic analysis
   - Stored in ReviewState::reviewable_diffs
   - Have real content, metadata, analysis results
   - Represent actual code changes

2. **Decision Index** mapping:
   - Created from code_impacts in Decision objects
   - Stored in ReviewDecisions::decision_index
   - Conceptual mappings, may not have corresponding actual diffs
   - Enable filtering and display of decisions affecting code regions

**For Phase 2 UI**: Will need to handle cases where:
- Decision affects code range, but no actual diff exists there (no-op code)
- Multiple decisions affect overlapping ranges
- Decision's code impact is broader than actual diff (function level vs hunk level)

### Query Context Design
All synthetic ReviewableDiffIds use `DiffQuery::head_to_unstaged()`:

```rust
ReviewableDiffId::new(
    DiffQuery::head_to_unstaged(),
    file.to_string(),
    LineRange { start_line: start, end_line: end, ... }
)
```

**Rationale**: Decisions represent changes in the contribution under review, which is HEAD (committed) vs unstaged (local changes). This matches the expected review context.

**For Phase 2**:
- JSON loader must preserve query context when creating decisions
- If other query contexts needed in future, would require changes to index matching logic

### TUI Display Integration

Current implementation in file_list.rs:

```rust
// For files: aggregate decisions from all diffs in file
let mut file_decisions = HashSet::new();
for diff_id in diff_ids {
    let decisions = review_engine.get_decisions_for_diff(diff_id);
    for decision in decisions {
        file_decisions.insert(decision.number);
    }
}
// Display: D[1,2,3]

// For individual diffs: show their specific decisions directly
let decisions = review_engine.get_decisions_for_diff(diff_id);
// Display: D[2] for decisions affecting this diff
```

**For Phase 2**:
- This display is a starting point only
- May want decision titles in tooltips
- May want color coding by confidence level
- May want filtering to hide low-confidence decisions
- Decision count could become overwhelming; consider max display limit

## Architecture Constraints to Maintain

### 1. Entity-Centric Design
All review data must be indexed by ReviewableDiffId. Don't introduce:
- Global decision lists without ReviewableDiffId context
- UI state that tracks which decisions are visible (put in ReviewDecisions or engine)
- Separate indexing structures that don't parallel ReviewableDiffId

### 2. Fail-Fast Error Handling
Following CLAUDE.md no-fallback rule:
- Decision loading must succeed completely or fail
- No partial decision loading with defaults
- No trying to recover from invalid decision data
- Invalid JSON should fail immediately, not silently skip bad records

### 3. No Backward Compatibility
Decision-based review is Phase 1 feature:
- Doesn't support reviews without decision mappings
- No graceful fallback to file-based view for missing decisions
- Old review files without decision data are incompatible
- This is intentional per decision-log constraints

### 4. Keep diffviz-core Pure
Never add decision concepts to core:
- Core handles semantic analysis only
- Decisions are review workflow metadata
- All decision logic belongs in diffviz-review
- Core ReviewableDiff untouched

## Key Files & Lines for Phase 2

### Essential Additions
- **diffviz-review/src/entities/decision.rs** (new): Complete entity definitions
  - Decision, CodeImpact, Confidence, ChangeType, DecisionLineRange types
  - ReviewDecisions collection and add_decision() method
  - Tests validate serialization and lookup

- **diffviz-review/src/engines/review_engine.rs** (lines ~810-845):
  - Three new public methods: set_decisions(), get_decisions_for_diff(), get_decision(), get_all_decisions()
  - These are the TUI's only interface to decision data

- **diffviz-review-tui/src/ui/components/file_list.rs** (lines 68-81, 128-138):
  - Decision badge display logic for files and individual diffs
  - Shows where UI integration points exist

### Testing & Fixtures
- No new test data needed in diffviz-review fixtures
- Tests focus on decision entity operations, not integration
- Phase 2 JSON loader tests will use test files

## Recommended Phase 2 Sequence

1. **Create JSON loader** (10-15% effort)
   - Deserialize decision-to-code-mapping.json into Decision objects
   - Validate JSON structure and fail fast on errors
   - Add to ReviewEngine or separate module

2. **Implement decision navigation state** (20-25% effort)
   - Add DecisionNavigationState parallel to NavigationState
   - Track selected decision, expanded state
   - Handle keyboard navigation (up/down/enter)

3. **Create UI components** (40-50% effort)
   - DecisionListPanel: Show list of decisions with code impact count
   - DecisionDetailPanel: Show decision rationale and code impacts
   - Decision to code navigation: "Jump to this impact"

4. **Add view toggle** (15-20% effort)
   - Keybinding to switch between file view and decision view
   - Maintain selection state when switching
   - Update draw function to render correct view

5. **Integration testing** (5-10% effort)
   - Load real JSON files
   - Validate decision navigation works end-to-end
   - Manual testing with actual contributions

## Data Structures Ready for Phase 2

### ReviewEngine Public API (Ready)
```rust
pub fn set_decisions(&mut self, decisions: ReviewDecisions)
pub fn get_decisions_for_diff(&self, reviewable_id: &ReviewableDiffId) -> Vec<&Decision>
pub fn get_decision(&self, number: u32) -> Option<&Decision>
pub fn get_all_decisions(&self) -> Vec<&Decision>
```

### ReviewState Field (Ready)
```rust
pub decisions: ReviewDecisions  // Initialized in all constructors
```

### Hardcoded Test Data (Ready)
- Three sample decisions in main.rs
- Demonstrates various scenarios
- Can be replaced by JSON loader without changing entity code

### Serialization Ready
- All decision types have serde derives
- Can serialize to JSON for validation
- Just need JSON schema definition and loader

## Migration Path from Hardcoded to JSON

When Phase 2 implements JSON loading:

1. **Before**: main.rs calls create_hardcoded_decisions()
2. **After**: main.rs would call load_decisions_from_json()
3. **Compatibility**: Same ReviewDecisions type, same ReviewEngine APIs
4. **No changes needed to**: ReviewState, ReviewEngine interface, TUI display code

The entity design enables this transition seamlessly.

## Performance Considerations

### Current Design Performance
- Decision lookup: O(1) HashMap access by ReviewableDiffId
- File decision aggregation: O(n) where n = diffs in file
- Serialization: Linear in decision count

### Potential Bottlenecks for Future
- If decisions become very large (1000+ decisions), HashMap overhead minimal
- File aggregation in TUI could be optimized with cache if needed
- Serialization not performance-critical for JSON loading

### No Optimization Needed Yet
Current implementation is simple and fast enough. Premature optimization would complicate code. Monitor performance during Phase 2 testing.

## Testing Strategy for Future Phases

### Unit Tests Already In Place
- Decision serialization/deserialization ✓
- Decision index lookup ✓
- Confidence and ChangeType enums ✓
- ReviewDecisions collection operations ✓

### Phase 2 Tests to Add
- JSON deserialization with valid/invalid data
- Decision index matching with actual ReviewableDiffs
- UI navigation state machine
- View toggle and selection preservation
- File list display with actual decisions

### Manual Testing Approach
Keep using main.rs with test fixtures:
- Provides consistent environment
- Can upgrade to real JSON files once Phase 2 loader ready
- Enables rapid iteration on UI layouts

## Success Criteria for Phase 2

- [ ] JSON loader successfully deserializes valid decision files
- [ ] DecisionNavigationState manages selection correctly
- [ ] UI components render without layout issues
- [ ] User can toggle between file and decision views
- [ ] Selecting a decision shows code impacts
- [ ] Jumping to impact shows correct code in diff view
- [ ] All existing tests pass with decision changes
- [ ] No clippy warnings in new code
- [ ] Manual testing with 3+ realistic decision mappings succeeds

## Questions Answered By Phase 1

✓ Do decision numbers provide useful review context? → Yes, badges show relevance
✓ Is the data model sufficient? → Yes, all Phase 1 use cases work
✓ Does entity-centric design work? → Yes, ReviewableDiffId mapping works well
✓ Can ReviewEngine expose decisions cleanly? → Yes, three simple methods sufficient
✓ Do hardcoded decisions reveal missing features? → No immediate gaps found

## Remaining Open Questions for Phase 2

1. **JSON Schema**: What level of detail should mapping include?
2. **Visualization**: Are badges ideal, or better in dedicated panel?
3. **Navigation**: Should decision view show all code at once, or sequential?
4. **Filtering**: Should users filter by confidence level, decision number, or file?
5. **Comments**: Separate comment layer per decision, or reuse instruction comments?

These will be answered through Phase 2 implementation and user feedback.
