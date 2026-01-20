# Decision Log - Phase 1 Implementation

## Decision: HashMap for Decision Index Lookup
**Context**: Need to map ReviewableDiffId to decisions affecting that code range
**Options Considered**:
- BTreeMap: Ordered but more complex for lookup
- HashSet per ReviewableDiffId: Redundant storage
- HashMap<ReviewableDiffId, Vec<u32>>: Simple, O(1) lookup

**Decision**: HashMap with Vec for decision numbers
**Reasoning**: Provides O(1) lookup performance and straightforward iteration over decisions affecting a specific diff

---

## Decision: Synthetic ReviewableDiffId Creation
**Context**: Decisions specify code impacts as file + line range, but ReviewableDiffId encapsulates query context
**Options Considered**:
- Store raw file/range pairs and convert on lookup: Complex lookup logic
- Create synthetic ReviewableDiffIds matching code impact ranges: Unified lookup with existing abstractions
- Separate index structure not using ReviewableDiffId: Breaks abstraction consistency

**Decision**: Create synthetic ReviewableDiffIds in ReviewDecisions::add_decision()
**Reasoning**: Maintains consistency with entity-centric design where ReviewableDiffId is the universal key. The synthetic IDs use a fixed query context (head_to_unstaged) which is the expected context for new contributions.

---

## Decision: Display-Only Decision Display in Phase 1
**Context**: Could add full navigation and filtering, but need to validate UX first
**Options Considered**:
- Full navigation in Phase 1: Risk over-engineering, tight coupling to TUI navigation system
- Display-only badges: Minimal, validates data structures and visual placement
- Defer all UI work to Phase 2: Miss opportunity for early feedback

**Decision**: Display-only decision badges in file list
**Reasoning**: Early UX validation without complex navigation changes. Answers key questions:
- Do decision numbers provide useful context in file list?
- Is badge placement clear to users?
- Do the hardcoded decisions reveal missing data structure needs?

Allows Phase 2 to build navigation with confidence in the foundation.

---

## Decision: Three-Level Confidence Scoring
**Context**: Need to model certainty of decision-to-code mapping
**Options Considered**:
- Binary (matched/unmatched): Too coarse, loses important nuance
- Numeric 0-100 scale: Over-engineered for MVP, harder to generate automatically
- Three levels (high/medium/low): Balanced granularity

**Decision**: Three-level Confidence enum
**Reasoning**:
- Matches common UX patterns (traffic light colors feasible)
- Sufficient for Phase 1 display and Phase 2 filtering
- Aligns with what dev-contribute can reasonably generate automatically
- Not over-engineered; can expand if needed in future phases

---

## Decision: ReviewDecisions as Centralized Collection
**Context**: Following entity-centric pattern, decisions need a collection type
**Options Considered**:
- Decisions directly in ReviewState with HashMap: Less encapsulation, harder to extend
- Separate ReviewDecisions collection type: Mirrors ReviewApprovals and ReviewInstructions patterns
- Embedded decision index in ReviewState: Mixes concerns

**Decision**: ReviewDecisions collection type with internal decision_index
**Reasoning**:
- Consistent with existing ReviewApprovals and ReviewInstructions patterns
- Encapsulates decision management and lookup logic
- Enables future features (filtering, statistics) without modifying ReviewState
- Clean separation of concerns: ReviewState never directly manipulates decisions

---

## Decision: Include No-Code Decisions
**Context**: Some decisions are architectural and don't produce code changes
**Options Considered**:
- Include with empty code_impacts vec: Preserves full decision context
- Exclude from model: Loses important information
- Separate model for architectural vs code decisions: Over-engineered

**Decision**: Include with empty code_impacts vec
**Reasoning**:
- Reviewers benefit from seeing all decisions, even pure architectural ones
- Helps understand design rationale without code context
- Simple to implement, no special casing in UI
- Future phases can filter if needed

---

## Decision: Hardcoded Test Decisions in main.rs
**Context**: Need test data for TUI before JSON loading is implemented
**Options Considered**:
- Load from test JSON file: Requires JSON loading (Phase 2 work)
- Hardcode in memory: Simple, no dependencies
- Use fixtures from MockDiffProvider: Mixed concerns, provider is about git data

**Decision**: Hardcode three sample decisions in main.rs
**Reasoning**:
- Enables immediate testing without Phase 2 prerequisites
- Creates realistic scenarios: single impacts, overlapping impacts, no-code decision
- Lives in test-only binary (main.rs), doesn't affect library code
- Can be replaced by JSON loader in Phase 2 without touching entity code

---

## Technical Insights & Lessons Learned

### Decision Index Creation Logic
When adding a decision, we create a synthetic ReviewableDiffId for each line range in each code impact. This design choice:
- **Pro**: Uses existing ReviewableDiffId semantics, consistent with entity-centric architecture
- **Pro**: O(1) lookup by reviewable_id when rendering
- **Con**: Creates multiple synthetic IDs for a single decision (one per line range per impact)
- **Con**: Synthetic IDs must use a fixed query context (head_to_unstaged)

**Future consideration**: If this becomes a bottleneck or causes issues with query context heterogeneity, could refactor to separate index type, but current approach keeps code simple for MVP.

### Integration with ReviewState Constructor
Updated both ReviewState::new() and ReviewState::with_review_data() to accept decisions parameter. This ensures:
- Decisions are always initialized (no None/undefined state)
- Consistency between new and deserialized states
- Fail-fast if decisions unavailable

### Export Surface Design
Re-exported decision types from lib.rs to enable TUI to create hardcoded decisions. Exports are minimal and focused:
- Entity types (Decision, CodeImpact, ChangeType, Confidence, DecisionLineRange)
- Collection type (ReviewDecisions)
- Enough for external code to construct decisions, but internal indexing remains encapsulated

## Open Questions for Phase 2

1. **JSON Loading**: Should JSON schema match these types exactly, or would a simpler format work better?
2. **UI Display**: Are badges the right visualization, or should decisions be in a separate sidebar?
3. **Decision Navigation**: Should selecting a decision show all affected code, or navigate through code sequentially?
4. **Overlapping Decisions**: When two decisions affect the same code, should we show both numbers or use a special indicator?
5. **Decision Comments**: Should reviewers be able to add comments specifically about decisions, or just about code?
