# Decision Log - Phase 1 Architecture Review

## Decision: ReviewableDiffId Mapping Strategy Revision

**Date**: 2026-01-20
**Category**: Architectural Refinement
**Phase**: Phase 1
**Status**: Approved for Implementation

### Original Decision (from 001-design-entities)
Create synthetic ReviewableDiffIds from CodeImpact data to build decision_index mapping.

### Revised Decision
Use existing ReviewableDiffIds from ReviewState by detecting line range overlaps between CodeImpacts and actual ReviewableDiffs.

### Rationale
1. **Grounded in Reality**: Decisions map only to diffs that are actually being reviewed
2. **Simpler Semantics**: CodeImpact line_ranges = "which code region does this decision affect"
3. **No Orphaned Data**: Every mapped ID corresponds to an actual review item
4. **Reuses Existing Logic**: Overlap detection already proven in instruction validation
5. **Aligns with Architecture**: ReviewState is the source of truth for review items

### Impact Summary
- **Code Changes**: Minimal - internal to ReviewDecisions
- **API Surface**: No changes - same public methods
- **Behavior**: Same visible behavior, cleaner internals
- **Risk Level**: Low - reusing proven patterns

### Implementation Approach
1. Add `build_index_from_review_state()` method to ReviewDecisions
2. Update ReviewEngine to call this method when setting decisions
3. Update Phase 1 test binary to use new indexing method
4. Add unit tests for overlap matching logic

### No Backward Compatibility Issues
- This is a refinement during Phase 1 implementation
- No released APIs affected
- No saved state format changes

### Next Steps
- Implement the revised architecture in Phase 1 code
- Verify existing Phase 1 tests still pass
- Update decision-to-code-mapping.json JSON schema documentation to clarify semantics

---

## Decision: Decision-First Navigation Hierarchy

**Date**: 2026-01-20
**Category**: UX Architecture
**Phase**: Phase 1
**Status**: Approved for Implementation

### Original Approach (from Roadmap)
- Primary navigation: File-based (existing pattern)
- Decision labels: Displayed as badges/prefixes in file view
- Decision view: Toggleable alternative view mode

### Revised Approach
**Decision-first navigation as the primary (and only) navigation pattern.**

Navigation hierarchy:
```
Decision List (primary view)
  ↓ [select decision, press Enter]
Decision Detail Modal (shows summary, rationale, code impacts)
  ↓ [select file from impacts, press Enter]
File View (shows chunks for selected decision in selected file)
  ↓ [select chunk]
Chunk Detail (existing diff view with comments/approval)
```

### Key Design Choices

**1. Unmapped Code Handling: Synthetic Decision 0**
- Create "Decision 0: Unmapped Changes" for ReviewableDiffs with no decision mapping
- Ensures all code is accessible through decision navigation
- No special fallback modes needed

**2. Decision Detail as Modal**
- Minimizes TUI architecture changes
- Modal pattern already exists in codebase
- Quick context view without full navigation state changes

**3. Actions Remain at Chunk Level**
- Comments/Instructions: At ReviewableDiffId level (unchanged)
- Approval: At ReviewableDiffId level (unchanged)
- Decisions are navigation/context only, not action targets

### Rationale

**1. Aligns with Core Vision**
"Decision-based review" means decisions are first-class, not just metadata. Making them primary navigation delivers on the promise.

**2. Better Review Workflow**
- Reviewer understands WHY before seeing WHAT
- All impacts of a decision reviewed as cohesive unit
- Context is front-and-center, not secondary

**3. Simpler Mental Model**
- One navigation path (decision → file → chunk)
- No view toggle confusion
- Decisions aren't just "another way to see the same thing"

**4. Cleaner Architecture**
- No dual navigation states to maintain
- No view mode synchronization complexity
- Single source of truth for navigation hierarchy

### Impact Summary

**Phase 1 Changes**:
- Larger scope than original "add badges" approach
- Delivers complete decision-based UX immediately
- Better validates the concept before Phase 2/3 commitment

**TUI Architecture**:
- Existing NavigationState pattern still applies
- Add DecisionNavigationState as primary state
- Reuse FileNavigationState for file/chunk drilling
- Modal pattern for decision detail view

**Data Requirements**:
- Must generate synthetic Decision 0 for unmapped diffs
- Must track current position in decision → file → chunk hierarchy

**Risk Level**: Medium
- More complex than badge display
- But delivers complete feature for validation
- Better investment if decision-based approach works

### Implementation Approach

1. Extend ReviewDecisions with `create_unmapped_decision()` method
2. Add DecisionNavigationState to TUI state
3. Create DecisionListComponent (primary view)
4. Create DecisionDetailModal component
5. Update FileListComponent to filter by selected decision
6. Wire navigation flow: decision → modal → file → chunk

### Backward Compatibility
- N/A - Phase 1 implementation, no released features yet

### Next Steps
- Update implementation roadmap with new Phase 1 steps
- Begin implementation with DecisionNavigationState
- Create test data ensuring both mapped and unmapped diffs exist
