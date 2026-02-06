# Code Context: Relevant Files and Functions

## Core Entity Definitions

### diffviz-review/src/entities/decision.rs
**Lines 12-28:** ChangeType and Confidence enum definitions (TO BE REMOVED)
- ChangeType: Addition | Modification | Deletion
- Confidence: High | Medium | Low
- Both have Serde derives for serialization

**Lines 30-35:** DecisionLineRange struct (UNCHANGED)
- Simple line range: start/end fields
- Used for mapping decisions to code locations

**Lines 40-46:** CodeImpact struct (TO BE MODIFIED)
- Current fields: file, line_ranges, change_type, confidence, reasoning
- Remove: change_type, confidence
- Keep: file, line_ranges, reasoning

**Lines 50-57:** Decision struct (TO BE MODIFIED)
- Current fields: number, title, summary, decision_log_line, code_impacts
- Remove: summary
- Add: rationale (Option<String>)
- Keep: number, title, decision_log_line, code_impacts

**Lines 71-156:** ReviewDecisions collection (UNCHANGED)
- add_decision(), build_index_from_review_state(), get_decision()
- Index building only uses CodeImpact.file and CodeImpact.line_ranges
- No changes needed

**Lines 270-923:** Test suite (TO BE UPDATED)
- create_test_decision() helper function
- Multiple test functions constructing Decision/CodeImpact structs
- All need updated to remove change_type, confidence, summary

## TUI Rendering

### diffviz-review-tui/src/ui/components/decision_details_panel.rs
**Lines 75-79:** Summary rendering (TO BE MODIFIED)
```rust
lines.push(Line::from(vec![Span::styled(
    &decision.summary,  // Change to decision.rationale
    Styles::primary(),
)]));
```

**Lines 139-162:** Change type and confidence rendering (TO BE REMOVED)
```rust
let change_type_str = match impact.change_type { /* ... */ };
let confidence_str = match impact.confidence { /* ... */ };
let confidence_style = match impact.confidence { /* ... */ };
```

## TUI Test Files (10 files, all need updates)

### diffviz-review-tui/tests/decision_approval_tests.rs
**Lines 39-76:** create_test_engine() constructs Decision structs
- All Decision construction includes: change_type, confidence, summary
- Update to remove these fields

### diffviz-review-tui/tests/decision_tree_expansion_tests.rs
**Similar pattern:** Decision construction with full field sets

### Other test files (8 more):
- keybinding_tests.rs
- panel_management_tests.rs
- leader_key_tests.rs
- input_mode_tests.rs
- core_navigation_tests.rs
- (and 3 more)

All follow same pattern: construct Decision/CodeImpact with change_type, confidence, summary

## Review Engine Integration

### diffviz-review/src/engines/review_engine.rs
**No changes needed** - ReviewEngine methods only query Decision structs, don't construct them

### diffviz-review/src/review_engine_builder.rs
**No changes needed** - Builder accepts Decision vec but doesn't construct them

## Import Statements to Update

### diffviz-review-tui tests
Current imports:
```rust
use diffviz_review::{ChangeType, CodeImpact, Confidence, Decision, DecisionLineRange};
```

Updated imports:
```rust
use diffviz_review::{CodeImpact, Decision, DecisionLineRange};
```

Remove ChangeType and Confidence from all imports.
