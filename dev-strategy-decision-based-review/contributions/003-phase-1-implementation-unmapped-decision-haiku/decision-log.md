# Additional Decisions - Phase 1 Implementation: Synthetic Decision 0

## Implementation Approach: Call create_unmapped_decision() After build_index_from_review_state()

- **Category**: API Flow Design
- **Decision**: The `create_unmapped_decision()` method is called after `build_index_from_review_state()` to ensure the decision_index is fully populated before identifying unmapped diffs
- **Rationale**: Ensures correct detection of unmapped diffs by comparing against complete index. Fail-fast approach: if a diff isn't in the index, it's unmapped.
- **Impact**: Two-phase indexing flow: (1) build main index, (2) create Decision 0 for unmapped. Order matters and is enforced by method naming/ordering.
- **Implications for Next Contributors**: ReviewEngine will eventually coordinate this two-phase call in a single method like `set_decisions_with_index()` which does both calls in sequence.

## Confidence Level: Medium for Naming (Confidence in Line Ranges)

- **Category**: CodeImpact Confidence Selection
- **Decision**: Use `Confidence::Medium` for all unmapped diffs in Decision 0
- **Rationale**: Unmapped code is detected via absence from index (100% certain it's unmapped). However, the mapping of that diff to Decision 0 is somewhat arbitrary - we don't actually know if it should be mapped. Medium reflects "we know it's unmapped, but that's the extent of our knowledge"
- **Alternatives Considered**:
  - `Confidence::Low` - Too pessimistic, we're confident it's unmapped
  - `Confidence::High` - Too optimistic, we don't know what it should be mapped to
  - `Confidence::Medium` - Chosen: realistic confidence in synthetic mapping
- **Impact**: UI can use this confidence level to visually distinguish unmapped changes from confident mappings
- **Implications for Next Contributors**: When dev-contribute implements actual mapping generation, it should use `High` for confident mappings and `Low` for uncertain ones. UI should eventually filter/sort by confidence.

## Code Impact Granularity: Per-Diff CodeImpact Entries

- **Category**: Data Organization
- **Decision**: Create one CodeImpact entry per unmapped diff (not grouped by file)
- **Rationale**: Preserves the 1:1 mapping between unmapped diffs and code impacts. Each diff has its own CodeImpact with exact line ranges. This matches the semantic model where each ReviewableDiff is a distinct semantic unit.
- **Alternative Considered**: Group all unmapped diffs from same file into single CodeImpact with multiple line ranges
  - **Why Discarded**: Would lose precision about which exact diffs are unmapped. Would make code_impacts harder to navigate in UI (need to parse multiple ranges per impact).
- **Impact**: Decision 0's code_impacts vector size = number of unmapped diffs. UI can iterate directly over impacts to navigate unmapped code.
- **Implications for Next Contributors**: When building TUI components, each code_impact in Decision 0 maps 1:1 to a ReviewableDiff. Makes file view filtering straightforward.

## Decision 0 Creation Only When Needed

- **Category**: Behavior Design
- **Decision**: Create Decision 0 only when there are actually unmapped diffs; don't create it for empty contributions
- **Rationale**: Simplifies navigation: if all diffs are mapped, no need for synthetic decision. Matches fail-fast principle: only create what's necessary.
- **Alternative Considered**: Always create Decision 0, even if empty
  - **Why Discarded**: Clutters decision list, confuses users about whether review is complete, violates principle of minimal viable slices
- **Impact**: Decision list only shows Decision 0 when needed. Users won't see "Unmapped Changes" if everything is already decided.
- **Implications for Next Contributors**: Navigation code should handle "no Decision 0 exists" gracefully. Don't assume Decision 0 is always present.

## Test Scenarios Validated

- **All Diffs Unmapped**: Decision 0 captures all diffs with correct line ranges
- **All Diffs Mapped**: Decision 0 not created; decision count unchanged
- **Mixed Mapped/Unmapped**: Decision 0 created with only unmapped diffs; mapped diffs stay with original decisions
- **Multiple Unmapped Diffs**: Handles various files and line ranges correctly
- **Preservation of Existing Decisions**: Adding Decision 0 doesn't modify or corrupt existing decisions 1+

## Open Questions for Future Contributors

1. **When should create_unmapped_decision() be called?** Currently manual API. Should it be automatic in ReviewEngine.set_decisions_with_index()? Yes - next contributor should add this orchestration.

2. **Should unmapped diffs be visible without Decision 0?** Design says no (fail-fast: Decision 0 only created when needed). But TUI could allow filtering "show all unmapped" separately. Defer to TUI implementation.

3. **How should Decision 0 sort in decision list?** Currently would sort to position 0 (lowest number). This puts unmapped first. Alternative: put at end? Decision for TUI implementation to determine via UI/UX review.
